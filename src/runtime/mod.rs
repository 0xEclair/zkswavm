pub mod wasmi_interpreter;

use specs::step::StepInfo;
use specs::{
    etable::EventTableEntry,
    mtable::{AccessType, LocationType, MemoryTableEntry, VarType},
    types::{CompileError, ExecutionError, Value},
    CompileTable, ExecutionTable,
};

use crate::runtime::wasmi_interpreter::WasmiRuntime;

pub struct CompileOutcome<M> {
    pub textual_repr: String,
    pub module: M,
    pub tables: CompileTable,
}

pub struct ExecutionOutcome {
    pub tables: ExecutionTable,
}

pub trait WasmRuntime {
    type Module;

    fn new() -> Self;
    fn compile(&self, textual_repr: &str) -> Result<CompileOutcome<Self::Module>, CompileError>;
    fn run(
        &self,
        compile_outcome: &CompileOutcome<Self::Module>,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<ExecutionOutcome, ExecutionError>;
}

pub type WasmInterpreter = WasmiRuntime;

pub fn memory_event_of_step(event: &EventTableEntry, emid: &mut u64) -> Vec<MemoryTableEntry> {
    let eid = event.eid;
    let mmid = event.inst.mmid.into();

    match &event.step_info {
        StepInfo::BrIfNez { value, dst_pc } => mem_op_from_stack_only_step(
            eid,
            emid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*value as u64],
            &[],
        ),
        StepInfo::Return {
            drop,
            keep,
            drop_values,
            keep_values,
        } => {
            assert_eq!(*drop as usize, drop_values.len());
            assert_eq!(keep.len(), keep_values.len());
            mem_op_from_stack_only_step(
                eid,
                emid,
                mmid,
                VarType::I32,
                VarType::I32,
                drop_values.iter().map(|value| *value).collect::<Vec<_>>()[..]
                    .try_into()
                    .unwrap(),
                keep_values.iter().map(|value| *value).collect::<Vec<_>>()[..]
                    .try_into()
                    .unwrap(),
            )
        }
        StepInfo::Drop { value } => {
            mem_op_from_stack_only_step(eid, emid, mmid, VarType::I32, VarType::I32, &[*value], &[])
        }
        StepInfo::Call { index } => {
            vec![]
        }
        StepInfo::GetLocal {
            depth,
            vtype,
            value,
        } => {
            let read = MemoryTableEntry {
                eid,
                emid: *emid,
                mmid,
                offset: *depth as u64,
                ltype: LocationType::Stack,
                atype: AccessType::Read,
                vtype: *vtype,
                value: *value,
            };
            *emid = (*emid).checked_add(1).unwrap();

            let write = MemoryTableEntry {
                eid,
                emid: *emid,
                mmid: mmid.into(),
                offset: 0,
                ltype: LocationType::Stack,
                atype: AccessType::Write,
                vtype: *vtype,
                value: *value,
            };
            *emid = (*emid).checked_add(1).unwrap();

            vec![read, write]
        }
        StepInfo::I32Const { value } => mem_op_from_stack_only_step(
            eid,
            emid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[],
            &[*value as u64],
        ),
        StepInfo::I32BinOp { left, right, value } => mem_op_from_stack_only_step(
            eid,
            emid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*right as u64, *left as u64],
            &[*value as u64],
        ),
        StepInfo::I32Comp { left, right, value } => mem_op_from_stack_only_step(
            eid,
            emid,
            mmid,
            VarType::I32,
            VarType::I32,
            &[*right as u64, *left as u64],
            &[*value as u64],
        ),
    }
}

fn mem_op_from_stack_only_step(
    eid: u64,
    emid: &mut u64,
    mmid: u64,
    inputs_type: VarType,
    outputs_type: VarType,
    pop_values: &[u64],
    push_values: &[u64],
) -> Vec<MemoryTableEntry> {
    let mut mem_ops = vec![];

    for i in 0..pop_values.len() {
        mem_ops.push(MemoryTableEntry {
            eid,
            emid: *emid,
            mmid,
            offset: i as u64,
            ltype: LocationType::Stack,
            atype: AccessType::Read,
            vtype: inputs_type,
            value: pop_values[i],
        });
        *emid = (*emid).checked_add(1).unwrap();
    }

    for i in 0..push_values.len() {
        mem_ops.push(MemoryTableEntry {
            eid,
            emid: *emid,
            mmid,
            offset: i as u64,
            ltype: LocationType::Stack,
            atype: AccessType::Write,
            vtype: outputs_type,
            value: push_values[i],
        });
        *emid = (*emid).checked_add(1).unwrap();
    }

    mem_ops
}
