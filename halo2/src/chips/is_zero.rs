// Code adapted from https://github.com/icemelon/halo2-examples/blob/6b8676487635169367e085a44e3b73c56290883f/src/is_zero.rs
// The original receives the value_inv column as a parameter, this one creates it within the chip configuration
// This version is updated to run with the latest (commit 7239a02ea34) version of halo2_proof

use group::ff::Field;
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

#[derive(Clone, Debug)]
pub struct IsZeroConfig<F> {
    pub value_inv: Column<Advice>,
    pub is_zero_expr: Expression<F>,
}

impl<F: Field> IsZeroConfig<F> {
    pub fn expr(&self) -> Expression<F> {
        self.is_zero_expr.clone()
    }
}

pub struct IsZeroChip<F: Field> {
    config: IsZeroConfig<F>,
}

impl<F: Field> IsZeroChip<F> {
    pub fn construct(config: IsZeroConfig<F>) -> Self {
        IsZeroChip { config }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        value: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>
    ) -> IsZeroConfig<F> {
        let value_inv = meta.advice_column();
        let mut is_zero_expr = Expression::Constant(F::ZERO);

        meta.create_gate("is_zero", |meta| {
            //
            // valid | value |  value_inv |  1 - value * value_inv | value * (1 - value* value_inv)
            // ------+-------+------------+------------------------+-------------------------------
            //  yes  |   x   |    1/x     |         0              |  0
            //  no   |   x   |    0       |         1              |  x
            //  yes  |   0   |    0       |         1              |  0
            //  yes  |   0   |    y       |         1              |  0
            //
            let value = value(meta);
            let q_enable = q_enable(meta);
            let value_inv = meta.query_advice(value_inv, Rotation::cur());

            is_zero_expr = Expression::Constant(F::ONE) - value.clone() * value_inv;
            vec![q_enable * value * is_zero_expr.clone()]
        });

        IsZeroConfig {
            value_inv,
            is_zero_expr,
        }
    }

    pub fn assign(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        value: Value<F>,
    ) -> Result<Value<bool>, Error> {
        let value_inv = value.map(|value| value.invert().unwrap_or(F::ZERO));
        region.assign_advice(|| "value inv", self.config.value_inv, offset, || value_inv)?;
        Ok(value.map(|value| value.is_zero_vartime()))
    }
}
