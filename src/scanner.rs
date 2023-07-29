#[derive(Debug)]
pub struct Scanner {
    pub start: usize,
    pub current: usize,
    pub line: usize,
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_')
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn skip_whitespace(&mut self, source: &String) {
        loop {
            if let Some(s) = self.peek(source, 0) {
                match s {
                    " " | "\r" | "\t" => {
                        self.advance(source);
                    }
                    "\n" => {
                        self.line += 1;
                        self.advance(source);
                    }
                    "/" => {
                        if let Some("/") = self.peek(source, 1) {
                            loop {
                                if self.peek(source, 0) == Some("\n") || self.is_at_end(source) {
                                    break;
                                };
                                self.advance(source);
                            }
                        } else {
                            return;
                        }
                    }
                    _ => return,
                }
            } else {
                return;
            }
        }
    }

    fn peek<'a>(&'a self, source: &'a String, distance: usize) -> Option<&'a str> {
        let index = self.current + distance;
        if index >= source.len() {
            None
        } else {
            source.get(index..=index)
        }
    }

    pub fn scan_token(&mut self, source: &String) -> Token {
        self.skip_whitespace(source);
        self.start = self.current;
        if self.is_at_end(source) {
            self.make_token(TokenType::EOF)
        } else {
            let c = self.advance(source);
            if c.chars().all(|x| x.is_ascii_digit()) {
                return self.number(source);
            }
            if c.chars().all(is_alpha) {
                return self.identifier(source);
            }
            match c {
                "(" => return self.make_token(TokenType::LEFT_PAREN),
                ")" => return self.make_token(TokenType::RIGHT_PAREN),
                "{" => return self.make_token(TokenType::LEFT_BRACE),
                "}" => return self.make_token(TokenType::RIGHT_BRACE),
                ";" => return self.make_token(TokenType::SEMICOLON),
                "," => return self.make_token(TokenType::COMMA),
                "." => return self.make_token(TokenType::DOT),
                "-" => return self.make_token(TokenType::MINUS),
                "+" => return self.make_token(TokenType::PLUS),
                "/" => return self.make_token(TokenType::SLASH),
                "*" => return self.make_token(TokenType::STAR),
                "!" => {
                    return self.make_token_if_matches(
                        source,
                        "=",
                        TokenType::BANG_EQUAL,
                        TokenType::BANG,
                    )
                }
                "=" => {
                    return self.make_token_if_matches(
                        source,
                        "=",
                        TokenType::EQUAL_EQUAL,
                        TokenType::EQUAL,
                    )
                }
                "<" => {
                    return self.make_token_if_matches(
                        source,
                        "=",
                        TokenType::LESS_EQUAL,
                        TokenType::LESS,
                    )
                }
                ">" => {
                    return self.make_token_if_matches(
                        source,
                        "=",
                        TokenType::GREATER_EQUAL,
                        TokenType::GREATER,
                    )
                }
                "\"" => return self.string(source),
                _ => {}
            }
            self.error_token("unexpected character".into())
        }
    }

    fn make_token_if_matches(
        &mut self,
        source: &String,
        string: &str,
        token_true: TokenType,
        token_false: TokenType,
    ) -> Token {
        let t = if self.matches(source, string) {
            token_true
        } else {
            token_false
        };
        self.make_token(t)
    }

    fn number(&mut self, source: &String) -> Token {
        loop {
            if self
                .peek(source, 0)
                .map_or(true, |x| x.chars().all(|y| y.is_ascii_digit()))
            {
                break;
            } else {
                self.advance(source);
            }
        }
        if self.peek(source, 0).map_or(false, |x| x == ".")
            && self
                .peek(source, 1)
                .map_or(false, |x| x.chars().all(|y| y.is_ascii_digit()))
        {
            self.advance(source);
            loop {
                if self
                    .peek(source, 0)
                    .map_or(true, |x| x.chars().all(|y| y.is_ascii_digit()))
                {
                    break;
                } else {
                    self.advance(source);
                }
            }
        }
        self.make_token(TokenType::NUMBER)
    }

