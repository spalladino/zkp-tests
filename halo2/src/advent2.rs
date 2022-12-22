mod chips;

use chips::is_zero::{IsZeroConfig, IsZeroChip};
use group::ff::PrimeField;
use halo2_proofs::{
    circuit::*,
    plonk::*, poly::Rotation, pasta::Fp, dev::MockProver,
};

fn const_val<F: PrimeField>(value: u64) -> Expression<F> {
    Expression::Constant(F::from(value))
}

#[derive(Debug, Clone)]
struct AdventConfig<F: PrimeField> {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>,
    pub is_zero_chips: [IsZeroConfig<F>; 2],
}

struct AdventChip<F: PrimeField, const N: usize> {
    config: AdventConfig<F>
}

impl<F: PrimeField, const N: usize> AdventChip<F, N> {
    fn construct(config: AdventConfig<F>) -> Self {
        Self { config }
    }

    fn configure(meta: &mut ConstraintSystem<F>, advice: [Column<Advice>; 3], instance: Column<Instance>, constant: Column<Fixed>) -> AdventConfig<F> {
        // We receive the advices as args so we can reuse them
        let [col_x, col_y, col_accum] = advice;

        // Selectors do get optimized by the backend, so no need to receive them as args
        let selector: Selector = meta.selector();

        // Enable equality so we can assign the last accum value to the public instance
        meta.enable_equality(col_accum);
        meta.enable_equality(instance);

        // QUESTION: Can we drop this? We just want to set the first accum to zero
        meta.enable_constant(constant);

        // yWins <==> (y+2-x) * (y-1-x) == 0;
        let y_wins = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(selector), 
            |meta| {
                let x = meta.query_advice(col_x, Rotation::cur());
                let y = meta.query_advice(col_y, Rotation::cur());
                (y.clone() + const_val(2) - x.clone()) * (y - const_val(1) - x)
            }
        );

        // isDraw <==> y-x == 0;
        let is_draw = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(selector), 
            |meta| {
                let x = meta.query_advice(col_x, Rotation::cur());
                let y = meta.query_advice(col_y, Rotation::cur());
                y - x
            }
        );

        // Create a custom gate for each round
        meta.create_gate("round", |meta| {
            let s = meta.query_selector(selector);
            
            let x = meta.query_advice(col_x, Rotation::cur());
            let y = meta.query_advice(col_y, Rotation::cur());
            let accum = meta.query_advice(col_accum, Rotation::cur());

            // We store the output in the accum column in the next row
            let out = meta.query_advice(col_accum, Rotation::next());

            // Constraints for each round
            // TODO: Review Rust borrowing rules to see if there's a way around cloning everything
            vec![
                // out = y_wins * 6 + is_draw * 3 + y + 1 + accum
                s.clone() * (out - (y_wins.expr() * F::from(6) + is_draw.expr() * F::from(3) + y.clone() + const_val(1) + accum)),
                // x in (0,1,2)
                s.clone() * x.clone() * (x.clone() - const_val(1)) * (x.clone() - const_val(2)),
                // y in (0,1,2)
                s.clone() * y.clone() * (y.clone() - const_val(1)) * (y.clone() - const_val(2)),
            ]
        });

        AdventConfig { advice: [col_x, col_y, col_accum], selector, instance, is_zero_chips: [y_wins, is_draw] }
    }
    
    fn assign(&self, mut layouter: impl Layouter<F>, xs: [Value<F>; N], ys: [Value<F>; N]) -> Result<AssignedCell<F,F>, Error> {
        // We assign the entire matrix in a single region so we can overlap round gadgets (see fibonacci/example2 in halo2-examples)
        // Otherwise, we'd have to copy_advice from one to the other (as we did in fibo1)
        layouter.assign_region(
            || "rps game", 
            |mut region| {
                let [col_x, col_y, col_accum] = self.config.advice;
                let [y_wins, is_draw] = &self.config.is_zero_chips;
                let mut accum_value: Value<F> = Value::known(F::ZERO);
                let mut out_cell = Err(Error::Synthesis);

                // Assign one row per round
                for row in 0..N {
                    let [x, y] = [xs[row], ys[row]]; 

                    // Enable the selector for the round gate
                    self.config.selector.enable(&mut region, row)?;

                    // QUESTION: Do we need to explicitly set this value to zero? 
                    // This is requiring us to add a constant column to the chip config just with zeros
                    if row == 0 {
                        region.assign_advice_from_constant(
                            || "zero", 
                            col_accum, 
                            0, 
                            F::ZERO
                        )?;
                    }

                    // Set x and y advice columns to the input values
                    region.assign_advice(
                        || format!("x[{}]", row),
                        col_x,
                        row,
                        || x
                    )?;

                    region.assign_advice(
                        || format!("y[{}]", row),
                        col_y,
                        row,
                        || y
                    )?;

                    // Assign the is_zero chips to the same expressions defined in the gates
                    // yWins <==> (y+2-x) * (y-1-x) == 0;
                    let y_wins_chip = IsZeroChip::construct(y_wins.clone());
                    let y_wins_value = (y + Value::known(F::from(2)) - x) * (y - Value::known(F::ONE) - x);
                    let y_wins = y_wins_chip.assign(&mut region, row, y_wins_value)?;

                    // isDraw <==> y-x == 0;
                    let is_draw_chip = IsZeroChip::construct(is_draw.clone());
                    let is_draw_value = y - x;
                    let is_draw = is_draw_chip.assign(&mut region, row, is_draw_value)?;

                    // Calculate the score of this round
                    let round_score = y_wins.zip(is_draw).and_then(|(y_wins, is_draw)| {
                        let partial_score = if y_wins { 6 } else if is_draw { 3 } else { 0 };
                        Value::known(F::from(partial_score)) + y + Value::known(F::ONE)
                    });

                    // Assign the col_accum *in the next row* to the new score
                    accum_value = accum_value + round_score;
                    out_cell = region.assign_advice(
                        || format!("out[{}]", row),
                        col_accum,
                        row + 1,
                        || accum_value
                    );
                };
                
                // Return the last cell in col_accum
                out_cell
            })
    }

    pub fn expose_public(&self, mut layouter: impl Layouter<F>, cell: &AssignedCell<F,F>, round: usize) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, round)
    }

}

