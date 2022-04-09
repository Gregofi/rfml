use crate::ast::Identifier;
use crate::ast::IntoBoxed;
use crate::ast::AST;
use crate::bytecode::*;
use crate::constants::*;
use crate::serializer::Serializable;
use std::collections::HashMap;
use std::io;
use std::io::Write;

struct RandomNameGenerator {
    cnt: usize,
}

impl RandomNameGenerator {
    fn new() -> Self {
        RandomNameGenerator { cnt: 0 }
    }

    fn generate(&mut self, str: &'static str) -> String {
        if str.chars().any(char::is_numeric) {
            panic!("String can't contain number!");
        }
        let mut new_str = String::from(str);
        new_str.push_str(&format!("_{}", self.cnt));
        self.cnt += 1;
        new_str
    }
}

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
        Globals {
            globals: Vec::new(),
        }
    }

    pub fn introduce_variable(&mut self, index: ConstantPoolIndex) {
        self.globals.push(index)
    }

    pub fn len(&self) -> u16 {
        self.globals.len().try_into().unwrap()
    }
}

impl Serializable for Globals {
    fn serializable_byte<W: Write>(&self, output: &mut W) -> std::io::Result<()> {
        output.write(&self.len().to_le_bytes())?;
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
            Some(_) => Ok(()),
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
        }
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
    let mut generator = RandomNameGenerator::new();

    _compile(
        ast,
        &mut pool,
        &mut code_dummy,
        &mut frame,
        &mut globals,
        &mut global_env,
        &mut generator,
        true,
    )
    .expect("Compilation failed");

    let mut f = io::stdout();
    pool.serializable_byte(&mut f)?;

    // Serialize globals
    // f.write(&0u16.to_le_bytes());
    globals.serializable_byte(&mut f)?;

    // Entry point: Main function is always added last.
    f.write(&(pool.len() - 1 as u16).to_le_bytes())?;

    // println!("{:?}\n{:?}", pool, globals);
    // println!("EP: {0}", pool.len() - 1);

    Ok(())
}

fn compile_fun_def(
    name: String,
    parameters: &Vec<Identifier>,
    body: &Box<AST>,
    is_method: bool,
    pool: &mut ConstantPool,
    globals: &mut Globals,
    global_env: &mut VecEnvironments,
    generator: &mut RandomNameGenerator,
) -> Result<ConstantPoolIndex, &'static str> {
    let mut env = VecEnvironments::new();
    if is_method {
        env.introduce_variable(String::from("this"))?;
    }

    for param in parameters.iter() {
        env.introduce_variable(param.0.clone())?;
    }

    let mut frame = Frame::Local(env);
    let mut code = Code::new();

    _compile(
        &body, pool, &mut code, &mut frame, globals, global_env, generator, false,
    )?;

    code.write_inst(Bytecode::Return);

    let locals = match frame {
        Frame::Local(env) => env.var_cnt,
        _ => unreachable!(),
    };

    let func = Constant::Function {
        name: pool.push(Constant::from(name)),
        parameters: (parameters.len() + is_method as usize).try_into().unwrap(),
        locals,
        code,
    };

    let fun_idx = pool.push(func);

    Ok(fun_idx)
}