    fn identifier(&mut self, source: &String) -> Token {
        while (self
            .peek(source, 0)
            .map_or(false, |x| x.chars().all(is_alpha)))
            || (self
                .peek(source, 0)
                .map_or(false, |x| x.chars().all(|y| y.is_ascii_digit())))
        {
            self.advance(source);
        }
        return self.make_token(self.identifier_type(source));
    }

    fn identifier_type(&self, source: &String) -> TokenType {
        match source.get(self.start..=self.start).unwrap() {
            "a" => return self.check_keyword(source, 1, 2, "nd", TokenType::AND),
            "c" => return self.check_keyword(source, 1, 4, "lass", TokenType::CLASS),
            "e" => return self.check_keyword(source, 1, 3, "lse", TokenType::ELSE),
            "f" => {
                if self.current - self.start > 1 {
                    match source.get(self.start + 1..=self.start + 1).unwrap() {
                        "a" => return self.check_keyword(source, 2, 3, "lse", TokenType::FALSE),
                        "o" => return self.check_keyword(source, 2, 1, "r", TokenType::FOR),
                        "u" => return self.check_keyword(source, 2, 1, "n", TokenType::FUN),
                        _ => (),
                    }
                }
            }
            "i" => return self.check_keyword(source, 1, 1, "f", TokenType::IF),
            "n" => return self.check_keyword(source, 1, 2, "il", TokenType::NIL),
            "o" => return self.check_keyword(source, 1, 1, "r", TokenType::OR),
            "p" => return self.check_keyword(source, 1, 4, "rint", TokenType::PRINT),
            "r" => return self.check_keyword(source, 1, 5, "eturn", TokenType::RETURN),
            "s" => return self.check_keyword(source, 1, 4, "uper", TokenType::SUPER),
            "t" => {
                if self.current - self.start > 1 {
                    match source.get(self.start + 1..=self.start + 1).unwrap() {
                        "h" => return self.check_keyword(source, 2, 2, "is", TokenType::THIS),
                        "r" => return self.check_keyword(source, 2, 2, "ue", TokenType::TRUE),
                        _ => (),
                    }
                }
            }
            "v" => return self.check_keyword(source, 1, 2, "ar", TokenType::VAR),
            "w" => return self.check_keyword(source, 1, 4, "hile", TokenType::WHILE),
            _ => (),
        }
        return TokenType::IDENTIFIER;
    }

    fn check_keyword(
        &self,
        source: &String,
        start: usize,
        length: usize,
        s: &str,
        token_type: TokenType,
    ) -> TokenType {
        if source
            .get(self.start + start..=(self.start + start + length))
            .map_or(false, |x| x == s)
        {
            token_type
        } else {
            TokenType::IDENTIFIER
        }
    }

    fn string(&mut self, source: &String) -> Token {
        while self.peek(source, 0) == Some("\"") && !self.is_at_end(source) {
            if self.peek(source, 0) == Some("\n") {
                self.line += 1;
            }
            self.advance(source);
        }
        if self.is_at_end(source) {
            self.error_token("Unterminated string.".into())
        } else {
            self.advance(source);
            self.make_token(TokenType::STRING)
        }
    }

    pub fn advance<'a>(&'a mut self, source: &'a String) -> &'a str {
        self.current += 1;
        source.get((self.current - 1)..self.current).unwrap()
    }

    fn matches(&mut self, source: &String, expected: &str) -> bool {
        if self.is_at_end(source) {
            return false;
        };
        if source.get(self.current..self.current + 1).unwrap() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            start: self.start,
            length: self.current - self.start,
            line: self.line,
        }
    }

    fn error_token(&self, message: String) -> Token {
        Token {
            token_type: TokenType::ERROR(message),
            start: 0,
            length: 0,
            line: self.line,
        }
    }

    fn is_at_end(&self, source: &String) -> bool {
        self.current >= source.len()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,
    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    ERROR(String),
    EOF,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
}
