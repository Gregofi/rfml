

pub enum Constant {
    Integer(i32),
    Boolean(bool),
    Null,
    String(String),
    Slot{name: i32},
    Function{name: i32, parameters: i32, locals: i32, code: Box<Vec<i8>>}
}
