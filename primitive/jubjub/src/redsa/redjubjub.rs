mod hash;
mod private_key;
mod public_key;
mod signature;

pub use private_key::SecretKey;
pub use public_key::PublicKey;

#[cfg(test)]
mod tests {
    use super::private_key::SecretKey;
    use rand_core::OsRng;
    use zero_bls12_381::Fr;
    use zkstd::behave::Group;

    #[ignore]
    #[test]
    fn signature_test() {
        for _ in 0..1000 {
            let msg = b"test";
            let randomness = OsRng;
            let priv_key = SecretKey(Fr::random(OsRng));
            let pub_key = priv_key.to_public_key();

            let sig = priv_key.sign(msg, randomness);

            assert!(pub_key.validate(msg, sig))
        }
    }
}
