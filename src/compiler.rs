use crate::ast::AST;
use crate::bytecode::*;
use crate::constants::*;
use crate::serializer::Serializable;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

trait Environments {
    fn enter_scope(&mut self);
    fn leave_scope(&mut self) -> Result<(), &'static str>;
    fn introduce_variable(&mut self, str: String) -> Result<LocalFrameIndex, &'static str>;
    fn has_variable(&self, str: &String) -> Option<LocalFrameIndex>;
    fn is_topmost(&self) -> bool;
}

#[derive(Debug)]
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

    pub fn len(&self) -> u16 {
        self.globals.len().try_into().unwrap()
    }
}

impl Serializable for Globals {
    fn serializable_byte<W: Write> (&self, output: &mut W) -> std::io::Result<()> {
        output.write(&self.globals.len().to_le_bytes())?;
        for global in self.globals.iter() {
            output.write(&global.to_le_bytes())?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Debug)]
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
        // Check if the variable doesn't already exist in the most topmost scope
        let env = self.envs.last_mut().expect("There is no scope.");
        match env.get(&str) {
            Some(_) => return Err("Variable already exists."),
            _ => (),
        }

        // Create new variable
        env.insert(str, self.var_cnt);
        self.var_cnt += 1;
        Ok(self.var_cnt - 1)
    }

    fn has_variable(&self, str: &String) -> Option<LocalFrameIndex> {
        // Check if variable is located in any environment, start
        // from the last.
        for env in self.envs.iter().rev() {
            let val = env.get(str);
            match val {
                Some(idx) => return Some(*idx),
                _ => (),
            }
        };
        None
    }

    fn is_topmost(&self) -> bool {
        self.envs.len() == 1
    }
}

pub fn compile(ast: &AST) -> std::io::Result<()> {
    let mut pool = ConstantPool::new();
    let mut code_dummy = Code::new();
    let mut frame = Frame::Top;
    let mut global_env = VecEnvironments::new();
    let mut globals = Globals::new();

    _compile(ast, &mut pool, &mut code_dummy, &mut frame, &mut globals,  &mut global_env, true).expect("Compilation failed");

    let mut f = File::create("foo.bc").expect("Unable to open output file.");
    pool.serializable_byte(&mut f)?;

    // Serialize globals
    f.write(&globals.len().to_le_bytes())?;
    globals.serializable_byte(&mut f)?;

    // Main function is always added last.
    f.write(&(pool.len() - 1).to_le_bytes())?;

    println!("{:?}\n{:?}", pool, globals);
    println!("EP: {0}", pool.len() - 1);

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
                Frame::Top if !global_env.is_topmost() => {
                    println!("{:?}", global_env);
                    let index = global_env.introduce_variable(name.0.clone())
                        .unwrap_or_else(|_| panic!("Variable '{0}' already exists in global environment", name.0));
                    _compile(value, pool, code, frame, globals, global_env, false)?;
                    code.write_inst(Bytecode::SetLocal { index: index });
                }
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
                // This behaves same way as local variable
                Frame::Top if !global_env.is_topmost() && global_env.has_variable(&name.0).is_some() => {
                    let idx = global_env.has_variable(&name.0).expect("Variable is not defined.");
                    code.write_inst(Bytecode::GetLocal { index: idx });
                    Ok(())
                }
                Frame::Top => {
                    if global_env.has_variable(&name.0).is_none() {
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
                Frame::Top if !global_env.is_topmost() && global_env.has_variable(&name.0).is_some() => {
                    let idx = global_env.has_variable(&name.0).unwrap();
                    code.write_inst(Bytecode::SetLocal { index: idx });
                    Ok(())
                }
                Frame::Top => {
                    if global_env.has_variable(&name.0).is_none() {
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
                locals: global_env.var_cnt,
                code: code_main,
            };

            pool.push(fun);

            Ok(())
        }
        AST::Block(asts) => {
            match frame {
                Frame::Top => {
                    global_env.enter_scope();
                }
                Frame::Local(env) => {
                    env.enter_scope();
                }
            }

            let mut it = asts.iter().peekable();
            // Discard all values from stack except the last one
            while let Some(ast) = it.next() {
                _compile(ast, pool, code, frame, globals, global_env, it.peek().is_some())?;
            }

            match frame {
                Frame::Top => {
                    global_env.leave_scope()?;
                }
                Frame::Local(env) => {
                    env.leave_scope()?;
                }
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
