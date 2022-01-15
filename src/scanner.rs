use crate::token::{Token, TokenResult, TokenType};
use std::{iter::Peekable, str::Chars};

pub struct Scanner<'a> {
    source: &'a String,
    chars: Peekable<Chars<'a>>,
    start: usize,
    current: usize,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Scanner {
            source,
            chars: source.chars().peekable(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> TokenResult {
		self.skip_whitespaces();

        self.start = self.current;
        match self.advance() {
            Some(c) => match c {
                // Single-char tokens
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

                // Two-char tokens
                '!' => self.make_token_if_matches(&'=', TokenType::BangEqual, TokenType::Bang),
                '=' => self.make_token_if_matches(&'=', TokenType::EqualEqual, TokenType::Equal),
                '<' => self.make_token_if_matches(&'=', TokenType::LessEqual, TokenType::Less),
                '>' => self.make_token_if_matches(&'=', TokenType::GreaterEqual, TokenType::Greater),

				// Error
                _ => TokenResult {
                    line: self.line,
                    data: Err(format!("Unexpected character '{}'", c)),
                },
            },
            None => self.make_eof(),
        }
    }

    fn make_token_if_matches(
        &mut self,
        expected: &char,
        on_match: TokenType,
        otherwise: TokenType,
    ) -> TokenResult {
        if self.matches(expected) {
            self.make_token(on_match)
        } else {
            self.make_token(otherwise)
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

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn matches(&mut self, expected: &char) -> bool {
        match self.peek() {
            Some(c) => {
                if c == expected {
                    self.advance();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn skip_whitespaces(&mut self) {
        loop {
			match self.peek() {
				Some(c) => 
				if *c == ' ' || *c == '\t' || *c == '\r' {
					self.advance();
				} else if *c == '\n' {
					self.line += 1;
					self.advance();
				} else {
					break;
				},
				None => break,
			}
		}
    }
}