fn _compile(
    ast: &AST,
    pool: &mut ConstantPool,
    code: &mut Code,
    frame: &mut Frame,
    globals: &mut Globals,
    global_env: &mut VecEnvironments,
    generator: &mut RandomNameGenerator,
    drop: bool,
) -> Result<(), &'static str> {
    match ast {
        AST::Integer(val) => {
            // Add it to constant pool.
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_if(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Boolean(val) => {
            let index = pool.push(Constant::from(*val));
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_if(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Null => {
            let index = pool.push(Constant::Null);
            code.write_inst(Bytecode::Literal { index });
            code.write_inst_if(Bytecode::Drop, drop);
            Ok(())
        }
        AST::Variable { name, value } => {
            _compile(
                value, pool, code, frame, globals, global_env, generator, false,
            )?;
            match frame {
                Frame::Local(env) => {
                    let index = env.introduce_variable(name.0.clone()).unwrap_or_else(|_| {
                        panic!(
                            "Variable '{0}' already exists in global environment",
                            name.0
                        )
                    });
                    code.write_inst(Bytecode::SetLocal { index: index });
                }
                Frame::Top if !global_env.is_topmost() => {
                    let index = global_env
                        .introduce_variable(name.0.clone())
                        .unwrap_or_else(|_| {
                            panic!(
                                "Variable '{0}' already exists in global environment",
                                name.0
                            )
                        });
                    code.write_inst(Bytecode::SetLocal { index: index });
                }
                Frame::Top => {
                    let name_index = pool.push(Constant::from(String::from(name.as_str())));
                    let slot_index = pool.push(Constant::Slot { name: name_index });
                    globals.introduce_variable(slot_index);
                    code.write_inst(Bytecode::SetGlobal { name: name_index });
                }
            }

            Ok(())
        }
        AST::Array { size, value } => {
            match **value {
                AST::Integer(_) | AST::Null | AST::AccessField {..} | AST::AccessArray {..} | AST::AccessVariable {..} => {
                    _compile(size, pool, code, frame, globals, global_env, generator, false)?;
                    _compile(value, pool, code, frame, globals, global_env, generator, false)?;
                    code.write_inst(Bytecode::Array);
                    Ok(())
                }
                _ => {
                    // Create a while loop that iterates over the array and evaluates the value every time
                    // var i = 0;
                    // var size = 0;
                    // var array = array(size, null);
                    // while ( i < size ) {
                    //    arr[i] = value;
                    //    i <- i + 1;
                    // }

                    // var i = 0;
                    let iter_var_name = generator.generate("i");
                    let iter_var = AST::Variable {
                        name: Identifier(iter_var_name.clone()),
                        value: AST::Integer(0).into_boxed(),
                    };
                    _compile(
                        &iter_var, pool, code, frame, globals, global_env, generator, true,
                    )?;

                    // var size = 0;
                    let size_var_name = generator.generate("size");
                    let size_var = AST::Variable {
                        name: Identifier(size_var_name.clone()),
                        value: size.clone(),
                    };
                    _compile(
                        &size_var, pool, code, frame, globals, global_env, generator, true,
                    )?;

                    // var array = array(size, null)
                    let array_var_name = generator.generate("array");
                    let array_var = AST::Variable {
                        name: Identifier(array_var_name.clone()),
                        value: AST::Array {
                            size: AST::AccessVariable {
                                name: Identifier(size_var_name.clone()),
                            }
                            .into_boxed(),
                            value: AST::Null.into_boxed(),
                        }
                        .into_boxed(),
                    };
                    _compile(
                        &array_var, pool, code, frame, globals, global_env, generator, true,
                    )?;

                    // arr[i] = value
                    let assign = AST::AssignArray {
                        array: AST::AccessVariable {
                            name: Identifier(array_var_name.clone()),
                        }
                        .into_boxed(),
                        index: AST::AccessVariable {
                            name: Identifier(iter_var_name.clone()),
                        }
                        .into_boxed(),
                        value: value.clone(),
                    }
                    .into_boxed();

                    // i <- i + 1
                    let iter_add = AST::CallMethod {
                        object: AST::AccessVariable {
                            name: Identifier(iter_var_name.clone()),
                        }
                        .into_boxed(),
                        name: Identifier("+".to_string()),
                        arguments: vec![AST::Integer(1).into_boxed()],
                    }
                    .into_boxed();
                    let iter_update = AST::AssignVariable { name: Identifier(iter_var_name.clone()), value: iter_add }.into_boxed();

                    // while (i < size)
                    //  arr[i] = value;
                    //  i <- i + 1
                    let init_loop = AST::Loop {
                        condition: AST::CallMethod {
                            object: AST::AccessVariable {
                                name: Identifier(iter_var_name.clone()),
                            }
                            .into_boxed(),
                            name: Identifier("<".to_string()),
                            arguments: vec![AST::AccessVariable {
                                name: Identifier(size_var_name.clone()),
                            }
                            .into_boxed()],
                        }
                        .into_boxed(),
                        body: AST::Block(vec![assign, iter_update]).into_boxed(),
                    }.into_boxed();

                    _compile(&init_loop, pool, code, frame, globals, global_env, generator, true)?;

                    let array_access = AST::AccessVariable { name: Identifier(array_var_name.clone()) };

                    _compile(&array_access, pool, code, frame, globals, global_env, generator, drop)?;

                    Ok(())

                }
            }
        }
        AST::Object { extends, members } => {
            _compile(
                extends, pool, code, frame, globals, global_env, generator, false,
            )?;

            // Compile the members and save the members as constant pool indexes
            // TODO: Probably return vec of results and then filter it
            let indexes: Vec<ConstantPoolIndex> = members
                .iter()
                .map(|ast| {
                    // Love me some stars
                    match &**ast {
                        AST::Function {
                            name,
                            parameters,
                            body,
                        } => compile_fun_def(
                            name.0.clone(),
                            parameters,
                            body,
                            true,
                            pool,
                            globals,
                            global_env,
                            generator,
                        )
                        .expect("Compilation of method definition failed"),
                        AST::Variable { name, value } => {
                            _compile(
                                &value, pool, code, frame, globals, global_env, generator, false,
                            )
                            .expect("Compilation failed");
                            let str_idx = pool.push(Constant::from(name.0.clone()));
                            let idx = pool.push(Constant::Slot { name: str_idx });
                            idx
                        }
                        _ => panic!("Object definition can only have method or variable."),
                    }
                })
                .collect();

            let obj = pool.push(Constant::Object { members: indexes });
            code.write_inst(Bytecode::Object { class: obj });

            Ok(())
        }
        AST::AccessVariable { name } => {
            match frame {
                Frame::Local(env) if env.has_variable(&name.0).is_some() => {
                    let idx = env
                        .has_variable(&name.0)
                        .unwrap_or_else(|| panic!("Variable '{}' is not defined.", &name.0));
                    code.write_inst(Bytecode::GetLocal { index: idx });
                }
                // In global scope but local because used in block
                Frame::Top
                    if !global_env.is_topmost() && global_env.has_variable(&name.0).is_some() =>
                {
                    let idx = global_env
                        .has_variable(&name.0)
                        .expect("Variable is not defined.");
                    code.write_inst(Bytecode::GetLocal { index: idx });
                }
                // Global variable
                _ => {
                    let idx = pool.push(Constant::from(name.0.clone()));
                    // TODO: Check if global exists. It can't be done this way,
                    // because they are saved as slots.
                    // if !globals.contains(idx) {
                    //     panic!("Global variable '{}' doesn't exist.", &name.0);
                    // }
                    code.write_inst(Bytecode::GetGlobal { name: idx });
                }
            };
            Ok(())
        }
        AST::AccessField { object, field } => {
            let field_idx = pool
                .find_by_str(&field.0)
                .unwrap_or_else(|| panic!("Field '{}' does not exist", field.0));
            // let slot_idx = pool.find(&Constant::Slot { name: field_idx }).expect("Slot doesn't exist");
            _compile(
                object, pool, code, frame, globals, global_env, generator, drop,
            )?;
            code.write_inst(Bytecode::GetField { name: field_idx });

            Ok(())
        }
        AST::AccessArray { array, index } => {
            _compile(
                array, pool, code, frame, globals, global_env, generator, false,
            )?;
            _compile(
                index, pool, code, frame, globals, global_env, generator, false,
            )?;

            let access_idx = pool.push(Constant::from(String::from("get")));
            code.write_inst(Bytecode::CallMethod {
                name: access_idx,
                arguments: 2,
            });
            Ok(())
        }
        AST::AssignVariable { name, value } => {
            _compile(
                value, pool, code, frame, globals, global_env, generator, false,
            )?;
            match frame {
                Frame::Local(env) if env.has_variable(&name.0).is_some() => {
                    let idx = env.has_variable(&name.0).unwrap();
                    code.write_inst(Bytecode::SetLocal { index: idx });
                }
                Frame::Top
                    if !global_env.is_topmost() && global_env.has_variable(&name.0).is_some() =>
                {
                    let idx = global_env.has_variable(&name.0).unwrap();
                    code.write_inst(Bytecode::SetLocal { index: idx });
                }
                _ => {
                    let idx = pool.push(Constant::from(name.0.clone()));
                    // TODO: Check if global exists. It can't be done this way,
                    // because they are saved as slots.
                    // if !globals.contains(idx) {
                    //     panic!("Global variable '{}' doesn't exist.", &name.0);
                    // }
                    code.write_inst(Bytecode::SetGlobal { name: idx });
                }
            }
            // AssignVariable only peeks, thats why it might be necessary to drop the value
            code.write_inst_if(Bytecode::Drop, drop);
            Ok(())
        }
        AST::AssignField {
            object,
            field,
            value,
        } => {
            let field_idx = pool
                .find_by_str(&field.0)
                .expect("Given field does not exist");
            // let slot_idx = pool.find(&Constant::Slot { name: field_idx }).expect("Slot doesn't exist");
            _compile(
                object, pool, code, frame, globals, global_env, generator, false,
            )?;
            _compile(
                value, pool, code, frame, globals, global_env, generator, false,
            )?;
            code.write_inst(Bytecode::SetField { name: field_idx });
            Ok(())
        }
        AST::AssignArray {
            array,
            index,
            value,
        } => {
            _compile(
                array, pool, code, frame, globals, global_env, generator, false,
            )?;
            _compile(
                index, pool, code, frame, globals, global_env, generator, false,
            )?;
            _compile(
                value, pool, code, frame, globals, global_env, generator, false,
            )?;

            let access_idx = pool.push(Constant::from(String::from("set")));
            code.write_inst(Bytecode::CallMethod {
                name: access_idx,
                arguments: 3,
            });
            Ok(())
        }
        AST::Function {
            name,
            parameters,
            body,
        } => {
            // Frame must be top
            if matches!(frame, Frame::Local(_)) {
                return Err("Functions can't be nested");
            }
            let func = compile_fun_def(name.0.clone(), parameters, body, false, pool, globals, global_env, generator)?;
            globals.introduce_variable(func);

            Ok(())
        }
        AST::CallFunction { name, arguments } => {
            // let fun_idx = pool
            //     .find_by_str(&name.0)
            //     .unwrap_or_else(|| panic!("Function '{}' does not exist.", &name.0));
            let fun_idx = pool.push(Constant::from(name.0.clone()));
            for ast in arguments {
                _compile(
                    ast, pool, code, frame, globals, global_env, generator, false,
                )?;
            }
            code.write_inst(Bytecode::CallFunction {
                name: fun_idx,
                arguments: arguments.len().try_into().unwrap(),
            });
            Ok(())
        }
        AST::CallMethod {
            object,
            name,
            arguments,
        } => {
            // let method_idx = pool.find_by_str(&name.0).expect("Called method does not exist.");
            let method_idx = pool.push(Constant::from(name.0.clone()));
            // Push object first and then the arguments.
            _compile(
                object, pool, code, frame, globals, global_env, generator, false,
            )?;
            for ast in arguments {
                _compile(
                    ast, pool, code, frame, globals, global_env, generator, false,
                )?;
            }
            code.write_inst(Bytecode::CallMethod {
                name: method_idx,
                arguments: (arguments.len() + 1).try_into().unwrap(),
            });

            Ok(())
        }
        // Here, global statements or functions definitions are
        AST::Top(asts) => {
            // Create the 'main' function
            let mut code_main = Code::new();

            for ast in asts.iter() {
                // We send here code_main even if new function is encountered,
                // but that function will define it's own code vector anyway.
                _compile(
                    ast,
                    pool,
                    &mut code_main,
                    &mut Frame::Top,
                    globals,
                    global_env,
                    generator,
                    true,
                )?;
            }

            // println!("{:?}", global_env);
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
                _compile(
                    ast,
                    pool,
                    code,
                    frame,
                    globals,
                    global_env,
                    generator,
                    it.peek().is_some() && !drop,
                )?;
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
        }
        AST::Loop { condition, body } => {
            let label_begin = pool.push(Constant::from(generator.generate("while_begin")));
            let label_cond = pool.push(Constant::from(generator.generate("while_cond")));

            // Since there is no instruction for negation, we need to evaluate the condition at the end
            // if it's false, we just fall through.
            code.write_inst(Bytecode::Jump { label: label_cond });

            // Body
            code.write_inst(Bytecode::Label { name: label_begin });
            _compile(
                body, pool, code, frame, globals, global_env, generator, drop,
            )?;

            // Condition
            code.write_inst(Bytecode::Label { name: label_cond });
            _compile(
                condition, pool, code, frame, globals, global_env, generator, false,
            )?;
            code.write_inst(Bytecode::Branch { label: label_begin });

            Ok(())
        }
        AST::Conditional {
            condition,
            consequent,
            alternative,
        } => {
            let label_then = pool.push(Constant::from(generator.generate("if_then")));
            let label_else = pool.push(Constant::from(generator.generate("if_else")));
            let label_merge = pool.push(Constant::from(generator.generate("if_merge")));

            _compile(
                condition, pool, code, frame, globals, global_env, generator, false,
            )?;
            code.write_inst(Bytecode::Branch { label: label_then });
            code.write_inst(Bytecode::Jump { label: label_else });

            // Then body
            code.write_inst(Bytecode::Label { name: label_then });
            _compile(
                consequent, pool, code, frame, globals, global_env, generator, false,
            )?;
            code.write_inst(Bytecode::Jump { label: label_merge });

            // Else body
            code.write_inst(Bytecode::Label { name: label_else });
            _compile(
                alternative,
                pool,
                code,
                frame,
                globals,
                global_env,
                generator,
                false,
            )?;

            // Merge label
            code.write_inst(Bytecode::Label { name: label_merge });

            Ok(())
        }
        AST::Print { format, arguments } => {
            let string = pool.push(Constant::from(format.clone()));
            for ast in arguments.iter() {
                _compile(
                    ast, pool, code, frame, globals, global_env, generator, false,
                )?;
            }
            let print = Bytecode::Print {
                format: string,
                arguments: arguments.len().try_into().unwrap(),
            };
            code.write_inst(print);
            code.write_inst_if(Bytecode::Drop, drop);
            Ok(())
        }
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
