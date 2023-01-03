#[macro_export]
macro_rules! bls12_g2_pairing {
    ($g2_projective:ident, $g2_affine:ident, $pairng_coeff:ident, $g2_pairing_affine:ident, $range_field:ident) => {
        use zero_crypto::behave::{G2Pairing, PairingRange, ParityCmp};

        impl ParityCmp for $pairng_coeff {}
        impl ParityCmp for $g2_pairing_affine {}

        impl G2Pairing for $g2_projective {
            type PairingRange = $range_field;
            type PairingCoeff = $pairng_coeff;
            type PairingRepr = $g2_pairing_affine;
            type G2Affine = $g2_affine;

            fn double_eval(&mut self) -> $pairng_coeff {
                // Adaptation of Algorithm 26, https://eprint.iacr.org/2010/354.pdf
                let tmp0 = self.x.square();
                let tmp1 = self.y.square();
                let tmp2 = tmp1.square();
                let tmp3 = (tmp1 + self.x).square() - tmp0 - tmp2;
                let tmp3 = tmp3 + tmp3;
                let tmp4 = tmp0 + tmp0 + tmp0;
                let tmp6 = self.x + tmp4;
                let tmp5 = tmp4.square();
                let zsquared = self.z.square();
                self.x = tmp5 - tmp3 - tmp3;
                self.z = (self.z + self.y).square() - tmp1 - zsquared;
                self.y = (tmp3 - self.x) * tmp4;
                let tmp2 = tmp2 + tmp2;
                let tmp2 = tmp2 + tmp2;
                let tmp2 = tmp2 + tmp2;
                self.y -= tmp2;
                let tmp3 = tmp4 * zsquared;
                let tmp3 = tmp3 + tmp3;
                let tmp3 = -tmp3;
                let tmp6 = tmp6.square() - tmp0 - tmp5;
                let tmp1 = tmp1 + tmp1;
                let tmp1 = tmp1 + tmp1;
                let tmp6 = tmp6 - tmp1;
                let tmp0 = self.z * zsquared;
                let tmp0 = tmp0 + tmp0;

                $pairng_coeff(tmp0, tmp3, tmp6)
            }

            fn add_eval(&mut self, rhs: $g2_affine) -> $pairng_coeff {
                // Adaptation of Algorithm 27, https://eprint.iacr.org/2010/354.pdf
                let zsquared = self.z.square();
                let ysquared = rhs.y.square();
                let t0 = zsquared * rhs.x;
                let t1 = ((rhs.y + self.z).square() - ysquared - zsquared) * zsquared;
                let t2 = t0 - self.x;
                let t3 = t2.square();
                let t4 = t3 + t3;
                let t4 = t4 + t4;
                let t5 = t4 * t2;
                let t6 = t1 - self.y - self.y;
                let t9 = t6 * rhs.x;
                let t7 = t4 * self.x;
                self.x = t6.square() - t5 - t7 - t7;
                self.z = (self.z + t2).square() - zsquared - t3;
                let t10 = rhs.y + self.z;
                let t8 = (t7 - self.x) * t6;
                let t0 = self.y * t5;
                let t0 = t0 + t0;
                self.y = t8 - t0;
                let t10 = t10.square() - ysquared;
                let ztsquared = self.z.square();
                let t10 = t10 - ztsquared;
                let t9 = t9 + t9 - t10;
                let t10 = self.z + self.z;
                let t6 = -t6;
                let t1 = t6 + t6;

                $pairng_coeff(t10, t1, t9)
            }
        }
    };
}

pub use bls12_g2_pairing;
