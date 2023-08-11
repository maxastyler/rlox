use std::{borrow::BorrowMut, cell::RefCell, collections::HashSet, fmt::Arguments, rc::Rc};

use crate::{
    ast::{Assignment, Block, Expression, Function, Literal, Symbol},
    chunk::{Chunk, OpCode},
    value::{self, Object, Value},
};

#[derive(Debug)]
pub struct Identity {
    name: Rc<String>,
}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.name, &other.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Local {
    pub identity: Identity,
    pub depth: usize,
    pub captured: bool,
}

#[derive(Debug, PartialEq)]
pub struct UpValue {
    pub index: usize,
    pub local: bool,
}

pub struct FunctionCompiler {
    pub locals: Vec<Local>,
    pub depth: usize,
}

impl FunctionCompiler {}

#[derive(Debug, PartialEq)]
pub struct Compiler {
    pub locals: Vec<Local>,
    pub upvalues: Vec<UpValue>,
    pub depth: usize,
    pub offset: usize,
    pub strings: HashSet<Rc<String>>,
    pub previous: Option<Box<Compiler>>,
    pub chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            locals: vec![],
            upvalues: vec![],
            depth: 0,
            offset: 0,
            strings: HashSet::new(),
            previous: None,
            chunk: Chunk::default(),
        }
    }

    fn intern_symbol(&mut self, symbol: &Symbol) -> Rc<String> {
        self.intern_string(&symbol.0)
    }

    fn intern_string(&mut self, string: &String) -> Rc<String> {
        self.strings.get_or_insert(Rc::new(string.clone())).clone()
    }

    fn pop_function(self, function: &Function) -> Option<(Compiler, value::Function)> {
        // Do popping stuff
        let mut c = *(self.previous?);
        c.strings = self.strings;
        Some((
            c,
            value::Function {
                arity: function.arguments.len(),
                chunk: self.chunk,
            },
        ))
    }

    fn find_local(&self, id: &Identity) -> Option<usize> {
        self.locals.iter().enumerate().rev().find_map(|(i, x)| {
            if x.identity == *id {
                Some(i)
            } else {
                None
            }
        })
    }

    fn find_nonlocal(&mut self, id: &Identity) -> Option<usize> {
        if let Some(x) = self.previous.as_mut() {
            if let Some(i) = x.find_local(id) {
                // This captures a local variable in the surrounding environment
                x.locals[i].captured = true;
                Some(self.add_upvalue(UpValue {
                    index: i,
                    local: true,
                }))
            } else if let Some(i) = x.find_nonlocal(id) {
                // There is no local variable in the surrounding environment, search higher
                Some(self.add_upvalue(UpValue {
                    index: i,
                    local: false,
                }))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn add_upvalue(&mut self, upvalue: UpValue) -> usize {
        if let Some(p) = self.upvalues.iter_mut().position(|x| *x == upvalue) {
            p
        } else {
            self.upvalues.push(upvalue);
            self.upvalues.len() - 1
        }
    }

    fn generate_symbol(&mut self, symbol: &Symbol) -> Option<()> {
        let s = self.intern_symbol(symbol);
        let identity = Identity { name: s };
        if let Some(index) = self.find_local(&identity) {
            self.chunk.codes.push(OpCode::GetLocal(index));
            Some(())
        } else if let Some(index) = self.find_nonlocal(&identity) {
            // try to find the value in an enclosing context
            self.chunk.codes.push(OpCode::GetUpValue(index));
            Some(())
        } else {
            None
        }
    }

    fn add_symbol_to_locals(&mut self, symbol: &Symbol) {
        let s = self.intern_symbol(&symbol);
        self.locals.push(Local {
            identity: Identity { name: s },
            captured: false,
            depth: self.depth,
        });
    }

    fn generate_assignment(mut self, assignment: &Assignment) -> Option<Self> {
        self.add_symbol_to_locals(&assignment.identifier);
        self.compile_expression(&assignment.value) // result of the expression will be on the stack
    }

    fn generate_literal(&mut self, literal: &Literal) {
        let value = self.create_value_from_literal(literal.clone());
        self.chunk.add_constant(value);
    }

    fn pop_value(&mut self) {
        self.chunk.add_pop()
    }

    fn generate_ignored(self, expression: &Expression) -> Option<Self> {
        let mut c = self.compile_expression(expression)?;
        c.pop_value();
        Some(c)
    }

    fn create_value_from_literal(&mut self, literal: Literal) -> Value {
        match literal {
            Literal::Nil => Value::Nil,
            Literal::Number(n) => Value::Number(n),
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::String(s) => Value::String(self.intern_string(&s)),
        }
    }

    pub fn compile_expression(mut self, expression: &Expression) -> Option<Self> {
        match expression {
            Expression::Assignment(assignment) => self.generate_assignment(assignment),
            Expression::Literal(l) => {
                self.generate_literal(l);
                Some(self)
            }
            Expression::Block(block) => self.compile_block(block),
            Expression::Symbol(sym) => {
                self.generate_symbol(sym);
                Some(self)
            }
            Expression::Ignored(e) => {
                let c = self.generate_ignored(e)?;
                Some(c)
            }
            Expression::Function(f) => {
                let c = self.generate_function(f)?;
                Some(c)
            }

            _ => unimplemented!(),
        }
    }

    pub fn compile_block(mut self, block: &Block) -> Option<Self> {
        self.depth += 1;
        let mut c = self;

        for e in block.0.iter() {
            c = c.compile_expression(e).unwrap();
        }
        if block.0.len() == 0 || matches!(block.0.last().unwrap(), Expression::Ignored(_)) {
            c.generate_literal(&Literal::Nil);
        }
        c.depth -= 1;
        Some(c)
    }

    pub fn generate_function(self, function: &Function) -> Option<Self> {
        let mut new_compiler = Compiler {
            upvalues: vec![],
            depth: self.depth + 1,
            locals: vec![],
            strings: self.strings.clone(),
            offset: self.offset + self.locals.len(),
            previous: Some(Box::new(self)),
            chunk: Chunk::default(),
        };

        for s in function.arguments.iter() {
            new_compiler.add_symbol_to_locals(&s);
        }

        new_compiler = new_compiler.compile_block(&function.block)?;

        let (mut old_compiler, compiled_fun) = new_compiler.pop_function(function)?;
        let value = Value::Object(Object::Function(Rc::new(compiled_fun)));
        let constant_function_index = old_compiler.chunk.add_constant(value);
	
        if let Some(id) = function.identifier.clone() {
            // we need to assign the function to the given identifier
            old_compiler.add_symbol_to_locals(&id);
        }

        Some(old_compiler)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::*;

    fn ass_lit(string: &str, lit: Literal) -> Expression {
        Expression::Assignment(Box::new(Assignment {
            identifier: Symbol(string.into()),
            value: Expression::Literal(lit),
        }))
    }

    fn get(string: &str) -> Expression {
        Expression::Symbol(Symbol(string.into()))
    }

    fn block(es: Vec<Expression>) -> Expression {
        Expression::Block(Box::new(Block(es)))
    }

    #[test]
    fn test_something() {
        let mut c = Compiler::new();
        let (_, e) = parse("fn x (a, b) {fn y () {a};}").unwrap();
        c = c.compile_expression(&e[0]).unwrap();
        assert_eq!(c, Compiler::new());
    }
}
