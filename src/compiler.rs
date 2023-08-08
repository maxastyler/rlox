use crate::{
    ast::{Expression, Symbol},
    chunk::Chunk,
    parser::parse,
};

pub struct Local {
    name: Symbol,
    depth: usize,
}

pub struct Compiler {
    depth: usize,
    locals_count: usize,
    locals: Vec<Local>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            depth: 0,
            locals_count: 0,
            locals: vec![],
        }
    }

    pub fn compile(&mut self, source: &str) -> Chunk {
        let Ok((rest, ast)) = parse(source);
        let mut chunk = Chunk::default();
        ast.into_iter()
            .for_each(|e| self.compile_expression(&mut chunk, e));
        chunk
    }

    fn compile_expression(&mut self, chunk: &mut Chunk, expression: Expression) {
        match expression {
            Expression::Call(c) => {}
            _ => unimplemented!(),
        }
    }
}
