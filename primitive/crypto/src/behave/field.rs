// This trait resresents prime field

use super::{
    algebra::Field,
    comp::{Basic, ParityCmp},
};
use crate::arithmetic::utils::Bits;

/// This is prime field trait
pub trait PrimeField: Field + Basic + ParityCmp {
    // prime order of this field
    const MODULUS: Self;

    // mongomery reduction inverse
    const INV: u64;

    fn from_u64(val: u64) -> Self;

    fn to_bits(self) -> Bits;

    fn is_zero(self) -> bool;

    fn double(self) -> Self;

    fn square(self) -> Self;

    fn double_assign(&mut self);

    fn square_assign(&mut self);
}
