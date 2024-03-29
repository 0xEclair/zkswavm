use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::plonk::{
    Column, ConstraintSystem, Error, Expression, Fixed, TableColumn, VirtualCells,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use specs::itable::InstructionTableEntry;
use std::marker::PhantomData;
use wasmi::tracer::itable::IEntry;

use crate::circuits::utils::bn_to_field;
use crate::constant;

trait Encode {
    fn encode(&self) -> BigUint;
    fn encode_addr(&self) -> BigUint;
}

impl Encode for InstructionTableEntry {
    fn encode(&self) -> BigUint {
        let opcode: BigUint = self.opcode.clone().into();
        let mut bn = self.encode_addr();
        bn <<= 128usize;
        bn += opcode;
        bn
    }

    fn encode_addr(&self) -> BigUint {
        let mut bn = BigUint::zero();
        bn += self.moid;
        bn <<= 16u8;
        bn += self.mmid;
        bn <<= 16u8;
        bn += self.fid;
        bn <<= 16u8;
        bn += self.bid;
        bn <<= 16u8;
        bn += self.iid;
        bn
    }
}

pub fn encode_inst_expr<F: FieldExt>(
    moid: Expression<F>,
    mmid: Expression<F>,
    fid: Expression<F>,
    bid: Expression<F>,
    iid: Expression<F>,
    opcode: Expression<F>,
) -> Expression<F> {
    let mut bn = BigUint::one();
    let mut acc = opcode;
    bn <<= 64u8;
    acc = acc + iid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + bid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + fid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + mmid * constant!(bn_to_field(&bn));
    bn <<= 16u8;
    acc = acc + moid * constant!(bn_to_field(&bn));

    acc
}

#[derive(Clone)]
pub struct InstructionConfig<F: FieldExt> {
    col: TableColumn,
    _mark: PhantomData<F>,
}

impl<F: FieldExt> InstructionConfig<F> {
    pub fn configure(col: TableColumn) -> InstructionConfig<F> {
        InstructionConfig {
            col,
            _mark: PhantomData,
        }
    }

    pub fn configure_in_table(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup(key, |meta| vec![(expr(meta), self.col)]);
    }
}

#[derive(Clone)]
pub struct InstructionChip<F: FieldExt> {
    config: InstructionConfig<F>,
}

impl<F: FieldExt> InstructionChip<F> {
    pub fn new(config: InstructionConfig<F>) -> InstructionChip<F> {
        InstructionChip { config }
    }

    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        instructions: &Vec<InstructionTableEntry>,
    ) -> Result<(), Error> {
        layouter.assign_table(
            || "itable",
            |mut table| {
                for (i, v) in instructions.iter().enumerate() {
                    table.assign_cell(
                        || "init instruction table",
                        self.config.col,
                        i,
                        || Ok(bn_to_field::<F>(&v.encode())),
                    )?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}
