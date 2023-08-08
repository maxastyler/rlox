use crate::{
    ast::{Expression, Literal, Symbol},
    chunk::Chunk,
    parser::parse,
    value::Value,
};

#[derive(Debug, PartialEq)]
pub struct Local {
    name: Symbol,
    depth: usize,
}

#[derive(Debug, PartialEq)]
pub struct Compiler {
    pub depth: usize,
    pub locals: Vec<Local>,
    pub locals_count: usize,
    pub chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            depth: 0,
            locals_count: 0,
            locals: vec![],
            chunk: Chunk::default(),
        }
    }
    pub fn enter_block(&mut self) {
        self.depth += 1;
    }

    pub fn exit_block(&mut self) {
        self.depth -= 1;
    }
}
