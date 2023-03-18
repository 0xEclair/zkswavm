use std::marker::PhantomData;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Layouter, Value};
use halo2_proofs::plonk::{ConstraintSystem, Error, Expression, TableColumn, VirtualCells};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::utils::{bn_to_field};

pub struct MemoryInit {
    mmid: u64,
    offset: u64,
    value: u64,
}

impl MemoryInit {
    pub fn encode(&self) -> BigUint {
        let mut bn = BigUint::zero();
        bn += self.mmid;
        bn <<=16;
        bn += self.offset;
        bn <<=64;
        bn += self.value;
        bn
    }
}

const MEMORY_INIT_TABLE_COLUMNS: usize = 3;

pub struct MemoryInitConfig<F: FieldExt> {
    col: TableColumn,
    _mark: PhantomData<F>
}

impl<F: FieldExt> MemoryInitConfig<F> {
    pub fn new(col: TableColumn) -> MemoryInitConfig<F> {
        MemoryInitConfig {
            col,
            _mark: PhantomData,
        }
    }

    pub fn encode(&self, mmid: Expression<F>, offset: Expression<F>, value: Expression<F>) -> Expression<F> {
        mmid * Expression::Constant(bn_to_field(&(BigUint::one() << 80)))
             + offset * Expression::Constant(bn_to_field(&(BigUint::one() << 64)))
             + value
    }

    pub fn configure_in_range(&self, meta: &mut ConstraintSystem<F>, key: &'static str, expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>) {
        meta.lookup(key, |meta| vec![(expr(meta), self.col)]);
    }
}

pub struct MemoryInitChip<F: FieldExt> {
    config: MemoryInitConfig<F>,
    _phantom: PhantomData<F>,
}

impl<F: FieldExt> MemoryInitChip<F> {
    pub fn add_memory_init(self, layouter: &mut impl Layouter<F>, memory_init: Vec<MemoryInit>) -> Result<(), Error> {
        layouter.assign_table(
            || "memory_init",
            |mut table| {
                for(i, v) in memory_init.iter().enumerate() {
                    table.assign_cell(
                        || "memory init table",
                        self.config.col,
                        i,
                        || Value::known(bn_to_field::<F>(&v.encode())),
                    )?;
                }

                Ok(())
            }
        )?;

        Ok(())
    }
}