//! This module extends GadgetBuilder with native field arithmetic methods.

use crate::r1cs::constraint_system::ConstraintSystem;
use crate::r1cs::expression::Expression;
use zkstd::common::Field;

impl<F: Field> ConstraintSystem<F> {
    /// The product of two `Expression`s `x` and `y`, i.e. `x * y`.
    pub fn product(&mut self, x: &Expression<F>, y: &Expression<F>) -> Expression<F> {
        if let Some(c) = x.as_constant() {
            return y * c;
        }
        if let Some(c) = y.as_constant() {
            return x * c;
        }

        let product_value = x.evaluate(&self.wire_values) * y.evaluate(&self.wire_values);
        let product = self.alloc_public(product_value);
        let product_exp = Expression::from(product);
        self.assert_product(x, y, &product_exp);

        product_exp
    }

    /// Returns `1 / x`, assuming `x` is non-zero. If `x` is zero, the gadget will not be
    /// satisfiable.
    pub fn inverse(&mut self, x: &Expression<F>) -> Expression<F> {
        let x_value = x.evaluate(&self.wire_values);
        let inverse_value = x_value.invert().expect("Can't find an inverse element");
        let x_inv = self.alloc_public(inverse_value);

        let x_inv_expression = Expression::from(x_inv);
        self.assert_product(x, &x_inv_expression, &Expression::one());

        x_inv_expression
    }
}
