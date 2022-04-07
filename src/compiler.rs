use crate::ast::AST;
use crate::bytecode::*;
use crate::constants::*;
use crate::serializer::Serializable;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

trait Environments {
    fn enter_scope(&mut self);
    fn leave_scope(&mut self) -> Result<(), &'static str>;
    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, &'static str>;
    fn has_variable(&self, str: &String) -> bool;
}

struct Globals {
    globals: Vec<ConstantPoolIndex>,
}

impl Globals {
    pub fn new() -> Self {
        Globals{ globals: Vec::new() }
    }

    pub fn introduce_variable(&mut self, index: ConstantPoolIndex) {
        self.globals.push(index)
    }
}

#[derive(PartialEq)]
pub struct VecEnvironments {
    envs: Vec<HashMap<String, LocalFrameIndex>>,
    var_cnt: u16,
}

#[derive(PartialEq)]
pub enum Frame {
    // For globals we use the variable and function names, so there is
    // no need to store it as indexes.
    Top,
    // Locals are different kind of beast thought.
    Local(VecEnvironments),
}

impl VecEnvironments {
    /**
     * Initializes environments with one env present.
     */
    fn new() -> Self {
        VecEnvironments {
            envs: vec![HashMap::new(); 1],
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
                Ok(())
            }
            None => Err("No env to pop."),
        }
    }

    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, &'static str> {
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

    fn has_variable(&self, str: &String) -> bool {
        for env in self.envs.iter().rev() {
            let val = env.get(str);
            match val {
                Some(idx) => return true,
                _ => (),
            }
        };
        false
    }
}

pub fn compile(ast: &AST) -> std::io::Result<()> {
    let mut pool = ConstantPool::new();
    let mut code_dummy = Code::new();
    let mut frame = Frame::Top;
    let mut global_env = VecEnvironments::new();
    let mut globals = Globals::new();

    _compile(ast, &mut pool, &mut code_dummy, &mut frame, &mut globals,  &mut global_env, true);

    let mut f = File::create("foo.bc").expect("Unable to open output file.");
    pool.serializable_byte(&mut f)?;

    // TODO: Globals
    f.write(&[0 as u8, 0 as u8])?;

    // Main function is always added last.
    f.write(&(pool.len() - 1).to_le_bytes())?;

    println!("{:?}", pool);

    Ok(())
}

fn _compile(
    ast: &AST,
    pool: &mut ConstantPool,
    code: &mut Code,
    frame: &mut Frame,
    globals: &mut Globals,
    global_env: &mut VecEnvironments,
    drop: bool
) -> Result<(), &'static str> {
    match ast {
        AST::Integer(val) => {
            // Add it to constant pool.
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_unless(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Boolean(val) => {
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_unless(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Null => {
            let index = pool.push(Constant::Null);
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_unless(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Variable { name, value } => {
            _compile(value, pool, code, frame, globals, global_env, false)?;
            match frame {
                Frame::Local(env) => unimplemented!(),
                Frame::Top => {
                    global_env.introduce_variable(String::from(name.as_str()))?;

                    let name_index = pool.push(Constant::from(String::from(name.as_str())));
                    let slot_index = pool.push(Constant::Slot { name: name_index });
                    globals.introduce_variable(slot_index);

                    code.write_inst(Bytecode::SetGlobal{ name: name_index});
                }
            }

            Ok(())
        },
        AST::Array { size, value } => todo!(),
        AST::Object { extends, members } => todo!(),
        AST::AccessVariable { name } => {
            match frame {
                Frame::Local(env) => unimplemented!(),
                Frame::Top => {
                    if !global_env.has_variable(&name.0) {
                        Err("Variable doesn't exists.")
                    } else {
                        // Just create new string with the name
                        let idx = pool.push(Constant::from(name.0.clone()));
                        code.write_inst(Bytecode::GetGlobal { name: idx });
                        Ok(())
                    }
                }
            }
        },
        AST::AccessField { object, field } => todo!(),
        AST::AccessArray { array, index } => todo!(),
        AST::AssignVariable { name, value } => {
            _compile(ast, pool, code, frame, globals, global_env, false)?;
            match frame {
                Frame::Local(env) => unimplemented!(),
                Frame::Top => {
                    if !global_env.has_variable(&name.0) {
                        Err("Variable doesn't exists.")
                    } else {
                        let idx = pool.push(Constant::from(name.0.clone()));
                        code.write_inst(Bytecode::SetGlobal { name: idx });
                        Ok(())
                    }
                }
            }
        },
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
        } => {
            // Frame must be top
            if matches!(frame, Frame::Local(_)) {
                return Err("Functions can't be nested");
            }

            let mut env = VecEnvironments::new();

            // Add arguments
            for param in parameters.iter() {
                env.introduce_variable(param.0.clone())?;
            }

            let mut frame = Frame::Local(env);
            let mut fun_code = Code::new();

            _compile(body, pool, &mut fun_code, &mut frame, globals, global_env, true)?;

            let locals_cnt = match frame {
                Frame::Local(env) => env.var_cnt,
                _ => unreachable!(),
            };

            let func = Constant::Function {
                name: pool.push(Constant::from(name.0.clone())),
                parameters: parameters.len().try_into().unwrap(),
                locals: locals_cnt,
                code: fun_code
            };

            pool.push(func);

            Ok(())
        }
        AST::CallFunction { name, arguments } => todo!(),
        AST::CallMethod {
            object,
            name,
            arguments,
        } => todo!(),
        // Here, global statements or functions definitions are
        AST::Top(asts) => {
            // Create the 'main' function
            let mut code_main = Code::new();

            for ast in asts.iter() {
                // We send here code_main even if new function is encountered,
                // but that function will define it's own code vector anyway.
                _compile(ast, pool, &mut code_main, &mut Frame::Top, globals, global_env, true)?;
            }

            let func_name = pool.push(Constant::from(String::from("λ:")));
            let fun = Constant::Function {
                name: func_name,
                parameters: 0,
                locals: 0,
                code: code_main,
            };

            pool.push(fun);

            Ok(())
        }
        AST::Block(asts) => {
            let mut it = asts.iter().peekable();
            while let Some(ast) = it.next() {
                _compile(ast, pool, code, frame, globals, global_env, it.peek().is_some())?;
            }

            Ok(())
        },
        AST::Loop { condition, body } => todo!(),
        AST::Conditional {
            condition,
            consequent,
            alternative,
        } => todo!(),
        AST::Print { format, arguments } => {
            let string = pool.push(Constant::from(format.clone()));
            for ast in arguments.iter() {
                _compile(ast, pool, code, frame, globals, global_env, false)?;
            }
            let print = Bytecode::Print{ format: string, arguments: arguments.len().try_into().unwrap() };
            code.write_inst(print);
            Ok(())
        },
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
