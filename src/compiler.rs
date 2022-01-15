use crate::{scanner::{Scanner, self}, token::TokenType};


pub struct Compiler<'a> {
	scanner: Scanner<'a>,
}

impl<'a> Compiler<'a> {
	pub fn new(source: &'a str) -> Self {
		Compiler {
			scanner: Scanner::new(source),
		}
	}

	pub fn testScanner(&self) {
		let mut line = -1;
		loop {
			let token = self.scanner.scan_token();
			if token.line != line {
				print!("{:4} ", token.line);
				line = token.line;
			} else {
				print!("    | ");
			}
			println!("{:2} '{}'", token.token_type, token.length, token.start);
			if token.token_type == TokenType::Eof { break; }
		}
	}
}
