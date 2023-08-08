use crate::chunk::Chunk;

pub struct Function<'a> {
    arity: usize,
    chunk: Chunk<'a>,
    name: &'a str,
}

pub enum Object<'a> {
    Function(Function<'a>),
}

#[derive(Clone)]
pub enum Value<'a> {
    Number(f64),
    Boolean(bool),
    Nil,
    String(&'a str),
    Object(&'a Object<'a>),
}
