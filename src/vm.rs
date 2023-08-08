#![allow(non_camel_case_types)]

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    ast::{Expression, Literal, Symbol},
    chunk::{Chunk, OpCode},
    compiler::Compiler,
    parser::parse,
    value::{Function, Value},
};

pub enum InterpretError {
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

pub struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    starting_slot: usize,
}

#[derive(Default)]
pub struct VM {
    frames: Vec<CallFrame>,
    slots: Vec<Rc<Value>>,
    globals: HashMap<String, Rc<Value>>,
    strings: HashSet<Rc<String>>,
    objects: Vec<Rc<Value>>,
}

impl VM {
    fn run(&mut self) -> Option<()> {
        while let Some(frame) = self.frames.last() {
            match (*frame.function).chunk.codes.get(frame.ip)? {
                OpCode::Constant(pos) => {
                    self.slots
                        .push((frame.function.chunk.constants.get(*pos)?).clone());
                }
                OpCode::Return => {
                    self.frames.pop();
                }
                OpCode::Tail => {
                    self.tail();
                }
                OpCode::GetLocal(pos) => {
                    self.get_local();
                }
                _ => unimplemented!(),
            }
            self.step()?;
        }
        Some(())
    }

    fn step(&mut self) -> Option<()> {
        let frame = self.frames.last_mut()?;
        if frame.ip + 1 >= frame.function.chunk.codes.len() {
            self.frames.pop();
        } else {
            frame.ip += 1;
        }
        Some(())
    }

    fn tail(&mut self) -> &mut Self {
        self
    }

    fn get_local(&mut self) -> &mut Self {
        self
    }

    fn push_frame(&mut self, function: Rc<Function>) -> &mut Self {
        self.frames.push(CallFrame {
            function,
            ip: 0,
            starting_slot: self.slots.len(),
        });
        self
    }

    fn compile(&mut self, string: &str) -> Vec<Compiler> {
        let mut compile_stack = vec![Compiler::new()];
        let (_, exps) = parse(string).unwrap();
        exps.into_iter().for_each(|e| {
            self.compile_expression(&mut compile_stack, e);
        });
        compile_stack
    }

    fn intern_string(&mut self, string: String) -> Rc<String> {
        self.strings.get_or_insert(Rc::new(string)).clone()
    }

    fn create_value_from_literal(&mut self, literal: Literal) -> Value {
        match literal {
            Literal::Nil => Value::Nil,
            Literal::Number(n) => Value::Number(n),
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::String(s) => Value::String(self.intern_string(s)),
        }
    }
    fn compile_expression(
        &mut self,
        compile_stack: &mut Vec<Compiler>,
        expression: Expression,
    ) -> Option<()> {
        match expression {
            Expression::Literal(l) => {
                let value = self.create_value_from_literal(l);
                let c = compile_stack.last_mut()?;
                c.chunk.add_constant(value);
            }
            _ => unimplemented!(),
            // Expression::Call(c) => {}
            // Expression::Parenthesised(p) => {}
            // Expression::Cond(c) => {}
            // Expression::Block(b) => {}
            // Expression::Assignment(a) => {}
            // Expression::Function(f) => {}

            // Expression::Symbol(s) => {}
        };
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::*;

    #[test]
    fn test_literal_works() {
        let mut vm = VM::default();

        let c_s = vm.compile("1;2;3;4");
	assert_eq!(c_s, vec![]);
    }
}
