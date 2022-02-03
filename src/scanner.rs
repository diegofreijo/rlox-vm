use peekmore::{PeekMore, PeekMoreIterator};

use crate::token::{Token, TokenResult, TokenType};
use std::str::Chars;

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a String,
    chars: PeekMoreIterator<Chars<'a>>,
    start: usize,
    current: usize,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Scanner {
            source,
            chars: source.chars().peekmore(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> TokenResult<'a> {
        self.skip_whitespaces();
        self.start = self.current;
        match self.advance() {
            Some(c) => match c {
                _ if Scanner::is_alpha(c) => self.identifier(),
                _ if Scanner::is_digit(c) => self.number(),

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
                '>' => {
                    self.make_token_if_matches(&'=', TokenType::GreaterEqual, TokenType::Greater)
                }

                // String literals
                '"' => self.string(),

                // Error
                _ => self.token_error(&format!("Unexpected character '{}'", c)),
            },
            None => self.make_eof(),
        }
    }

    fn make_token_if_matches(
        &mut self,
        expected: &char,
        on_match: TokenType,
        otherwise: TokenType,
    ) -> TokenResult<'a> {
        if self.matches(expected) {
            self.make_token(on_match)
        } else {
            self.make_token(otherwise)
        }
    }

    fn make_token(&self, token_type: TokenType) -> TokenResult<'a> {
        TokenResult {
            line: self.line,
            token_type,
            data: Ok(Token {
                start: self.start,
                end: self.current,
                lexeme: &self.source[self.start..self.current],
            }),
        }
    }

	fn make_identifier_token(&self) -> TokenResult<'a> {
		let lexeme = &self.source[self.start..self.current];
		match lexeme {
			"and" => self.make_token(TokenType::And),
			"class" => self.make_token(TokenType::Class),
			"else" => self.make_token(TokenType::Else),
			"if" => self.make_token(TokenType::If),
			"nil" => self.make_token(TokenType::Nil),
			"or" => self.make_token(TokenType::Or),
			"print" => self.make_token(TokenType::Print),
			"return" => self.make_token(TokenType::Return),
			"super" => self.make_token(TokenType::Super),
			"var" => self.make_token(TokenType::Var),
			"while" => self.make_token(TokenType::While),
			"false" => self.make_token(TokenType::False),
			"for" => self.make_token(TokenType::For),
			"fun" => self.make_token(TokenType::Fun),
			"this" => self.make_token(TokenType::This),
			"true" => self.make_token(TokenType::True),
			_ => self.make_token(TokenType::Identifier),
		}
		
	}

    fn make_eof(&self) -> TokenResult<'a> {
        TokenResult {
            line: self.line,
            token_type: TokenType::Eof,
            data: Ok(Token {
                start: self.start,
                end: self.current,
                lexeme: "",
            }),
        }
    }

    fn token_error(&self, message: &str) -> TokenResult<'a> {
        TokenResult {
            line: self.line,
            token_type: TokenType::Error,
            data: Err(message.to_string()),
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn peek_next(&mut self) -> Option<&char> {
        self.chars.peek_nth(1)
    }

    fn peek_matches(&mut self, expected: &char) -> bool {
        match self.peek() {
            Some(c) => c == expected,
            None => false,
        }
    }

    fn is_eof(&mut self) -> bool {
        self.peek() == None
    }

    fn peek_next_matches(&mut self, expected: &char) -> bool {
        match self.peek_next() {
            Some(c) => c == expected,
            None => false,
        }
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
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') => {
                    if self.peek_next_matches(&'/') {
                        self.advance();
                        self.advance();
                        loop {
                            if self.peek_matches(&'\n') || self.is_eof() {
                                break;
                            } else {
                                self.advance();
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> TokenResult<'a> {
        // I already consumed the initial " before. I'm storing as a lexeme the string
        // with no "s
        self.start += 1;

        while !self.peek_matches(&'"') && !self.is_eof() {
            if self.peek_matches(&'\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_eof() {
            self.token_error(&format!("Unterminated string. Token so far: {:?}", self.make_token(TokenType::String)))
        } else {
            let ret = self.make_token(TokenType::String);
            self.advance();
            ret
        }
    }

    fn number(&mut self) -> TokenResult<'a> {
        while self.peek_is_digit() {
            self.advance();
        }

        if self.peek_matches(&'.') {
            self.advance();
            while self.peek_is_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn peek_is_digit(&mut self) -> bool {
        match self.peek() {
            Some(c) => Scanner::is_digit(*c),
            None => false,
        }
    }

	fn peek_is_alpha(&mut self) -> bool {
		match self.peek() {
            Some(c) => Scanner::is_alpha(*c),
            None => false,
        }
	}

	fn identifier(&mut self) -> TokenResult<'a> {
		while self.peek_is_alpha() || self.peek_is_digit() {
			self.advance();
		}

		self.make_identifier_token()
	}

    fn is_digit(c: char) -> bool {
        // matches!(c, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')
		c.is_digit(10)
    }

	fn is_alpha(c: char) -> bool {
		 c == '_' || ('A'..'z').contains(&c)
    }
}

#[cfg(test)]
mod tests {
    use crate::{scanner, token::TokenType};

    #[test]
    fn peek() {
        let source = String::from("1234");
        let mut scanner = scanner::Scanner::new(&source);
        assert!(scanner.peek_matches(&'1'));

        assert_eq!(scanner.advance(), Some('1'));
        assert!(scanner.peek_matches(&'2'));
        assert!(scanner.peek_next_matches(&'3'));

        assert_eq!(scanner.advance(), Some('2'));
        assert!(scanner.peek_matches(&'3'));
        assert!(scanner.peek_next_matches(&'4'));
    }

    #[test]
    fn empty_source() {
        assert_token(String::from(""), TokenType::Eof);
        assert_token(String::from("    "), TokenType::Eof);
        assert_token(String::from("\r\t\t 	"), TokenType::Eof);
        assert_token(String::from("\n"), TokenType::Eof);
    }

    #[test]
    fn error_source() {
        assert_error_token(String::from("%"));
        assert_error_token(String::from("@"));
    }

    #[test]
    fn single_chars() {
        assert_token(String::from(""), TokenType::Eof);
        assert_token(String::from("("), TokenType::LeftParen);
        assert_token(String::from("}"), TokenType::RightBrace);
        assert_token(String::from("-"), TokenType::Minus);
        assert_token(String::from("+"), TokenType::Plus);
        assert_token(String::from("/"), TokenType::Slash);
    }

    #[test]
    fn double_chars() {
        assert_token(String::from("=="), TokenType::EqualEqual);
        assert_token(String::from("!="), TokenType::BangEqual);
        assert_token(String::from(">"), TokenType::Greater);
        assert_token(String::from(">="), TokenType::GreaterEqual);
    }

    #[test]
    fn full_source() {
        assert_tokens(String::from("+-"), &vec![TokenType::Plus, TokenType::Minus]);
        assert_tokens(
            String::from("==="),
            &vec![TokenType::EqualEqual, TokenType::Equal],
        );
        assert_tokens(
            String::from("()\n{}"),
            &vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
            ],
        );
    }

    #[test]
    fn coments() {
        assert_tokens(String::from("//pepe"), &vec![]);
        assert_tokens(String::from("+\n//pepe"), &vec![TokenType::Plus]);
        assert_tokens(String::from("/\n"), &vec![TokenType::Slash]);
        assert_tokens(String::from("/\n//pepe"), &vec![TokenType::Slash]);
        assert_tokens(
            String::from("/\n//pepe\n/"),
            &vec![TokenType::Slash, TokenType::Slash],
        );
    }

    #[test]
    fn strings() {
        assert_token_lexeme(String::from("\"pepe\""), TokenType::String, "pepe");
        assert_token_lexeme(String::from("\"pepe\"\n"), TokenType::String, "pepe");
        assert_token_lexeme(String::from("\"pepe\"\n\n"), TokenType::String, "pepe");
        assert_token_lexeme(String::from("\"\""), TokenType::String, "");
    }

	#[test]
	fn numbers() {
		assert_token_lexeme(String::from("0"), TokenType::Number, "0");
		assert_token_lexeme(String::from("4"), TokenType::Number, "4");
		assert_token_lexeme(String::from("42"), TokenType::Number, "42");
		assert_token_lexeme(String::from("13.99"), TokenType::Number, "13.99");
	}

	#[test]
	fn identifier() {
		assert_token(String::from("class"), TokenType::Class);
		assert_token(String::from("if"), TokenType::If);
		assert_token(String::from("while"), TokenType::While);
		assert_token(String::from("true"), TokenType::True);
		assert_token(String::from("false"), TokenType::False);
	
		assert_token_lexeme(String::from("pepe"), TokenType::Identifier, "pepe");
		assert_token_lexeme(String::from("for1"), TokenType::Identifier, "for1");
		assert_token_lexeme(String::from("whiles"), TokenType::Identifier, "whiles");
	}

    fn assert_token(source: String, expected: TokenType) {
        let mut scanner = scanner::Scanner::new(&source);
        let token = scanner.scan_token();

        assert_eq!(token.token_type, expected);
        
        assert_eq!(scanner.scan_token().token_type, TokenType::Eof);
    }

    fn assert_token_lexeme(source: String, expected_type: TokenType, expected_lexeme: &str) {
        let mut scanner = scanner::Scanner::new(&source);
        let token = scanner.scan_token();
        let data = token.data.unwrap();

        assert_eq!(token.token_type, expected_type);
        assert_eq!(data.lexeme, expected_lexeme);
        
        assert_eq!(scanner.scan_token().token_type, TokenType::Eof);
    }

    fn assert_tokens(source: String, expected_tokens: &Vec<TokenType>) {
        let mut scanner = scanner::Scanner::new(&source);
        for expected in expected_tokens {
            let actual = scanner.scan_token();
            assert_eq!(actual.token_type, *expected);
        }

        assert_eq!(scanner.scan_token().token_type, TokenType::Eof);
    }

    fn assert_error_token(source: String) {
        let mut scanner = scanner::Scanner::new(&source);
        let token = scanner.scan_token();

        assert!(token.data.is_err());
    }
}
