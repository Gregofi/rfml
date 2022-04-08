use crate::bytecode::Code;
use crate::serializer::Serializable;

pub type ConstantPoolIndex = u16;

fn from_usize(i: usize) -> ConstantPoolIndex {
    i.try_into().unwrap()
}

#[derive(PartialEq, Clone, Debug)]
pub enum Constant {
    Integer(i32),
    Boolean(bool),
    Null,
    String(String),
    Slot{name: ConstantPoolIndex},
    Function{name: ConstantPoolIndex, parameters: u8, locals: u16, code: Code}
}

impl From<i32> for Constant {
    fn from(num: i32) -> Self {
        Constant::Integer(num)
    }
}

impl From<bool> for Constant {
    fn from(b: bool) -> Self {
        Constant::Boolean(b)
    }
}

impl From<String> for Constant {
    fn from(s: String) -> Self {
        Constant::String(s)
    }
}

impl Serializable for Constant {
    fn serializable_byte<W: std::io::Write> (&self, output: &mut W) -> std::io::Result<()> {
        match self {
            Constant::Integer(val) => {
                output.write(&[0x00 as u8])?;
                output.write(&(val.to_le_bytes()))?;
            },
            Constant::Boolean(val) => {
                output.write(&[0x06 as u8])?;
                output.write(&((*val as u8).to_le_bytes()))?;
            },
            Constant::Null => {
                output.write(&[0x01 as u8])?;
            },
            Constant::String(str) => {
                output.write(&[0x02 as u8])?;
                output.write(&(str.len() as u32).to_le_bytes())?;
                output.write(str.as_bytes())?;
            },
            Constant::Slot { name } => {
                output.write(&0x04u8.to_le_bytes())?;
                output.write(&name.to_le_bytes())?;
            },
            Constant::Function { name, parameters, locals, code } => {
                output.write(&[0x03 as u8])?;
                output.write(&name.to_le_bytes())?;
                output.write(&parameters.to_le_bytes())?;
                output.write(&locals.to_le_bytes())?;
                output.write(&code.len().to_le_bytes())?;
                for bytecode in code.insert_point.iter() {
                    bytecode.serializable_byte(output)?;
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ConstantPool(Vec<Constant>);


impl ConstantPool {
    pub fn new() -> Self {
        ConstantPool(Vec::new())
    }

    pub fn push(&mut self, constant: Constant) -> ConstantPoolIndex {
        self.0.push(constant);
        (self.0.len() - 1).try_into().unwrap()
    }

    pub fn find(&mut self, constant: &Constant) -> Option<ConstantPoolIndex> {
        self.0.iter().position(|x| constant == x).map(|x| from_usize(x))
    }

    pub fn len(&self) -> u16 {
        self.0.len().try_into().unwrap()
    }
}


impl Serializable for ConstantPool {
    fn serializable_byte<W: std::io::Write> (&self, output: &mut W) -> std::io::Result<()> {
        output.write(&self.len().to_le_bytes())?;

        for constant in self.0.iter() {
            constant.serializable_byte(output)?;
        }

        Ok(())
    }
}