use std::{collections::HashSet, rc::Rc};

use crate::{
    ast::{Assignment, Expression, Function, Literal, Symbol},
    chunk::{Chunk, OpCode},
    parser::parse,
    value::{UpValue, Value},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Local {
    pub name: Rc<String>,
    pub depth: usize,
}

#[derive(Debug, PartialEq)]
pub struct SubCompiler {
    pub locals: Vec<Local>,
    pub chunk: Chunk,
    pub name: Rc<String>,
    pub depth: usize,
    pub upvalues: Vec<UpValue>,
    pub offset: usize,
}

impl SubCompiler {
    pub fn new(call_depth: usize, name: Rc<String>) -> Self {
        SubCompiler {
            offset: 0,
            locals: vec![],
            depth: call_depth + 1,
            name,
            chunk: Chunk::default(),
            upvalues: vec![],
        }
    }

    pub fn enter_block(&mut self) {
        self.depth += 1;
    }

    pub fn exit_block(&mut self) {
        self.depth -= 1;
    }

    pub fn add_pop(&mut self) {
        self.chunk.add_pop();
    }

    pub fn get_from_stack(&mut self, position: usize) {
        self.chunk.codes.push(OpCode::CopyToTopFromStack(position));
    }

    pub fn add_constant(&mut self, constant: Value) {
        self.chunk.add_constant(constant);
    }

    pub fn add_assign_to_slot(&mut self, slot_index: usize) {
        self.chunk.codes.push(OpCode::AssignToSlot(slot_index));
    }

    pub fn add_create_slot(&mut self) {
        self.chunk.codes.push(OpCode::CreateSlot)
    }

    pub fn create_new_local(&mut self, name: Rc<String>) {
        self.add_create_slot();
        self.locals.push(Local {
            depth: self.depth,
            name: name,
        })
    }

    /// Get the index of the last local that had this symbol
    pub fn last_declaration(&self, symbol: Symbol) -> Option<(usize, &Local)> {
        self.locals.iter().enumerate().rev().find_map(|(i, x)| {
            if *x.clone().name == symbol.0 {
                Some((i, x))
            } else {
                None
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Compiler {
    pub sub_compilers: Vec<(usize, SubCompiler)>,
    pub strings: HashSet<Rc<String>>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut c = Compiler {
            strings: HashSet::new(),
            sub_compilers: vec![],
        };
        let s = c.intern_string("MAIN".into());
        c.push_compiler(s);
        c
    }

    fn intern_string(&mut self, string: String) -> Rc<String> {
        self.strings.get_or_insert(Rc::new(string)).clone()
    }

    pub fn enter_block(&mut self) {
        if let Some((_, sc)) = self.sub_compilers.last_mut() {
            sc.enter_block()
        }
    }

    pub fn exit_block(&mut self) {
        if let Some((_, sc)) = self.sub_compilers.last_mut() {
            sc.exit_block()
        }
    }

    pub fn add_constant(&mut self, value: Value) {
        if let Some((_, sc)) = self.sub_compilers.last_mut() {
            sc.add_constant(value)
        }
    }

    pub fn expression(&mut self, expression: &Expression) -> Option<()> {
        None
    }

    pub fn add_assignment(&mut self, assignment: &Assignment) -> Option<()> {
        self.expression(&assignment.value);
        if let Some((i, l)) = self.last_declaration(&assignment.identifier) {
            if l.depth == self.sub_compilers.last_mut()?.1.depth {
                // The assignment is at the same depth as the previous assignment
                // to this symbol, assign in its place
                self.sub_compilers.last_mut()?.1.add_assign_to_slot(i);
                return None;
            }
        }
        let s = self.intern_string(assignment.identifier.0.clone());
        self.sub_compilers.last_mut()?.1.create_new_local(s);
        None
    }

    pub fn push_compiler(&mut self, name: Rc<String>) {
        let new_sc = if let Some(sc) = self.sub_compilers.last() {
            let pos = sc.0 + sc.1.locals.len();
            (pos, SubCompiler::new(sc.1.depth, name))
        } else {
            (0, SubCompiler::new(0, name))
        };
        self.sub_compilers.push(new_sc);
    }

    pub fn compile_function(&mut self, function: Function) {}

    /// Get the index of the last declaration
    pub fn last_declaration(&self, symbol: &Symbol) -> Option<(usize, &Local)> {
        self.sub_compilers.iter().rev().find_map(|(i, sc)| {
            sc.last_declaration(symbol.clone())
                .and_then(|(j, l)| Some((j + i, l)))
        })
    }

    fn create_upvalue(&mut self, symbol: &Symbol) -> Option<()> {
        None
    }

    pub fn read_local(&mut self, symbol: &Symbol) -> Option<()> {
        if let Some((i, _)) = self
            .sub_compilers
            .last()?
            .1
            .last_declaration(symbol.clone())
        {
            // The last declaration was in this function
            // Just need to put it onto the top of the stack
            let pos = self.sub_compilers.last()?.0;
            self.sub_compilers.last_mut()?.1.get_from_stack(i + pos);
        } else {
            // The last declaration was out of this function
            // Need to find/create an upvalue to it
        }
        None
        // self.sub_compilers.last_mut()?.1.la
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_declaration() {
        let mut c = Compiler::new();

        assert_eq!(c.last_declaration(&Symbol("ooh".into())), None);
        c.add_assignment(&Assignment {
            identifier: Symbol("hi".into()),
            value: Expression::Symbol(Symbol("empty".into())),
        });
        c.add_assignment(&Assignment {
            identifier: Symbol("there".into()),
            value: Expression::Symbol(Symbol("empty".into())),
        });

        assert_eq!(
            c.last_declaration(&Symbol("there".into())),
            Some((
                1,
                &Local {
                    depth: 1,
                    name: Rc::new("there".into())
                }
            ))
        );
        assert_eq!(
            c.last_declaration(&Symbol("hi".into())),
            Some((
                0,
                &Local {
                    depth: 1,
                    name: Rc::new("hi".into())
                }
            ))
        );
        c.push_compiler(Rc::new("Cool FUNCTION".into()));
        c.add_assignment(&Assignment {
            identifier: Symbol("hi2".into()),
            value: Expression::Symbol(Symbol("empty".into())),
        });
        c.add_assignment(&Assignment {
            identifier: Symbol("there".into()),
            value: Expression::Symbol(Symbol("empty".into())),
        });
        assert_eq!(c.sub_compilers.last().unwrap().1.depth, 2);

        assert_eq!(
            c.last_declaration(&Symbol("there".into())),
            Some((
                3,
                &Local {
                    depth: 2,
                    name: Rc::new("there".into())
                }
            ))
        );
        assert_eq!(
            c.last_declaration(&Symbol("hi".into())),
            Some((
                0,
                &Local {
                    depth: 1,
                    name: Rc::new("hi".into())
                }
            ))
        );
        assert_eq!(
            c.last_declaration(&Symbol("hi2".into())),
            Some((
                2,
                &Local {
                    depth: 2,
                    name: Rc::new("hi2".into())
                }
            ))
        );
    }
}
