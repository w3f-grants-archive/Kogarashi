use crate::Fp;
use serde::{Deserialize, Serialize};
use zero_bls12_381::Fr;
use zero_crypto::arithmetic::edwards::*;
use zero_crypto::common::*;
use zero_crypto::dress::curve::edwards::*;

pub const EDWARDS_D: Fr = Fr::to_mont_form([
    0x01065fd6d6343eb1,
    0x292d7f6d37579d26,
    0xf5fd9207e6bd7fd4,
    0x2a9318e74bfa2b48,
]);

const X: Fr = Fr::to_mont_form([
    0x4df7b7ffec7beaca,
    0x2e3ebb21fd6c54ed,
    0xf1fbf02d0fd6cce6,
    0x3fd2814c43ac65a6,
]);

const Y: Fr = Fr::to_mont_form([
    0x0000000000000012,
    000000000000000000,
    000000000000000000,
    000000000000000000,
]);

const T: Fr = Fr::to_mont_form([
    0x07b6af007a0b6822b,
    0x04ebe6448d1acbcb8,
    0x036ae4ae2c669cfff,
    0x0697235704b95be33,
]);

#[derive(Clone, Copy, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct JubjubAffine {
    x: Fr,
    y: Fr,
}

impl SigUtils for JubjubAffine {
    const LENGTH: usize = 32;

    fn to_bytes(self) -> [u8; Self::LENGTH] {
        let mut tmp = self.x.to_bytes();
        let u = self.y.to_bytes();
        tmp[31] |= u[0] << 7;

        tmp
    }

    fn from_bytes(mut bytes: [u8; Self::LENGTH]) -> Option<Self> {
        let sign = (bytes[31] >> 7) == 1;
        bytes[31] &= 0b01111111;
        match Fr::from_bytes(bytes) {
            Some(y) => {
                let y2 = y.square();
                let yd = y2 * EDWARDS_D + Fr::one();
                let y2 = y2 - Fr::one();
                match yd.invert() {
                    Some(inv) => {
                        let y2 = y2 * inv;

                        match y2.sqrt() {
                            Some(mut x) => {
                                if x.is_odd() != sign {
                                    x = -x;
                                }
                                Some(Self { x, y })
                            }
                            None => None,
                        }
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}

impl JubjubAffine {
    pub const fn from_raw_unchecked(x: Fr, y: Fr) -> JubjubAffine {
        JubjubAffine { x, y }
    }
}

impl Add for JubjubAffine {
    type Output = JubjubExtended;

    fn add(self, rhs: JubjubAffine) -> Self::Output {
        add_point(self.to_extended(), rhs.to_extended())
    }
}

impl Neg for JubjubAffine {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
        }
    }
}

impl Sub for JubjubAffine {
    type Output = JubjubExtended;

    fn sub(self, rhs: JubjubAffine) -> Self::Output {
        add_point(self.to_extended(), rhs.neg().to_extended())
    }
}

impl Mul<Fr> for JubjubAffine {
    type Output = JubjubExtended;

    fn mul(self, rhs: Fr) -> Self::Output {
        scalar_point(self.to_extended(), &rhs)
    }
}

impl Mul<JubjubAffine> for Fr {
    type Output = JubjubExtended;

    fn mul(self, rhs: JubjubAffine) -> Self::Output {
        scalar_point(rhs.to_extended(), &self)
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct JubjubExtended {
    x: Fr,
    y: Fr,
    t: Fr,
    z: Fr,
}

impl JubjubExtended {
    pub fn batch_normalize<'a>(
        y: &'a mut [JubjubExtended],
    ) -> impl Iterator<Item = JubjubAffine> + 'a {
        y.iter().map(|p| JubjubAffine::from(*p))
    }
}

impl Add for JubjubExtended {
    type Output = JubjubExtended;

    fn add(self, rhs: JubjubExtended) -> Self::Output {
        add_point(self, rhs)
    }
}

impl Neg for JubjubExtended {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
            t: -self.t,
            z: self.z,
        }
    }
}

impl SigUtils for JubjubExtended {
    const LENGTH: usize = 32;

    fn to_bytes(self) -> [u8; Self::LENGTH] {
        self.to_affine().to_bytes()
    }

    fn from_bytes(bytes: [u8; Self::LENGTH]) -> Option<Self> {
        match JubjubAffine::from_bytes(bytes) {
            Some(point) => Some(point.to_extended()),
            None => None,
        }
    }
}

impl Sub for JubjubExtended {
    type Output = JubjubExtended;

    fn sub(self, rhs: JubjubExtended) -> Self::Output {
        add_point(self, rhs.neg())
    }
}

impl Mul<Fr> for JubjubExtended {
    type Output = JubjubExtended;

    fn mul(self, rhs: Fr) -> Self::Output {
        scalar_point(self, &rhs)
    }
}

impl Mul<JubjubExtended> for Fr {
    type Output = JubjubExtended;

    fn mul(self, rhs: JubjubExtended) -> Self::Output {
        scalar_point(rhs, &self)
    }
}

twisted_edwards_curve_operation!(Fr, Fr, EDWARDS_D, JubjubAffine, JubjubExtended, X, Y, T);

impl Mul<Fp> for JubjubExtended {
    type Output = JubjubExtended;

    #[inline]
    fn mul(self, rhs: Fp) -> JubjubExtended {
        &self * &rhs
    }
}

impl<'a, 'b> Mul<&'b Fp> for &'a JubjubExtended {
    type Output = JubjubExtended;

    #[inline]
    fn mul(self, rhs: &'b Fp) -> JubjubExtended {
        let mut res = JubjubExtended::ADDITIVE_IDENTITY;
        let mut acc = *self;
        for &naf in rhs.to_nafs().iter() {
            if naf == Naf::Plus {
                res += acc;
            } else if naf == Naf::Minus {
                res -= acc;
            }
            acc = acc.double();
        }
        res
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use zero_crypto::dress::curve::weierstrass::*;

    curve_test!(jubjub, Fr, JubjubAffine, JubjubExtended, 100);

    #[test]
    fn serde_test() {
        let s = Fr::random(OsRng);
        let point = s * JubjubAffine::ADDITIVE_GENERATOR;
        let bytes = point.to_bytes();
        let point_p = JubjubAffine::from_bytes(bytes).unwrap();

        assert_eq!(point.to_affine(), point_p)
    }
}
