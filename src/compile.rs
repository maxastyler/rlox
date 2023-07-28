use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
};

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
        let prev = std::mem::replace(&mut self.current, None);
        self.previous = prev;
        loop {
            match scanner.scan_token(self.string).token_type {
                TokenType::ERROR(_) => {}
                _ => break,
            }
            self.error_at_current();
        }
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
        self.emit_byte(OpCode::OP_RETURN.into(), chunk)
    }

    fn expression(&mut self) {}

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
}

pub fn compile(source: &String, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new();
    let mut parser = Parser::new(&source);
    let mut line: Option<usize> = None;
    scanner.advance(source);
    parser.expression();
    parser.consume(&mut scanner, TokenType::EOF);
    parser.end_compiler(chunk);
    true
}
