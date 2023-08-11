#[derive(Debug, PartialEq, Clone)]
pub struct Symbol(pub String);

#[derive(Debug, PartialEq)]
pub struct Function {
    pub identifier: Option<Symbol>,
    pub arguments: Vec<Symbol>,
    pub block: Block,
}
#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub identifier: Symbol,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Nil,
    Number(f64),
    String(String),
    Boolean(bool),
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::String(value)
    }
}
impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Literal::String(value.into())
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Literal::Number(value)
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Boolean(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Parenthesised(pub Vec<Expression>);
#[derive(Debug, PartialEq)]
pub struct Block(pub Vec<Expression>);
#[derive(Debug, PartialEq)]
pub struct Call {
    pub function: Expression,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Cond {
    pub conditions: Vec<(Expression, Expression)>,
    pub otherwise: Option<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Call(Box<Call>),
    Parenthesised(Box<Parenthesised>),
    Cond(Box<Cond>),
    Block(Box<Block>),
    Assignment(Box<Assignment>),
    Function(Box<Function>),
    Literal(Literal),
    Symbol(Symbol),
    Ignored(Box<Expression>),
}
