use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    chunk::{Chunk, OpCode},
    debug::disassemble_chunk,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};

#[derive(Debug)]
struct Parser<'a> {
    current: Option<Token>,
    previous: Option<Token>,
    string: &'a String,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a String) -> Self {
        Parser {
            current: None,
            previous: None,
            string: source,
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self, scanner: &mut Scanner) {
        println!("The value of T is: {:?}\nscanner: {:?}", self, scanner);
        let prev = std::mem::replace(&mut self.current, None);
        self.previous = prev;
        loop {
	    println!("The scanner is: {:?}", scanner);
            let t = scanner.scan_token(self.string);
	    println!("The token is: {:?}", t);
            self.current = Some(t.clone());
            match t.token_type {
                TokenType::ERROR(_) => {}
                _ => break,
            }
            self.error_at_current();
        }
        println!("The value of T is: {:?}\nscanner: {:?}", self, scanner);
    }

    fn error(&mut self) {
        self.error_at(false);
    }

    fn error_at_current(&mut self) {
        self.error_at(true);
    }

    fn error_at(&mut self, current: bool) {
        if (current && self.current.is_some()) || (!current && self.previous.is_some()) {
            if self.panic_mode {
                return;
            }
            self.panic_mode = true;
            let token = if current {
                self.current.as_ref().unwrap()
            } else {
                self.previous.as_ref().unwrap()
            };
            eprint!("[line {}] Error", token.line);

            match token.token_type {
                TokenType::ERROR(_) => (),
                TokenType::EOF => eprint!(" at end"),
                _ => eprint!(
                    " at '{}'",
                    self.string
                        .get(token.start..token.start + token.length)
                        .unwrap()
                ),
            }
            eprintln!(
                ": {}",
                if let TokenType::ERROR(s) = &token.token_type {
                    s
                } else {
                    ""
                }
            );
            self.had_error = true;
        }
    }

    fn consume(&mut self, scanner: &mut Scanner, token: TokenType) {
        if let Some(t) = &self.current {
            if matches!(t.clone().token_type, token) {
                self.advance(scanner);
                return;
            }
        }
        self.error_at_current()
    }

    fn emit_byte(&self, byte: u8, chunk: &mut Chunk) {
        chunk.write_chunk(byte, self.previous.as_ref().unwrap().line)
    }

    fn emit_bytes(&self, byte_1: u8, byte_2: u8, chunk: &mut Chunk) {
        self.emit_byte(byte_1, chunk);
        self.emit_byte(byte_2, chunk);
    }

    fn end_compiler(&self, chunk: &mut Chunk) {
        self.emit_byte(OpCode::OP_RETURN.into(), chunk);
        if !self.had_error {
            disassemble_chunk(chunk, "code")
        }
    }

    fn grouping(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
        self.expression(scanner, chunk);
        self.consume(scanner, TokenType::RIGHT_PAREN);
    }

    fn number(&mut self, chunk: &mut Chunk) {
        let prev = self.previous.as_ref().unwrap();
        let num = self
            .string
            .get(prev.start..prev.start + prev.length)
            .unwrap()
            .parse::<f64>()
            .unwrap();
        self.emit_constant(num, chunk);
    }

    fn emit_constant(&mut self, constant: Value, chunk: &mut Chunk) {
        self.emit_bytes(
            OpCode::OP_CONSTANT.into(),
            chunk.make_constant(constant),
            chunk,
        );
    }

    fn unary(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
        let op_type = self.previous.as_ref().unwrap().token_type.clone();
        self.parse_precedence(scanner, chunk, Precedence::UNARY);

        match op_type {
            TokenType::MINUS => self.emit_byte(OpCode::OP_NEGATE.into(), chunk),
            _ => panic!(),
        }
    }

    fn binary(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
        let op_type = self.previous.as_ref().unwrap().token_type.clone();
        let (_, _, precedence) = get_rule(op_type.clone());
        self.parse_precedence(
            scanner,
            chunk,
            Precedence::try_from(1 + precedence as u8).unwrap(),
        );

        match op_type {
            TokenType::PLUS => self.emit_byte(OpCode::OP_ADD.into(), chunk),
            TokenType::MINUS => self.emit_byte(OpCode::OP_SUBTRACT.into(), chunk),
            TokenType::STAR => self.emit_byte(OpCode::OP_MULTIPLY.into(), chunk),
            TokenType::SLASH => self.emit_byte(OpCode::OP_DIVIDE.into(), chunk),
            _ => panic!(),
        }
    }

    fn expression(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
        self.parse_precedence(scanner, chunk, Precedence::ASSIGNMENT);
    }

    fn parse_precedence(
        &mut self,
        scanner: &mut Scanner,
        chunk: &mut Chunk,
        precedence: Precedence,
    ) {
        self.advance(scanner);
        println!("{:?}", self.previous);
        let (prefix, _, _) = get_rule(self.previous.as_ref().unwrap().token_type.clone());
        match prefix {
            Some(p) => self.prefix_rule(scanner, chunk, p),
            None => self.error(),
        }
        let pu8 = precedence as u8;
        while pu8 <= get_rule(self.current.as_ref().unwrap().token_type.clone()).2 as u8 {
            self.advance(scanner);
            let (_, infix, _) = get_rule(self.current.as_ref().unwrap().token_type.clone());
            if infix {
                self.binary(scanner, chunk)
            }
        }
    }

    fn prefix_rule(&mut self, scanner: &mut Scanner, chunk: &mut Chunk, rule: RuleType) {
        match rule {
            RuleType::Grouping => self.grouping(scanner, chunk),
            RuleType::Unary => self.unary(scanner, chunk),
            RuleType::Number => self.number(chunk),
        }
    }
}
enum RuleType {
    Grouping,
    Unary,
    Number,
}

type ParseRule = (Option<RuleType>, bool, Precedence);

fn get_rule(token: TokenType) -> ParseRule {
    match token {
        TokenType::LEFT_PAREN => (Some(RuleType::Grouping), false, Precedence::NONE),
        TokenType::MINUS => (Some(RuleType::Unary), true, Precedence::TERM),
        TokenType::PLUS => (None, true, Precedence::TERM),
        TokenType::SLASH => (None, true, Precedence::FACTOR),
        TokenType::STAR => (None, true, Precedence::FACTOR),
        TokenType::NUMBER => (Some(RuleType::Number), false, Precedence::NONE),
        _ => (None, false, Precedence::NONE),
    }
}

#[derive(Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum Precedence {
    NONE,
    ASSIGNMENT,
    OR,
    AND,
    EQUALITY,
    COMPARISON,
    TERM,
    FACTOR,
    UNARY,
    CALL,
    PRIMARY,
}

pub fn compile(source: &String, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new();
    let mut parser = Parser::new(&source);
    let mut line: Option<usize> = None;
    // scanner.advance(source);
    parser.expression(&mut scanner, chunk);
    parser.consume(&mut scanner, TokenType::EOF);
    parser.end_compiler(chunk);
    true
}
