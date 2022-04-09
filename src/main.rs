pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod constants;
pub mod debug;
pub mod serializer;

use ast::AST;
use compiler::compile;
use std::env;
use std::fs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Usage: fml command file");
    }

    if args[1] == "compile" {
        let program = fs::read_to_string(&args[2]).unwrap();
        let tree: AST = serde_json::from_str(&program).unwrap();
        compile(&tree)
    } else {
        panic!("Following commands are supported: compile")
    }
}
