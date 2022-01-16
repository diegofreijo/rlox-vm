use crate::{
    chunk::{Chunk, Operation},
    scanner::Scanner,
    token::{TokenResult, TokenType},
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    previous: TokenResult<'a>,
    current: TokenResult<'a>,
    pub had_error: bool,
    panic_mode: bool,
    pub chunk: Chunk,
}

impl<'a> Compiler<'a> {
    pub fn from(source: &'a String) -> Compiler<'a> {
        let mut ret = Compiler {
            scanner: Scanner::new(&source),
            previous: TokenResult::invalid(),
            current: TokenResult::invalid(),
            had_error: false,
            panic_mode: false,
            chunk: Chunk::new(),
        };

        ret.advance();
        ret.expression();
        ret.consume(TokenType::Eof, "Expect end of expression");

        ret.chunk.emit(Operation::Return);

        ret
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            match &self.current.data.clone() {
                Ok(_) => break,
                Err(message) => self.error_at_current(&message),
            }
        }
    }
    
    fn expression(&mut self) {
        todo!()
    }

    fn consume(&mut self, expected: TokenType, message: &str) {
        if self.current.token_type == expected  {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }


    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.line, message);
    }

    fn error_at(&mut self, line: i32, message: &str) {
        if !self.panic_mode {
            self.panic_mode = true;
            println!("[line {}] Error: {}", line, message);
            self.had_error = true;
        }
    }

    // pub fn test_scanner(&mut self) {
    //     let mut line = -1;
    //     loop {
    //         let res = self.scanner.scan_token();
    //         if res.line != line {
    //             print!("{:4} ", res.line);
    //             line = res.line;
    //         } else {
    //             print!("   | ");
    //         }

    //         match res.data {
    //             Ok(token) => {
    //                 println!("{:?}		'{}'", token.token_type, token.lexeme);
    //                 if token.token_type == TokenType::Eof {
    //                     break;
    //                 }
    //             }
    //             Err(message) => println!("Error '{}'", message),
    //         }
    //     }
    // }
}
