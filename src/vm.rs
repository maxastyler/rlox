#![allow(non_camel_case_types)]
use std::collections::{HashMap, HashSet};

use crate::{
    chunk::{Chunk, OpCode},
    compile::{self, compile},
    value::Value,
};

pub enum InterpretError {
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

#[derive(Default)]
pub struct VM<'a> {
    chunk_positions: Vec<(Chunk<'a>, usize)>,
    stack: Vec<&'a Value<'a>>,
    globals: HashMap<String, Value<'a>>,
    strings: HashSet<String>,
    objects: Vec<Value<'a>>,
}

impl<'a> VM<'a> {
    fn run(&mut self) -> Option<()> {
        while let Some((chunk, pos)) = self.chunk_positions.last_mut() {
            match chunk.codes.get(*pos)? {
                OpCode::Constant(pos) => {
                    self.stack.push(chunk.constants.get(*pos)?);
                }
                OpCode::Return => {
                    self.chunk_positions.pop();
                }
                OpCode::Tail => {
                    self.tail();
                }
                OpCode::GetLocal(pos) => {
                    self.get_local();
                }
                _ => unimplemented!(),
            };

            if *pos + 1 >= chunk.codes.len() {
                self.chunk_positions.pop();
            } else {
                *pos += 1;
            }
        }
        None
    }

    fn tail(&mut self) -> &mut Self {
        self
    }

    fn get_local(&mut self) -> &mut Self {
        self
    }

    fn push_chunk(&mut self, chunk: Chunk) -> &mut Self {
        self.chunk_positions.push((chunk, 0));
        self
    }

    pub fn interpret(&mut self, source: &String) -> Result<(), InterpretError> {
        let mut chunk = Chunk::default();
        if compile(&source, &mut chunk) {
            let mut vm: VM = Default::default();
            return Ok({
                vm.run();
                ()
            });
        } else {
            return Err(InterpretError::INTERPRET_COMPILE_ERROR);
        }
    }
}
