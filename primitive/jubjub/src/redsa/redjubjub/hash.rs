use blake2b_simd::{Params, State};
use zero_bls12_381::Fr;

pub(crate) fn hash_to_scalar(a: &[u8], b: &[u8]) -> Fr {
    SaplingHash::default().update(a).update(b).finalize()
}

struct SaplingHash(State);

impl Default for SaplingHash {
    fn default() -> Self {
        let state = Params::new()
            .hash_length(64)
            .personal(b"FROST_RedJubjubM")
            .to_state();

        Self(state)
    }
}

impl SaplingHash {
    pub(crate) fn update(&mut self, bytes: &[u8]) -> &mut Self {
        self.0.update(bytes);
        self
    }

    pub(crate) fn finalize(&self) -> Fr {
        let digest = self.0.finalize();
        Fr::from_hash(digest.as_ref())
    }
}
