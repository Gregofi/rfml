use crate::bytecode::Code;
use crate::serializer::Serializable;

pub type ConstantPoolIndex = i16;

fn from_usize(i: usize) -> ConstantPoolIndex {
    i.try_into().unwrap()
}

#[derive(PartialEq, Clone)]
pub enum Constant {
    Integer(i32),
    Boolean(bool),
    Null,
    String(String),
    Slot{name: i32},
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
    fn serializable_byte<W: std::io::Write> (&self, output: &mut W) -> Result<(), &'static str> {
        match self {
            Constant::Integer(_) => todo!(),
            Constant::Boolean(_) => todo!(),
            Constant::Null => todo!(),
            Constant::String(_) => todo!(),
            Constant::Slot { name } => todo!(),
            Constant::Function { name, parameters, locals, code } => todo!(),
        }
    }

    fn serializable_human(&self) {
        match self {
            Constant::Integer(val) => {
                print!("{0}", val);
            }
            Constant::Boolean(val) => {
                print!("{0}", val);
            },
            Constant::Null => {
                print!("null");
            },
            Constant::String(str) => {
                print!("{0}", str);
            },
            Constant::Slot { name } => todo!(),
            Constant::Function { name, parameters, locals, code } => {
                print!("fn{{ name: {0}, parameters: {1}, locals: {2}, code:\n", name, parameters, locals);
                for inst in code.insert_point.iter() {
                    inst.serializable_human();
                    println!("");
                }
                print!("}}");
            },
        };
    }
}

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
}
