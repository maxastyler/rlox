#![allow(non_camel_case_types)]
use std::{
    ops::{Add, Div, Mul, Sub},
    ptr::null,
};

use crate::{
    chunk::{Chunk, OpCode},
    compile::{self, compile},
    debug::{disassemble_chunk, disassemble_instruction},
    value::{print_value, Value},
};

pub enum InterpretError {
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    fn read_constant(&mut self) -> Value {
        let constant = self.chunk.values[self.chunk.codes[self.ip] as usize];
        self.ip += 1;
        constant
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            print!("          ");
            self.stack.iter().for_each(|v| {
                print!("[ ");
                print_value(v);
                print!(" ]");
            });
            println!();
            disassemble_instruction(&self.chunk, self.ip);

            let instruction = self.chunk.codes[self.ip];
            self.ip += 1;
            if let Ok(c) = OpCode::try_from(instruction) {
                match c {
                    OpCode::OP_CONSTANT => {
                        let constant = self.read_constant();
                        self.push(constant);
                    }
                    OpCode::OP_NEGATE => {
                        let value = -self.pop();
                        self.push(value);
                    }
                    OpCode::OP_RETURN => {
                        print_value(&self.pop());
                        println!();
                        return Ok(());
                    }
                    OpCode::OP_ADD => self.binop(Add::add),
                    OpCode::OP_SUBTRACT => self.binop(Sub::sub),
                    OpCode::OP_MULTIPLY => self.binop(Mul::mul),
                    OpCode::OP_DIVIDE => self.binop(Div::div),
                }
            } else {
                panic!()
            }
        }
    }

    fn binop(&mut self, op: fn(a: Value, b: Value) -> Value) {
        let vb = self.pop();
        let va = self.pop();
        self.push(op(va, vb));
    }

    pub fn new(chunk: Chunk) -> VM {
        VM {
            ip: 0,
            chunk,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }

    fn push(&mut self, value: Value) {
        if self.stack.len() == STACK_MAX {
            panic!("STACK OVERFLOW")
        } else {
            self.stack.push(value);
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn interpret(&mut self, source: &String) -> Result<(), InterpretError> {
        let mut chunk = Chunk::default();
        if compile(&source, &mut chunk) {
            let mut vm = VM::new(chunk);
            return vm.run();
        } else {
            return Err(InterpretError::INTERPRET_COMPILE_ERROR);
        }
    }
}
