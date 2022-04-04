
pub type ConstantPoolIndex = i16;

fn from_usize(i: usize) -> ConstantPoolIndex {
    i.try_into().unwrap()
}

#[derive(Eq, PartialEq, Clone)]
pub enum Constant {
    Integer(i32),
    Boolean(bool),
    Null,
    String(String),
    Slot{name: i32},
    Function{name: i32, parameters: i32, locals: i32, code: Box<Vec<i8>>}
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
