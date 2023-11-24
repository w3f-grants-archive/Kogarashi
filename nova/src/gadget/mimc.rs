use crate::hash::Mimc;
use r1cs::gadget::FieldAssignment;
use r1cs::{CircuitDriver, R1cs};

pub(crate) struct MimcAssignment<const ROUND: usize, C: CircuitDriver> {
    constants: [C::Scalar; ROUND],
}

impl<const ROUND: usize, C: CircuitDriver> Default for MimcAssignment<ROUND, C> {
    fn default() -> Self {
        Self {
            constants: Mimc::<ROUND, C::Scalar>::default().constants,
        }
    }
}

impl<const ROUND: usize, C: CircuitDriver> MimcAssignment<ROUND, C> {
    pub(crate) fn hash(
        &self,
        cs: &mut R1cs<C>,
        mut xl: FieldAssignment<C>,
        mut xr: FieldAssignment<C>,
    ) -> FieldAssignment<C> {
        for c in self.constants.iter().map(|c| FieldAssignment::constant(c)) {
            let cxl = &xl + &c;
            let mut ccxl = FieldAssignment::square(cs, &cxl);
            ccxl = &FieldAssignment::mul(cs, &ccxl, &cxl) + &xr;
            xr = xl;
            xl = ccxl;
        }
        xl
    }
}
