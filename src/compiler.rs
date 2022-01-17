use crate::{
    chunk::{Chunk, Operation},
    scanner::Scanner,
    token::{TokenResult, TokenType},
};

#[derive(Clone)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

// #[derive(Clone)]
// struct ParseRule<'a> {
//     prefix: Option<fn(&'a mut Compiler<'a>) -> ()>,
//     infix: Option<fn(&'a mut Compiler<'a>) -> ()>,
//     precedence: Precedence,
// }

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
            // TODO: see how can I remove this clone()
            match &self.current.data.clone() {
                Ok(_) => break,
                Err(message) => self.error_at_current(&message),
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(&Precedence::Assignment);
    }

    fn number(&mut self) {
        let token_data = self.previous.data.as_ref().unwrap();
        let val = token_data.lexeme.parse::<f64>().unwrap();
        self.chunk.emit_constant(val);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type;

        match operator_type {
            TokenType::Minus => self.chunk.emit(Operation::Negate),
            _ => todo!(),
        }

        self.parse_precedence(&Precedence::Unary);
    }

    fn binary(&mut self) {
        let operator_type = self.previous.token_type;
        match operator_type {
            TokenType::Plus => self.chunk.emit(Operation::Add),
            TokenType::Minus => self.chunk.emit(Operation::Substract),
            TokenType::Star => self.chunk.emit(Operation::Multiply),
            TokenType::Slash => self.chunk.emit(Operation::Divide),
            _ => todo!(),
        }

        let next_precedence = Compiler::get_precedence(operator_type).next();
        self.parse_precedence(&next_precedence);
    }

    fn parse_precedence(&mut self, precedence: &Precedence) {
        self.advance();
        self.prefix_rule(self.previous.token_type);
    }

    fn consume(&mut self, expected: TokenType, message: &str) {
        if self.current.token_type == expected {
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

    // fn get_rule<'b>(operator_type: TokenType) -> ParseRule<'a> {
    //     match operator_type {
    //         TokenType::LeftParen => ParseRule {
    //             prefix: Some(Compiler::grouping),
    //             infix: None,
    //             precedence: Precedence::None,
    //         },
    //         TokenType::Minus => ParseRule {
    //             prefix: Some(Compiler::unary),
    //             infix: Some(Compiler::binary),
    //             precedence: Precedence::Term,
    //         },
    //         TokenType::Plus => ParseRule {
    //             prefix: None,
    //             infix: Some(Compiler::binary),
    //             precedence: Precedence::Term,
    //         },
    //         TokenType::Slash => ParseRule {
    //             prefix: None,
    //             infix: Some(Compiler::binary),
    //             precedence: Precedence::Factor,
    //         },
    //         TokenType::Star => ParseRule {
    //             prefix: None,
    //             infix: Some(Compiler::binary),
    //             precedence: Precedence::Factor,
    //         },
    //         TokenType::Number => ParseRule {
    //             prefix: Some(Compiler::number),
    //             infix: None,
    //             precedence: Precedence::None,
    //         },
    //         _ => ParseRule {
    //             prefix: None,
    //             infix: None,
    //             precedence: Precedence::None,
    //         },
    //     }
    // }

    fn get_precedence(operator_type: TokenType) -> Precedence {
        match operator_type {
            TokenType::LeftParen => Precedence::None,
            TokenType::Minus => Precedence::Term,
            TokenType::Plus => Precedence::Term,
            TokenType::Slash => Precedence::Factor,
            TokenType::Star => Precedence::Factor,
            TokenType::Number => Precedence::None,
            _ => Precedence::None,
        }
    }

    fn prefix_rule(&mut self, operator_type: TokenType) {
        match operator_type {
            TokenType::LeftParen => self.grouping(),
            TokenType::Minus => self.unary(),
            TokenType::Number => self.number(),
            _ => panic!("Expect expresion"),
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
    //
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
