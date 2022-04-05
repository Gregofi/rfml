use std::io::Write;
use crate::constants::ConstantPoolIndex;
use crate::constants::ConstantPool;

pub type LocalFrameIndex = u16;
pub type ArgsCount = i8;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Bytecode {
    Literal {
        index: ConstantPoolIndex,
    },
    GetLocal {
        index: LocalFrameIndex,
    },
    SetLocal {
        index: LocalFrameIndex,
    },
    GetGlobal {
        name: ConstantPoolIndex,
    },
    SetGlobal {
        name: ConstantPoolIndex,
    },
    Object {
        class: ConstantPoolIndex,
    },
    Array,
    GetField {
        name: ConstantPoolIndex,
    },
    SetField {
        name: ConstantPoolIndex,
    },
    CallMethod {
        name: ConstantPoolIndex,
        arguments: ArgsCount,
    },
    CallFunction {
        name: ConstantPoolIndex,
        arguments: ArgsCount,
    },
    Label {
        name: ConstantPoolIndex,
    },
    Print {
        format: ConstantPoolIndex,
        arguments: ArgsCount,
    },
    Jump {
        label: ConstantPoolIndex,
    },
    Branch {
        label: ConstantPoolIndex,
    },
    Return,
    Drop,
}

trait Serializable {
    fn serializable<W: Write> (&self, output: &mut W) -> Result<(), &'static str>;
}

impl Serializable for Bytecode {
    fn serializable<W: Write> (&self, output: &mut W) -> Result<(), &'static str> {
        match self {
            Bytecode::Literal { index } => todo!(),
            Bytecode::GetLocal { index } => todo!(),
            Bytecode::SetLocal { index } => todo!(),
            Bytecode::GetGlobal { name } => todo!(),
            Bytecode::SetGlobal { name } => todo!(),
            Bytecode::Object { class } => todo!(),
            Bytecode::Array => todo!(),
            Bytecode::GetField { name } => todo!(),
            Bytecode::SetField { name } => todo!(),
            Bytecode::CallMethod { name, arguments } => todo!(),
            Bytecode::CallFunction { name, arguments } => todo!(),
            Bytecode::Label { name } => todo!(),
            Bytecode::Print { format, arguments } => todo!(),
            Bytecode::Jump { label } => todo!(),
            Bytecode::Branch { label } => todo!(),
            Bytecode::Return => todo!(),
            Bytecode::Drop => todo!(),
        }
    }
}

pub struct Program(Vec<Bytecode>);