struct AdventCircuit<F: PrimeField, const N: usize> {
    xs: [Value<F>; N],
    ys: [Value<F>; N],
}

impl<F: PrimeField, const N: usize> Circuit<F> for AdventCircuit<F, N> {
    type Config = AdventConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            xs: [Value::unknown(); N],
            ys: [Value::unknown(); N],
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_x = meta.advice_column();
        let col_y = meta.advice_column();
        let col_accum = meta.advice_column();
        let instance = meta.instance_column();
        let constant = meta.fixed_column();

        AdventChip::<F,N>::configure(meta, [col_x, col_y, col_accum], instance, constant)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = AdventChip::construct(config);
        
        // Assign returns the last row from col_accum that contains the total accumulated score, 
        // which we expose by matching it to the public instance that corresponds to round N-1
        let out_cell = chip.assign(layouter.namespace(|| "rps"), self.xs, self.ys)?;        
        chip.expose_public(layouter.namespace(|| "out"), &out_cell, N-1)?;
        
        Ok(())
    }
}

// Returns the circuit configured with the private inputs, and the public inputs
fn make_circuit<const N: usize>(xs: [u64; N], ys: [u64; N], out: u64) -> (AdventCircuit<Fp, N>, Vec<Vec<Fp>>) {
    // The plays in each round are the private inputs to the circuit
    let circuit = AdventCircuit {
        xs: xs.map(|val| Value::known(Fp::from(val))),
        ys: ys.map(|val| Value::known(Fp::from(val))),
    };

    // We can fill the public instances with zeros up until the last score,
    // since we only care about the total accumulated at the end of all rounds
    let mut outs = vec![Fp::zero(); N];
    outs[N-1] = Fp::from(out);

    (circuit, vec![outs])
}

// Draws the circuit into the img folder
fn draw_circuit<F: PrimeField, const N: usize>(circuit: &AdventCircuit<F,N>) {
    use plotters::prelude::*;
    let root = BitMapBackend::new("img/advent2-layout.png", (1024, 3096)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled("Advent2 Layout", ("sans-serif", 60)).unwrap();
    halo2_proofs::dev::CircuitLayout::default()
        .render(4, circuit, &root)
        .unwrap();
}

fn main() {
    let k = 4;

    let xs = [0,1,2];
    let ys = [1,0,2];
    let out = 15;

    let (circuit, public_input) = make_circuit(xs, ys, out);

    draw_circuit(&circuit);

    MockProver::run(k, &circuit, public_input).unwrap().assert_satisfied();
}
