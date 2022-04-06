use std::io::Write;
use crate::constants::ConstantPoolIndex;
use crate::constants::ConstantPool;
use crate::serializer::Serializable;

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

impl Serializable for Bytecode {
    fn serializable_byte<W: Write> (&self, output: &mut W) -> Result<(), &'static str> {
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

    fn serializable_human (&self) {
        match self {
            Bytecode::Literal { index } => {
                print!("lit {0}", index);
            }
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
            Bytecode::Print { format, arguments } => {
                print!("print {0} {1}", format, arguments);
            }
            Bytecode::Jump { label } => todo!(),
            Bytecode::Branch { label } => todo!(),
            Bytecode::Return => {
                print!("return");
            },
            Bytecode::Drop => {
                print!("drop");
            },
        };
    }
}

#[derive(PartialEq, Clone)]
pub struct Code {
    pub insert_point: Vec<Bytecode>,
}

impl Code {
    pub fn new() -> Code {
        Code {
            insert_point: Vec::new(),
        }
    }

    pub fn write_inst(&mut self, inst: Bytecode) {
        self.insert_point.push(inst)
    }

    pub fn write_insts(&mut self, insts: Code) {
        self.insert_point.extend(insts.insert_point);
    }
}

