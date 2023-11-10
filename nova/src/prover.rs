use crate::{pedersen::PedersenCommitment, relaxed_r1cs::RelaxedR1cs};

use r1cs::{CircuitDriver, DenseVectors, R1cs};
use zkstd::common::Ring;

pub struct Prover<C: CircuitDriver> {
    // public parameters
    pp: PedersenCommitment<C::Affine>,

    // r1cs structure
    f: R1cs<C>,
}

impl<C: CircuitDriver> Prover<C> {
    pub fn prove(&self, r1cs: R1cs<C>, relaxed_r1cs: RelaxedR1cs<C>) -> RelaxedR1cs<C> {
        // compute cross term t
        let t = self.compute_cross_term(&r1cs, &relaxed_r1cs);

        // TODO: replace with transcript
        let lc_random = C::Scalar::one();
        let commit_t = self.pp.commit(&t, &lc_random);

        // fold instance
        let instance = relaxed_r1cs.fold_instance(&r1cs, lc_random, commit_t);

        // fold witness
        let witness = relaxed_r1cs.fold_witness(r1cs, lc_random, t);

        // return folded relaxed r1cs
        relaxed_r1cs.update(instance, witness)
    }

    // T = AZ1 ◦ BZ2 + AZ2 ◦ BZ1 − u1 · CZ2 − u2 · CZ1
    fn compute_cross_term(
        &self,
        r1cs: &R1cs<C>,
        relaxed_r1cs: &RelaxedR1cs<C>,
    ) -> DenseVectors<C::Scalar> {
        let u1 = C::Scalar::one();
        let u2 = relaxed_r1cs.u();
        let m = self.f.m();
        let (a, b, c) = self.f.matrices();
        let (w0, w1) = (DenseVectors::new(r1cs.w()), relaxed_r1cs.w());
        let (x0, x1) = (DenseVectors::new(r1cs.x()), relaxed_r1cs.x());

        // matrices and z vector matrix multiplication
        let az2 = a.prod(&m, &x1, &w1);
        let bz1 = b.prod(&m, &x0, &w0);
        let az1 = a.prod(&m, &x0, &w0);
        let bz2 = b.prod(&m, &x1, &w1);
        let cz2 = c.prod(&m, &x1, &w1);
        let cz1 = c.prod(&m, &x0, &w0);

        // matrices Hadamard product
        let az2bz1 = az2 * bz1;
        let az1bz2 = az1 * bz2;

        // vector scalar mutltiplication
        let c1cz2 = cz2 * u1;
        let c2cz1 = cz1 * u2;

        // vector addition and subtraction
        az2bz1 + az1bz2 - c1cz2 - c2cz1
    }
}
