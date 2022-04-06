pub mod bytecode;
pub mod constants;
pub mod ast;
pub mod compiler;
pub mod serializer;

use ast::AST;
use compiler::compile;
use std::env;
use std::fs;

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let program = fs::read_to_string(&args[2]);
    let tree: AST = serde_json::from_str(&program.unwrap()).unwrap();
    compile(&tree)?;
    Ok(())
}
