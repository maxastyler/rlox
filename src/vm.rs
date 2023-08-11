#![allow(non_camel_case_types)]

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    ast::{Assignment, Expression, Literal, Symbol},
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
        compiler: &mut Compiler,
        expression: Expression,
    ) -> Option<()> {
        match expression {
            Expression::Literal(l) => {
                let value = self.create_value_from_literal(l);
                compile_stack.last_mut()?.add_constant(value);
            }
            Expression::Ignored(e) => {
                self.compile_expression(compile_stack, *e);
                compile_stack.last_mut()?.pop();
            }
            Expression::Parenthesised(p) => {
                self.contained_expression_list(compile_stack, p.0);
            }
            Expression::Block(b) => {
                compile_stack.last_mut()?.enter_block();
                self.contained_expression_list(compile_stack, b.0);
                compile_stack.last_mut()?.exit_block();
            }
            Expression::Assignment(a) => {
                self.assign(compile_stack, *a);
            }
            _ => unimplemented!(),
            // Expression::Call(c) => {}
            // Expression::Parenthesised(p) => {}
            // Expression::Cond(c) => {}
            // Expression::Function(f) => {}
            // Expression::Symbol(s) => {}
        };
        Some(())
    }

    fn assign(&mut self, compile_stack: &mut Vec<Compiler>, assignment: Assignment) -> Option<()> {
        let id_name = self.intern_string(assignment.identifier.0);
        let current_compiler = compile_stack.last()?;
        if let Some(local) = current_compiler
            .locals
            .iter()
            .enumerate()
            .rev()
            .find(|&x| Rc::ptr_eq(&id_name, &x.1.name))
        {
            current_compiler.depth;
        } else {
        }

        Some(())
    }

    fn contained_expression_list(
        &mut self,
        compile_stack: &mut Vec<Compiler>,
        expressions: Vec<Expression>,
    ) -> Option<()> {
        let empty_expression = expressions.is_empty()
            | expressions
                .last()
                .is_some_and(|x| matches!(x, Expression::Ignored(_)));
        expressions.into_iter().for_each(|e| {
            self.compile_expression(compile_stack, e);
        });
        if empty_expression {
            compile_stack.last_mut()?.add_constant(Value::Nil);
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

        let c_s = vm.compile("-1;2\n\n;3;4");
        assert_eq!(
            c_s.last().unwrap().chunk.codes,
            vec![
                OpCode::Constant(0),
                OpCode::Pop,
                OpCode::Constant(1),
                OpCode::Pop,
                OpCode::Constant(2),
                OpCode::Pop,
                OpCode::Constant(3)
            ]
        );
    }

    #[test]
    fn test_parens_work() {
        let mut vm = VM::default();

        let c_s = vm.compile("((-1;2;)\n\n;3);4;");
        assert_eq!(
            c_s.last().unwrap().chunk.codes,
            vec![
                OpCode::Constant(0),
                OpCode::Pop,
                OpCode::Constant(1),
                OpCode::Pop,
                OpCode::Constant(2),
                OpCode::Pop,
                OpCode::Constant(3),
                OpCode::Pop,
                OpCode::Constant(4),
                OpCode::Pop
            ]
        );
    }

    #[test]
    fn test_braces_work() {
        let mut vm = VM::default();

        let c_s = vm.compile("({-1;2;}\n\n;3);4;");
        assert_eq!(
            c_s.last().unwrap().chunk.codes,
            vec![
                OpCode::Constant(0),
                OpCode::Pop,
                OpCode::Constant(1),
                OpCode::Pop,
                OpCode::Constant(2),
                OpCode::Pop,
                OpCode::Constant(3),
                OpCode::Pop,
                OpCode::Constant(4),
                OpCode::Pop
            ]
        );
    }
}
