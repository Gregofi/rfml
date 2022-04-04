use std::collections::HashMap;
use crate::constants::*;
use crate::bytecode::LocalFrameIndex;
use crate::ast;

struct Compiler {
    constant_pool: Vec<Box<Constant>>,
}

type Offset = i32;

trait Environments {
    fn enter_scope(&mut self);
    fn leave_scope(&mut self) -> Result<(), &'static str>;
    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, String>;
}

struct VecEnvironments{envs: Vec<HashMap<String, LocalFrameIndex>>, var_cnt: i16}


enum Frame {
    // For globals we use the variable and function names, so there is
    // no need to store it as indexes.
    Top,
    // Locals are different kind of beast thought.
    Local(VecEnvironments),
}

pub fn compile() -> Result<(), &'static str> {
    unimplemented!();
}

impl VecEnvironments {
    fn new() -> Self {
        VecEnvironments{envs: Vec::new(), var_cnt: 0}
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
            None => Err("No env to pop.")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_test() {
        let mut env = VecEnvironments::new();
        env.enter_scope();
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index.")
        }
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index.")
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index.")
        }
        env.enter_scope();
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index.")
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index.")
        }
        match env.introduce_variable(String::from("c")) {
            Ok(idx) if idx == 2 => (),
            _ => panic!("No insert or wrong index.")
        }
        env.leave_scope();
        match env.introduce_variable(String::from("b")) {
            Ok(idx) if idx == 1 => (),
            _ => panic!("No insert or wrong index.")
        }
        match env.introduce_variable(String::from("a")) {
            Ok(idx) if idx == 0 => (),
            _ => panic!("No insert or wrong index.")
        }
        // d should reuse the index from c
        match env.introduce_variable(String::from("d")) {
            Ok(idx) if idx == 2 => (),
            _ => panic!("No insert or wrong index.")
        }


        match env.leave_scope() {
            Err(mess) => panic!("{}", mess),
            _ => ()
        }
        match env.leave_scope() {
            Ok(_) => panic!("There shouldn't be an enviroment to pop."),
            _ => (),
        }
    }
}