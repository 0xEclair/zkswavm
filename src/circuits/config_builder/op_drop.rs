use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, VirtualCells};
use num_bigint::BigUint;
use specs::etable::EventTableEntry;
use specs::itable::OpcodeClass;
use std::marker::PhantomData;

use crate::circuits::event::{EventCommonConfig, EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::utils::{bn_to_field, Context};
use crate::{constant, cur};

pub struct DropConfig<F: FieldExt> {
    enable: Column<Advice>,
    _mark: PhantomData<F>,
}

pub struct DropConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for DropConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        rtable: &RangeConfig<F>,
        itable: &InstructionConfig<F>,
        mtable: &MemoryConfig<F>,
        jtable: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        Box::new(DropConfig {
            enable: opcode_bit,
            _mark: PhantomData,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for DropConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // (3 << 77) * enable.cur
        constant!(bn_to_field(
            &(BigUint::from(OpcodeClass::Drop as u64) << (64 + 13))
        )) * cur!(meta, self.enable)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // -1 * enable.cur
        constant!(-F::one()) * cur!(meta, self.enable)
    }

    fn assign(&self, ctx: &mut Context<'_, F>, entry: &EventTableEntry) -> Result<(), Error> {
        Ok(())
    }

    fn opcode_class(&self) -> OpcodeClass {
        OpcodeClass::Drop
    }
}
