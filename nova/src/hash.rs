mod circuit;
mod helper;

use helper::BlakeHelper;
use zkstd::common::{BNAffine, PrimeField};

pub(crate) struct Mimc<const ROUND: usize, F: PrimeField> {
    constants: [F; ROUND],
}

impl<const ROUND: usize, F: PrimeField> Default for Mimc<ROUND, F> {
    fn default() -> Self {
        let mut constants = [F::zero(); ROUND];
        let mut helper = BlakeHelper::default();
        for constant in constants.iter_mut() {
            let bytes = helper.get();
            helper.update(&bytes);
            *constant = helper.finalize()
        }

        Self { constants }
    }
}

impl<const ROUND: usize, F: PrimeField> Mimc<ROUND, F> {
    pub(crate) fn hash(&self, mut xl: F, mut xr: F) -> F {
        for c in self.constants {
            let mut cxl = xl;
            cxl += c;
            let mut ccxl = cxl.square();
            ccxl *= cxl;
            ccxl += xr;
            xr = xl;
            xl = ccxl;
        }
        xl
    }
}

pub(crate) struct MimcRO<const ROUND: usize, F: PrimeField> {
    hasher: Mimc<ROUND, F>,
    state: Vec<F>,
    key: F,
}

impl<const ROUND: usize, F: PrimeField> Default for MimcRO<ROUND, F> {
    fn default() -> Self {
        Self {
            hasher: Mimc::default(),
            state: Vec::default(),
            key: F::zero(),
        }
    }
}

impl<const ROUND: usize, F: PrimeField> MimcRO<ROUND, F> {
    pub(crate) fn append(&mut self, absorb: F) {
        self.state.push(absorb)
    }

    pub(crate) fn append_point<A: BNAffine<Base = F>>(&mut self, point: A) {
        self.append(point.get_x());
        self.append(point.get_y());
        self.append(if point.is_identity() {
            A::Base::zero()
        } else {
            A::Base::one()
        });
    }

    pub(crate) fn squeeze(&self) -> F {
        self.state.iter().fold(self.key, |acc, scalar| {
            let h = self.hasher.hash(*scalar, acc);
            acc + scalar + h
        })
    }
}
