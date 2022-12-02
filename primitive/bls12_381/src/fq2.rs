use crate::fq::Fq;
use zero_crypto::dress::extention_field::*;

// sextic twist of Fp12
#[derive(Debug, Clone, Copy, Decode, Encode)]
pub struct Fq2(pub(crate) [Fq; 2]);

const ZERO: Fq2 = Fq2([Fq::zero(); 2]);

const ONE: [Fq; 2] = [Fq::one(), Fq::zero()];

const LIMBS_LENGTH: usize = 2;

extention_field_operation!(Fq2, Fq, ZERO, ONE, LIMBS_LENGTH);

#[cfg(test)]
mod tests {
    use super::Fq2;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;
    use zero_crypto::behave::Group;
    use zero_crypto::common::PrimeField;

    prop_compose! {
        fn arb_jubjub_fq()(bytes in [any::<u8>(); 16]) -> Fq2 {
            Fq2::random(XorShiftRng::from_seed(bytes))
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100000))]
        #[test]
        fn fq2_add_test(a in arb_jubjub_fq()) {
            // a + a = a * 2
            let b = a + a;
            let c = a.double();
            assert_eq!(b, c);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100000))]
        #[test]
        fn fq2_sub_test(a in arb_jubjub_fq()) {
            // a - a = a * 2 - a * 2
            let b = a - a;
            let c = a.double();
            let d = a.double();
            let e = c - d;

            assert_eq!(b, e);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn fq2_mul_test(a in arb_jubjub_fq(), b in arb_jubjub_fq(), c in arb_jubjub_fq()) {
            // a * b + a * c
            let ab = a * b;
            let ac = a * c;
            let d = ab + ac;

            // a * (b + c)
            let bc = b + c;
            let e = a * bc;

            assert_eq!(d, e);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn fq2_square_test(a in arb_jubjub_fq(), b in arb_jubjub_fq()) {
            // (a * a) * (b * b)
            let aa = a * a;
            let bb = b * b;
            let c = aa * bb;

            // a^2 * b^2
            let aa = a.square();
            let bb = b.square();
            let d = aa * bb;

            assert_eq!(c, d);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn fq2_invert_test(a in arb_jubjub_fq()) {
            let inv = a.invert();

            match inv {
                Some(x) => {
                    let b = a * x;
                    assert_eq!(b, Fq2::one())
                },
                None => {}
            }
        }
    }
}
