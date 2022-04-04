use crate::ast::{self, AST};
use crate::bytecode::{Bytecode, LocalFrameIndex};
use crate::constants::*;
use std::collections::HashMap;

type Offset = i32;

pub struct Code {
    insert_point: Vec<Bytecode>,
}

impl Code {
    pub fn write_inst(&mut self, inst: Bytecode) {
        self.insert_point.push(inst)
    }
}

trait Environments {
    fn enter_scope(&mut self);
    fn leave_scope(&mut self) -> Result<(), &'static str>;
    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, String>;
}

struct VecEnvironments {
    envs: Vec<HashMap<String, LocalFrameIndex>>,
    var_cnt: i16,
}

enum Frame {
    // For globals we use the variable and function names, so there is
    // no need to store it as indexes.
    Top,
    // Locals are different kind of beast thought.
    Local(VecEnvironments),
}

impl VecEnvironments {
    fn new() -> Self {
        VecEnvironments {
            envs: Vec::new(),
            var_cnt: 0,
        }
    }
}

impl Environments for VecEnvironments {
    fn enter_scope(&mut self) {
        self.envs.push(HashMap::new());
    }

    fn leave_scope(&mut self) -> Result<(), &'static str> {
        match self.envs.pop() {
            Some(env) => {
                self.var_cnt -= env.keys().len() as i16;
                Ok(())
            }
            None => Err("No env to pop."),
        }
    }

    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, String> {
        // Check if the variable doesn't already exist
        for env in self.envs.iter().rev() {
            let val = env.get(&str);
            match val {
                Some(idx) => return Ok(*idx),
                _ => (),
            }
        }

        // If not, create new
        let env = self.envs.last_mut().unwrap();
        env.insert(str, self.var_cnt);
        self.var_cnt += 1;
        Ok(self.var_cnt - 1)
    }
}

pub fn compile(ast: &AST, pool: &mut ConstantPool, code: &mut Code) -> Result<(), &'static str> {
    match ast {
        AST::Integer(val) => {
            // Add it to constant pool.
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            Ok(())
        }
        AST::Boolean(val) => {
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            Ok(())
        }
        AST::Null => {
            let index = pool.push(Constant::Null);
            code.write_inst(Bytecode::Literal { index });
            Ok(())
        }
        AST::Variable { name, value } => todo!(),
        AST::Array { size, value } => todo!(),
        AST::Object { extends, members } => todo!(),
        AST::AccessVariable { name } => todo!(),
        AST::AccessField { object, field } => todo!(),
        AST::AccessArray { array, index } => todo!(),
        AST::AssignVariable { name, value } => todo!(),
        AST::AssignField {
            object,
            field,
            value,
        } => todo!(),
        AST::AssignArray {
            array,
            index,
            value,
        } => todo!(),
        AST::Function {
            name,
            parameters,
            body,
        } => todo!(),
        AST::CallFunction { name, arguments } => todo!(),
        AST::CallMethod {
            object,
            name,
            arguments,
        } => todo!(),
        AST::Top(_) => {
            // Here, global statements or functions definitions are

            unimplemented!();
        }
        AST::Block(_) => todo!(),
        AST::Loop { condition, body } => todo!(),
        AST::Conditional {
            condition,
            consequent,
            alternative,
        } => todo!(),
        AST::Print { format, arguments } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_test() {
        let mut env = VecEnvironments::new();
        env.enter_scope();
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index."),
        }
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index."),
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index."),
        }
        env.enter_scope();
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index."),
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index."),
        }
        match env.introduce_variable(String::from("c")) {
            Ok(idx) if idx == 2 => (),
            _ => panic!("No insert or wrong index."),
        }
        env.leave_scope();
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index."),
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index."),
        }
        // d should reuse the index from c
        match env.introduce_variable(String::from("d")) {
            Ok(idx) if idx == 2 => (),
            _ => panic!("No insert or wrong index."),
        }

        match env.leave_scope() {
            Err(mess) => panic!("{}", mess),
            _ => (),
        }
        match env.leave_scope() {
            Ok(_) => panic!("There shouldn't be an enviroment to pop."),
            _ => (),
        }
    }
}
