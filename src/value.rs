use std::rc::Rc;

use crate::chunk::Chunk;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Rc<String>,
}

#[derive(Debug, PartialEq)]
pub struct UpValue {
    pub value: Rc<Value>,
}

#[derive(Debug, PartialEq)]
pub enum Object {
    Function(Rc<Function>),
    UpValue(Rc<UpValue>),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    String(Rc<String>),
    Symbol(Rc<String>),
    Object(Object),
}
