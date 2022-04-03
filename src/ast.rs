use std::fmt::Debug;
use std::cmp::PartialEq;
use serde::{Serialize, Deserialize};

#[derive(PartialEq,Debug,Serialize,Deserialize,Clone)]
pub enum AST {
    Integer(i32),
    Boolean(bool),
    Null,

    Variable { name: Identifier, value: Box<AST> },
    Array { size: Box<AST>, value: Box<AST> },
    Object { extends: Box<AST>, members: Vec<Box<AST>> },

    AccessVariable { name: Identifier },
    AccessField { object: Box<AST>, field: Identifier },
    AccessArray { array: Box<AST>, index: Box<AST> },

    AssignVariable { name: Identifier, value: Box<AST> },
    AssignField { object: Box<AST>, field: Identifier, value: Box<AST> },
    AssignArray { array: Box<AST>, index: Box<AST>, value: Box<AST> },

    Function { name: Identifier, parameters: Vec<Identifier>, body: Box<AST> },
    //Operator { operator: Operator, parameters: Vec<Identifier>, body: Box<AST> },    // TODO Consider merging with function

    CallFunction { name: Identifier, arguments: Vec<Box<AST>> },
    CallMethod { object: Box<AST>, name: Identifier, arguments: Vec<Box<AST>> },
    //CallOperator { object: Box<AST>, operator: Operator, arguments: Vec<Box<AST>> }, // TODO Consider removing
    //Operation { operator: Operator, left: Box<AST>, right: Box<AST> },               // TODO Consider removing

    Top (Vec<Box<AST>>),
    Block (Vec<Box<AST>>),
    Loop { condition: Box<AST>, body: Box<AST> },
    Conditional { condition: Box<AST>, consequent: Box<AST>, alternative: Box<AST> },

    Print { format: String, arguments: Vec<Box<AST>> },
}
