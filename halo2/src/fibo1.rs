use std::marker::PhantomData;

use group::ff::Field;
use halo2_proofs::{
    circuit::*,
    plonk::*, poly::Rotation, pasta::Fp, dev::MockProver,
};

#[derive(Debug, Clone)]
struct ACell<F: Field>(AssignedCell<F,F>);

#[derive(Debug, Clone)]
struct FiboConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>,
}

struct FiboChip<F: Field> {
    config: FiboConfig,
    _marker: PhantomData<F>,
}

impl<F: Field> FiboChip<F> {
    fn construct(config: FiboConfig) -> Self {
        Self { config, _marker: PhantomData }
    }

    fn configure(meta: &mut ConstraintSystem<F>, advice: [Column<Advice>; 3], instance: Column<Instance>) -> FiboConfig {
        // We receive the advices as args so we can reuse them
        let [col_a, col_b, col_c] = advice;

        // Selectors do get optimized by the backend, so no need to receive them as args
        let selector: Selector = meta.selector();

        // Needed to use permutation argument
        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        // Create a custom gate for addition. There are no pre-built gates in Halo2.
        meta.create_gate("add", |meta| {
            let s = meta.query_selector(selector);
            
            // Rotation lets us pick the current row, or a row given an offset
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            
            // If selector s is set, then a+b=c
            vec![s * (a + b - c)]
        });

        FiboConfig { advice: [col_a,col_b,col_c], selector, instance }
    }
    
    fn assign_first_row(&self, mut layouter: impl Layouter<F>, a: Value<F>, b: Value<F>) -> Result<(ACell<F>, ACell<F>, ACell<F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                // Enables the selector on the first row of this region
                self.config.selector.enable(&mut region, 0)?;
                
                let a_cell = region.assign_advice(
                    || "a", 
                    self.config.advice[0], 
                    0, 
                    || a
                ).map(ACell)?;

                let b_cell = region.assign_advice(
                    || "b", 
                    self.config.advice[1], 
                    0, 
                    || b
                ).map(ACell)?;

                // Assign a+b to the c cell
                let c_val = a.and_then(|a| b.map(|b| a + b));

                let c_cell = region.assign_advice(
                    || "c", 
                    self.config.advice[2],
                    0, 
                    || c_val
                ).map(ACell)?;

                Ok((a_cell, b_cell, c_cell))
            }
        )
    }

    fn assign_row(&self, mut layouter: impl Layouter<F>, prev_b: &ACell<F>, prev_c: &ACell<F>) -> Result<ACell<F>, Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                
                // Copy b to a in the new region
                prev_b.0.copy_advice(|| "a", &mut region, self.config.advice[0], 0)?;

                // Copy c to b in the new region
                prev_c.0.copy_advice(|| "b", &mut region, self.config.advice[1], 0)?;

                // Calculate new c value as a+b
                let c_val = prev_b.0.value().and_then(|b| {
                    prev_c.0.value().map(|c| *b + *c)
                });

                let c_cell = region.assign_advice(
                    || "c", 
                    self.config.advice[2], 
                    0, 
                    || c_val
                ).map(ACell)?;

                Ok(c_cell)
            }
        )
    }

    pub fn expose_public(&self, mut layouter: impl Layouter<F>, cell: &ACell<F>, row: usize) -> Result<(), Error> {
        layouter.constrain_instance(cell.0.cell(), self.config.instance, row)
    }

}

#[derive(Default)]
struct MyCircuit<F> {
    pub a: Value<F>,
    pub b: Value<F>,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let instance = meta.instance_column();

        FiboChip::configure(meta, [col_a, col_b, col_c], instance)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = FiboChip::construct(config);
        let (prev_a, mut prev_b, mut prev_c) = chip.assign_first_row(
            layouter.namespace(|| "first_row"),
            self.a, self.b
        )?;

        chip.expose_public(layouter.namespace(|| "private a"), &prev_a, 0)?;
        chip.expose_public(layouter.namespace(|| "private b"), &prev_b, 1)?;

        for _i in 3..10{
            let c_cell = chip.assign_row(
                layouter.namespace(|| "next row"),
                &prev_b,
                &prev_c,
            )?;
            prev_b = prev_c;
            prev_c = c_cell;
        }

        chip.expose_public(layouter.namespace(|| "out"), &prev_c, 2)?;

        Ok(())
    }
}
 
fn main() {
    let k = 4;
    let a = Fp::from(1);
    let b = Fp::from(1);
    let out = Fp::from(55);

    let circuit = MyCircuit {
        a: Value::known(a), b: Value::known(b),
    };

    let public_input = vec![a,b,out];

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();

    // Draw circuit!
    use plotters::prelude::*;
    let root = BitMapBackend::new("img/fibo1-layout.png", (1024, 3096)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled("Fibo1 Layout", ("sans-serif", 60)).unwrap();
    halo2_proofs::dev::CircuitLayout::default()
        .render(4, &circuit, &root)
        .unwrap();
}
