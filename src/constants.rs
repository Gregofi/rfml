use crate::bytecode::Code;

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

pub struct ConstantPool(Vec<Constant>);
impl ConstantPool {
    fn new() -> Self {
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
