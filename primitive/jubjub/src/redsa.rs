use crate::curve::JubjubExtend;
use zero_crypto::behave::*;

pub struct PublicKey(JubjubExtend);

impl DigitalSig for PublicKey {
    const LENGTH: usize = 32;
}

impl PublicKey {
    pub fn new(raw: JubjubExtend) -> Self {
        PublicKey(raw)
    }

    pub fn as_bytes(self) -> [u8; Self::LENGTH] {
        self.0.as_bytes()
    }
}
