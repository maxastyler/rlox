use std::fmt::Debug;

use nom::{character::is_space, And};

#[derive(PartialEq)]
pub enum Literal {
    Nil,
    False,
    True,
    Int(i64),
    Double(f64),
    Symbol(Symbol),
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => f.write_str("nil"),
            Self::False => f.write_str("false"),
            Self::True => f.write_str("true"),
            Self::Int(i) => f.write_fmt(format_args!("{}", i)),
            Self::Double(d) => f.write_fmt(format_args!("{}", d)),
	    Self::Symbol(s) => f.write_fmt(format_args!("{:?}", s))
        }
    }
}

#[derive(PartialEq)]
pub enum Binary {
    And(Vec<Expression>),
    Plus(Vec<Expression>),
}

impl Debug for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.inner_v();
        match v.len() {
            0 => panic!("Binary with empty vector"),
            1 => f.write_fmt(format_args!("({} {:?})", self.to_op_string(), v[0])),
            _ => {
                f.write_fmt(format_args!("({:?}", v[0]))?;
                for x in v.iter().skip(1) {
                    f.write_fmt(format_args!(" {} {:?}", self.to_op_string(), x))?;
                }
                f.write_str(")")
            }
        }
    }
}

impl Binary {
    pub fn ops() -> &'static [&'static str] {
        &["and", "+"]
    }

    fn inner_v(&self) -> &Vec<Expression> {
        match self {
            Binary::And(v) => v,
            Binary::Plus(v) => v,
        }
    }

    pub fn from_op_string(s: &str, vec: Vec<Expression>) -> Self {
        match s {
            "and" => Binary::And(vec),
            "+" => Binary::Plus(vec),
            _ => panic!("Could not match operator string"),
        }
    }

    pub fn to_op_string(&self) -> &'static str {
        match self {
            Binary::And(_) => "and",
            Binary::Plus(_) => "+",
        }
    }

    fn construct(f: fn(Vec<Expression>) -> Binary, mut v: Vec<Expression>) -> Expression {
        match v.len() {
            0 => panic!("Binary had length 0 vector"),
            1 => v.remove(0).simplify(),
            _ => Expression::Binary(Box::new(f(v.into_iter().map(|x| x.simplify()).collect()))),
        }
    }

    pub fn simplify(self) -> Expression {
        let (f, v): (fn(Vec<Expression>) -> Binary, Vec<_>) = match self {
            Binary::And(v) => (Binary::And, v),
            Binary::Plus(v) => (Binary::Plus, v),
        };
        Binary::construct(f, v)
    }
}

#[derive(Debug, PartialEq)]
pub struct Symbol(pub String);

#[derive(Debug, PartialEq)]
pub struct Parenthesised(pub Box<Expression>);

#[derive(PartialEq)]
pub enum Expression {
    Literal(Literal),
    Call(Symbol, Vec<Expression>),
    Binary(Box<Binary>),
    Assignment(Symbol, Box<Expression>),
    Def(Symbol, Vec<Symbol>, Box<Expression>),
    Parenthesised(Vec<Expression>),
    Block(Vec<Expression>),
}

impl Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Def(name, args, body) => {
                f.write_fmt(format_args!("{:?}({:?}){:?}", name, args, body))
            }
            Self::Literal(l) => f.write_fmt(format_args!("L|{:?}|", l)),
            Self::Binary(b) => f.write_fmt(format_args!("{:?}", b)),
            Self::Call(s, es) => f.write_fmt(format_args!("Call({:?}, {:?})", s, es)),
            Self::Parenthesised(p) => f.write_fmt(format_args!("Parens({:?})", p)),
            Self::Block(b) => f.write_fmt(format_args!("Block{{{:?}}}", b)),
            Self::Assignment(symbol, exp) => f.write_fmt(format_args!("{:?} = {:?}", symbol, exp)),
        }
    }
}

impl Expression {
    pub fn simplify(self) -> Self {
        match self {
            Expression::Binary(b) => b.simplify(),
            Expression::Block(es) => {
                Expression::Block(es.into_iter().map(|s| s.simplify()).collect())
            }
            x => x,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Assignment(Symbol, Box<Expression>),
    Print(Box<Expression>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_simplifies_correctly() {
        assert_eq!(
            Binary::Plus(vec![Expression::Binary(Box::new(Binary::And(vec![
                Expression::Literal(Literal::Double(3.0))
            ])))])
            .simplify(),
            Expression::Literal(Literal::Double(3.0))
        );
    }

    #[test]
    fn nested_binary_simplifies_correctly() {
        assert_eq!(
            Binary::And(vec![
                Expression::Binary(Box::new(Binary::Plus(vec![Expression::Literal(
                    Literal::Double(3.0)
                )]))),
                Expression::Binary(Box::new(Binary::Plus(vec![Expression::Literal(
                    Literal::Double(3.0)
                )])))
            ])
            .simplify(),
            Expression::Binary(Box::new(Binary::And(vec![
                Expression::Literal(Literal::Double(3.0)),
                Expression::Literal(Literal::Double(3.0))
            ])))
        )
    }
}
