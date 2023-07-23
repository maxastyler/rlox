#![allow(non_camel_case_types)]
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::value::Value;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    OP_RETURN,
    OP_CONSTANT,
}

#[derive(Default)]
pub struct Chunk {
    pub codes: Vec<u8>,
    pub values: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn add_constant(&mut self, value: Value) -> usize {
        let index = self.values.len();
        self.values.push(value);
        index
    }

    pub fn write_chunk(&mut self, byte: u8, line: usize) {
        self.codes.push(byte);
        self.lines.push(line);
    }
}
