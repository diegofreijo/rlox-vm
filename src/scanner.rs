use crate::token::{Token, TokenResult, TokenType};
use std::str::Chars;

pub struct Scanner<'a> {
    source: &'a String,
    chars: Chars<'a>,
    start: usize,
    current: usize,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Scanner {
            source: source,
            chars: source.chars(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> TokenResult {
        self.start = self.current;
        match self.advance() {
            Some(c) => match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ';' => self.make_token(TokenType::Semicolon),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '/' => self.make_token(TokenType::Slash),
                '*' => self.make_token(TokenType::Star),
                _ => TokenResult {
                    line: self.line,
                    data: Err(format!("Unexpected character '{}'", c)),
                },
            },
            None => self.make_eof(),
        }
    }

    fn make_token(&self, token_type: TokenType) -> TokenResult {
        TokenResult {
            line: self.line,
            data: Ok(Token {
                token_type,
                start: self.start,
                end: self.current,
                lexeme: &self.source[self.start..self.current],
            }),
        }
    }

	fn make_eof(&self) -> TokenResult {
        TokenResult {
            line: self.line,
            data: Ok(Token {
                token_type: TokenType::Eof,
                start: self.start,
                end: self.current,
                lexeme: "",
            }),
        }
    }

    // fn token_error(&self, message: &String) -> Result<Token, TokenError> {
    //     Err(TokenError {
    //         lexeme: &message.clone(),
    //         line: self.line,
    //     })
    // }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }
}
