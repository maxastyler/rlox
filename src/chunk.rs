use std::rc::Rc;

use crate::value::Value;

#[derive(Debug, PartialEq)]
pub enum OpCode {
    Constant(usize),
    Return,
    Pop,
    GetLocal(usize),
    GetGlobal(Rc<String>),
    SetLocal(usize, Value),
    SetGlobal(Rc<String>, Value),
    CopyToTopFromStack(usize),
    AssignToSlot(usize),
    CreateSlot,
}

#[derive(Default, Debug, PartialEq)]
pub struct Chunk {
    pub codes: Vec<OpCode>,
    pub constants: Vec<Rc<Value>>,
}

impl Chunk {
    pub fn add_constant(&mut self, value: Value) -> &mut Self {
        let index = self.constants.len();
        self.constants.push(Rc::new(value));
        self.codes.push(OpCode::Constant(index));
        self
    }

    pub fn add_pop(&mut self) {
        self.codes.push(OpCode::Pop)
    }
}
