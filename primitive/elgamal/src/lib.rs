// Copyright (C) 2020-2023 Invers (JP) INC.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

use core::ops::{Add, Sub};
use num_traits::{CheckedAdd, CheckedSub};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use zero_jubjub::{Fp, JubJubAffine, JubJubExtended, GENERATOR_EXTENDED};

/// Number encrypted by ElGamal encryption
#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Deserialize, Serialize)]
pub struct EncryptedNumber {
    s: JubJubAffine,
    t: JubJubAffine,
}

impl Default for EncryptedNumber {
    fn default() -> Self {
        Self {
            s: JubJubAffine::identity(),
            t: JubJubAffine::identity(),
        }
    }
}

impl EncryptedNumber {
    /// Init encrypted number
    pub fn new(s: JubJubAffine, t: JubJubAffine) -> Self {
        Self { s, t }
    }

    /// Enctypt number by private key
    pub fn encrypt(private_key: Fp, value: u32, random: Fp) -> Self {
        let g = GENERATOR_EXTENDED;
        let public_key = g * private_key;
        let left = g * Fp::from(value as u64) + public_key * random;
        EncryptedNumber {
            s: JubJubAffine::from(left),
            t: JubJubAffine::from(g * random),
        }
    }

    /// Decrypt encrypted number by brute force
    pub fn decrypt(&self, private_key: Fp) -> Option<u32> {
        let g = GENERATOR_EXTENDED;
        let decrypted_message =
            JubJubExtended::from(self.s) - (JubJubExtended::from(self.t) * private_key);

        let mut acc = JubJubExtended::identity();
        for i in 0..150000 {
            if acc == decrypted_message {
                return Some(i);
            }
            acc += g;
        }
        None
    }

    /// Get left and right affine point
    pub fn get_coordinate(self) -> (JubJubAffine, JubJubAffine) {
        (self.s, self.t)
    }
}

impl Add for EncryptedNumber {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            s: JubJubAffine::from(JubJubExtended::from(self.s) + JubJubExtended::from(rhs.s)),
            t: JubJubAffine::from(JubJubExtended::from(self.t) + JubJubExtended::from(rhs.t)),
        }
    }
}

impl Sub for EncryptedNumber {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            s: JubJubAffine::from(JubJubExtended::from(self.s) - JubJubExtended::from(rhs.s)),
            t: JubJubAffine::from(JubJubExtended::from(self.t) - JubJubExtended::from(rhs.t)),
        }
    }
}

impl CheckedAdd for EncryptedNumber {
    #[inline]
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        Some(Self {
            s: JubJubAffine::from(JubJubExtended::from(self.s) + JubJubExtended::from(rhs.s)),
            t: JubJubAffine::from(JubJubExtended::from(self.t) + JubJubExtended::from(rhs.t)),
        })
    }
}

impl CheckedSub for EncryptedNumber {
    #[inline]
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        Some(Self {
            s: JubJubAffine::from(JubJubExtended::from(self.s) - JubJubExtended::from(rhs.s)),
            t: JubJubAffine::from(JubJubExtended::from(self.t) - JubJubExtended::from(rhs.t)),
        })
    }
}

/// interface for circuit public inputs
pub trait TransferAmountPublic {
    /// get s and t cypher text
    fn get(self) -> (JubJubAffine, JubJubAffine);
}

impl TransferAmountPublic for EncryptedNumber {
    fn get(self) -> (JubJubAffine, JubJubAffine) {
        self.get_coordinate()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;
    use zero_crypto::behave::*;
    use zero_jubjub::Fp;

    use crate::EncryptedNumber;

    prop_compose! {
        fn arb_fr()(bytes in [any::<u8>(); 16]) -> Fp {
            Fp::random(XorShiftRng::from_seed(bytes))
        }
    }
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(25))]
        #[test]
        fn test_encrypt_decrypt(priv_k in arb_fr(), random in arb_fr(), balance in any::<u16>()) {
            let enc_balance = EncryptedNumber::encrypt(priv_k, balance as u32, random);

            let dec_balance = enc_balance.decrypt(priv_k);
            assert_eq!(dec_balance.unwrap(), balance as u32);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(25))]
        #[test]
        fn test_homomorphic(
            priv_k in arb_fr(), random1 in arb_fr(), random2 in arb_fr(),
            balance1 in any::<u16>(), balance2 in any::<u16>()
        ) {
            let (balance1, balance2) = if balance1 > balance2 {
                (balance1 as u32, balance2 as u32)
            } else {
                (balance2 as u32, balance1 as u32)
            };

            let enc_balance1 = EncryptedNumber::encrypt(priv_k, balance1, random1);
            let enc_balance2 = EncryptedNumber::encrypt(priv_k, balance2, random2);
            let enc_sub = enc_balance1 - enc_balance2;
            let enc_add = enc_balance1 + enc_balance2;

            let dec_sub = enc_sub.decrypt(priv_k);
            let dec_add = enc_add.decrypt(priv_k);

            assert_eq!(dec_sub.unwrap(), balance1 - balance2);
            assert_eq!(dec_add.unwrap(), balance1 + balance2);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn test_elgamal(
            alice_pk in arb_fr(), bob_pk in arb_fr(), alice_balance in 15..u16::MAX, bob_balance in 10..u16::MAX,
            transfer_amount in 10..u16::MAX, alice_randomness in 10..u64::MAX, bob_randomness in 10..u64::MAX,
            alice_transfer_randomness in 10..u64::MAX
        ) {
            let (alice_balance, transfer_amount) = if alice_balance > transfer_amount {
                (alice_balance as u32, transfer_amount as u32)
            } else {
                (transfer_amount as u32, alice_balance as u32)
            };
            let bob_balance = bob_balance as u32;

            // TODO
            let (alice_randomness, alice_transfer_randomness) = if alice_randomness > alice_transfer_randomness {
                (alice_randomness, alice_transfer_randomness)
            } else {
                (alice_transfer_randomness, alice_randomness)
            };
            let alice_randomness = Fp::from(alice_randomness);
            let bob_randomness = Fp::from(bob_randomness);
            let alice_transfer_randomness = Fp::from(alice_transfer_randomness);

            let alice_balance_enc = EncryptedNumber::encrypt(alice_pk, alice_balance, alice_randomness);
            let bob_balance_enc = EncryptedNumber::encrypt(bob_pk, bob_balance, bob_randomness);

            let transfer_amount_enc_alice =
                EncryptedNumber::encrypt(alice_pk, transfer_amount, alice_transfer_randomness);
            let transfer_amount_enc_bob =
                EncryptedNumber::encrypt(bob_pk, transfer_amount, alice_transfer_randomness);

            let alice_after_balance_enc = alice_balance_enc - transfer_amount_enc_alice;
            let bob_after_balance_enc = bob_balance_enc + transfer_amount_enc_bob;

            let alice_randomness_sum = alice_randomness - alice_transfer_randomness;
            let bob_randomness_sum = bob_randomness + alice_transfer_randomness;

            let explicit_alice = alice_balance - transfer_amount;
            let explicit_bob = bob_balance + transfer_amount;
            let exp_alice_balance_enc =
                EncryptedNumber::encrypt(alice_pk, explicit_alice, alice_randomness_sum);
            let exp_bob_balance_enc =
                EncryptedNumber::encrypt(bob_pk, explicit_bob, bob_randomness_sum);

            assert_eq!(exp_alice_balance_enc.t, alice_after_balance_enc.t);
            assert_eq!(exp_bob_balance_enc, bob_after_balance_enc);
        }
    }
}
