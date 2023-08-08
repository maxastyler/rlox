#![allow(non_camel_case_types)]
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::value::Value;

pub enum OpCode<'a> {
    Constant(usize),
    Return,
    Tail,
    GetLocal(usize),
    GetGlobal(&'a str),
    SetLocal(usize, Value),
    SetGlobal(&'a str, Value),
}

#[derive(Default)]
pub struct Chunk<'a> {
    pub codes: Vec<OpCode<'a>>,
    pub constants: Vec<Value>,
}

impl<'a> Chunk<'a> {
    fn add_constant(&mut self, value: Value) -> &mut Self {
        let index = self.constants.len();
        self.constants.append(value);
        self.codes.append(OpCode::Constant(index));
        self
    }
}
