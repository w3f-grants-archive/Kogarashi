use core::ops::{Add, Mul, Sub};

use core::iter;
use rand_core::RngCore;
use zero_crypto::behave::FftField;
use zero_crypto::common::Vec;

// a_n-1 , a_n-2, ... , a_0
#[derive(Debug, Clone, PartialEq)]
pub struct Polynomial<F>(pub Vec<F>);

pub struct Witness<F> {
    s_eval: F,
    a_eval: F,
    q_eval: F,
    denominator: F,
}

impl<F: FftField> Polynomial<F> {
    pub fn new(coeffs: Vec<F>) -> Self {
        Self(coeffs)
    }

    // polynomial evaluation domain
    // r^0, r^1, r^2, ..., r^n
    pub fn setup(k: usize, rng: impl RngCore) -> (F, Vec<F>) {
        let randomness = F::random(rng);
        (
            randomness,
            (0..(1 << k))
                .scan(F::one(), |w, _| {
                    let tw = *w;
                    *w *= randomness;
                    Some(tw)
                })
                .collect::<Vec<_>>(),
        )
    }

    // commit polynomial to domain
    pub fn commit(&self, domain: &Vec<F>) -> F {
        assert!(self.0.len() <= domain.len());
        let diff = domain.len() - self.0.len();

        self.0
            .iter()
            .zip(domain.iter().skip(diff))
            .fold(F::zero(), |acc, (a, b)| acc + *a * *b)
    }

    // evaluate polynomial at
    pub fn evaluate(&self, at: &F) -> F {
        self.0
            .iter()
            .rev()
            .fold(F::zero(), |acc, coeff| acc * at + coeff)
    }

    // no remainder polynomial division with at
    // f(x) - f(at) / x - at
    pub fn divide(&self, at: &F) -> Self {
        let mut coeffs = self
            .0
            .iter()
            .rev()
            .scan(F::zero(), |w, coeff| {
                let tmp = *w + coeff;
                *w = tmp * at;
                Some(tmp)
            })
            .collect::<Vec<_>>();
        coeffs.pop();
        coeffs.reverse();
        Self(coeffs)
    }

    /// σ^n - 1
    pub fn t(n: u64, tau: F) -> F {
        tau.pow(n) - F::one()
    }

    fn format_degree(mut self) -> Self {
        while self.0.last().map_or(false, |c| c == &F::zero()) {
            self.0.pop();
        }
        self
    }

    // create witness for f(a)
    pub fn create_witness(self, at: &F, s: &F, domain: Vec<F>) -> Witness<F> {
        // p(x) - p(at) / x - at
        let quotient = self.divide(at);
        // p(s)
        let s_eval = self.commit(&domain);
        // p(at)
        let a_eval = self.evaluate(at);
        // p(s) - p(at) / s - at
        let q_eval = quotient.evaluate(s);
        // s - at
        let denominator = *s - *at;

        Witness {
            s_eval,
            a_eval,
            q_eval,
            denominator,
        }
    }
}

impl<F: FftField> Add for Polynomial<F> {
    type Output = Polynomial<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let zero = F::zero();
        let (left, right) = if self.0.len() > rhs.0.len() {
            (self.0.iter(), rhs.0.iter().chain(iter::repeat(&zero)))
        } else {
            (rhs.0.iter(), self.0.iter().chain(iter::repeat(&zero)))
        };
        Self(left.zip(right).map(|(a, b)| *a + *b).collect()).format_degree()
    }
}

impl<F: FftField> Sub for Polynomial<F> {
    type Output = Polynomial<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        let zero = F::zero();
        let (left, right) = if self.0.len() > rhs.0.len() {
            (self.0.iter(), rhs.0.iter().chain(iter::repeat(&zero)))
        } else {
            (rhs.0.iter(), self.0.iter().chain(iter::repeat(&zero)))
        };
        Self(left.zip(right).map(|(a, b)| *a - *b).collect()).format_degree()
    }
}

impl<F: FftField> Mul<F> for Polynomial<F> {
    type Output = Polynomial<F>;

    fn mul(self, scalar: F) -> Polynomial<F> {
        Self(self.0.into_iter().map(|coeff| coeff * &scalar).collect())
    }
}

impl<F: FftField> Witness<F> {
    // verify witness
    pub fn verify_eval(self) -> bool {
        self.q_eval * self.denominator == self.s_eval - self.a_eval
    }
}

#[cfg(test)]
mod tests {
    use super::Polynomial;
    use rand_core::OsRng;
    use zero_bls12_381::Fr;
    use zero_crypto::behave::{Group, PrimeField};

    fn arb_fr() -> Fr {
        Fr::random(OsRng)
    }

    fn arb_poly(k: u32) -> Polynomial<Fr> {
        Polynomial(
            (0..(1 << k))
                .map(|_| Fr::random(OsRng))
                .collect::<Vec<Fr>>(),
        )
    }

    fn naive_multiply<F: PrimeField>(a: Vec<F>, b: Vec<F>) -> Vec<F> {
        let mut c = vec![F::zero(); a.len() + b.len() - 1];
        a.iter().enumerate().for_each(|(i_a, coeff_a)| {
            b.iter().enumerate().for_each(|(i_b, coeff_b)| {
                c[i_a + i_b] += *coeff_a * *coeff_b;
            })
        });
        c
    }

    #[test]
    fn polynomial_scalar() {
        let poly = arb_poly(10);
        let at = arb_fr();
        let scalared = poly.clone() * at;
        let test = Polynomial(poly.0.into_iter().map(|coeff| coeff * at).collect());
        assert_eq!(scalared, test);
    }

    #[test]
    fn polynomial_division_test() {
        let at = arb_fr();
        let divisor = arb_poly(10);
        // dividend = divisor * quotient
        let factor_poly = vec![Fr::one(), -at];

        // divisor * (x - at) = dividend
        let poly_a = Polynomial(naive_multiply(divisor.0, factor_poly.clone()));

        // dividend / (x - at) = quotient
        let quotient = poly_a.divide(&at);

        // quotient * (x - at) = divident
        let original = Polynomial(naive_multiply(quotient.0, factor_poly));

        assert_eq!(poly_a.0, original.0);
    }
}
