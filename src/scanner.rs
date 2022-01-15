use std::str::CharIndices;
use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    chars: CharIndices<'a>,
	start: usize,
	current: usize,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            chars: source.char_indices(),
			start: 0,
			current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
		self.start = self.current;
        match self.advance() {
            Some((i, c)) => {
				match c {
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
					_ => self.error_token(format!("Unexpected character '{}'", c)),
				}
			},
            None => self.make_token(TokenType::Eof),
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
		Token { token_type, start: self.start, end: self.current, line: self.line }
    }

    fn error_token(&self, _message: String) -> Token {
		Token { token_type: TokenType::Error, start: 0, end: 0, line: self.line }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
		self.current += 1;
        self.chars.next()
    }
}
