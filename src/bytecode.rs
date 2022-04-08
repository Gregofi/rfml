use std::io::Write;
use crate::constants::ConstantPoolIndex;
use crate::constants::ConstantPool;
use crate::serializer::Serializable;

pub type LocalFrameIndex = u16;
pub type ArgsCount = u8;

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
    fn serializable_byte<W: Write> (&self, output: &mut W) -> std::io::Result<()> {
        match self {
            Bytecode::Literal { index } => {
                output.write(&0x01u8.to_le_bytes())?;
                output.write(&index.to_le_bytes())?;
            },
            Bytecode::GetLocal { index } => {
                output.write(&0x0Au8.to_le_bytes())?;
                output.write(&index.to_le_bytes())?;
            },
            Bytecode::SetLocal { index } => {
                output.write(&0x09u8.to_le_bytes())?;
                output.write(&index.to_le_bytes())?;
            },
            Bytecode::GetGlobal { name } => {
                output.write(&0x0Cu8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Bytecode::SetGlobal { name } => {
                output.write(&0x0Bu8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Bytecode::Object { class } => {
                output.write(&0x04u8.to_le_bytes())?;
                output.write(&class.to_le_bytes())?;
            },
            Bytecode::Array => todo!(),
            Bytecode::GetField { name } => {
                output.write(&0x05u8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Bytecode::SetField { name } => {
                output.write(&0x06u8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Bytecode::CallMethod { name, arguments } => todo!(),
            Bytecode::CallFunction { name, arguments } => {
                output.write(&0x08u8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
                output.write(&arguments.to_le_bytes())?;
            },
            Bytecode::Label { name } => {
                output.write(&0x00u8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Bytecode::Print { format, arguments } => {
                output.write(&[0x02 as u8])?;
                output.write(&format.to_le_bytes())?;
                output.write(&arguments.to_le_bytes())?;
            },
            Bytecode::Jump { label } => {
                output.write(&0x0Eu8.to_le_bytes())?;
                output.write(&label.to_le_bytes())?;
            },
            Bytecode::Branch { label } => {
                output.write(&0x0Du8.to_le_bytes())?;
                output.write(&label.to_le_bytes())?;

            }
            Bytecode::Return => {
                output.write(&0x0Fu8.to_le_bytes())?;
            },
            Bytecode::Drop => {
                output.write(&0x10u8.to_le_bytes())?;
            }
        };

        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
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

    pub fn write_inst_if(&mut self, inst: Bytecode, cond: bool) {
        if cond {
            self.insert_point.push(inst)
        }
    }

    pub fn write_insts(&mut self, insts: Code) {
        self.insert_point.extend(insts.insert_point);
    }

    pub fn len(&self) -> u32 {
        self.insert_point.len().try_into().unwrap()
    }
}

