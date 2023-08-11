use nom::branch::alt;

#[derive(Clone, Debug, PartialEq)]
struct Token {
    pub token_type: TokenType,
    pub length: usize,
}

fn scan_str(s: &str) -> impl Iterator<Item = &str> {
    (1..=s.len()).map(|i| &s[0..i])
}

fn match_str(m: &str, input: &str) -> bool {
    if input.len() >= m.len() {
        &input[..m.len()] == m
    } else {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TokenType {
    Nil,
    True,
    False,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Dot,
    SemiColon,
    Comma,
    Bang,
    String(String),
    Symbol(String),
}

impl Token {
    pub fn next(input: &str) -> Option<Self> {
        [
            (TokenType::OpenBrace, "{"),
            (TokenType::CloseBrace, "}"),
            (TokenType::OpenParen, "("),
            (TokenType::CloseParen, ")"),
            (TokenType::Nil, "nil"),
            (TokenType::False, "false"),
            (TokenType::True, "true"),
        ]
        .iter()
        .find_map(|(t, ts)| {
            if match_str(ts, input) {
                Some(Token {
                    token_type: t.clone(),
                    length: ts.len(),
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // assert_eq!(Token::next("falseee"), None);
        // let x = scan_str("hi there").collect::<Vec<&str>>();
        // assert_eq!(x, vec![""]);
    }
}
