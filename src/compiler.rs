use crate::{
    chunk::{Chunk, Operation},
    scanner::Scanner,
    token::{TokenResult, TokenType},
};

#[derive(Clone, PartialEq, PartialOrd)]
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
        let ret_order = (self.order()+1).min(10);
        Precedence::from_order(ret_order)
    }

    fn order(&self) -> u8 {
        match self {
            Precedence::None => 0,
            Precedence::Assignment => 1,
            Precedence::Or => 2,
            Precedence::And => 3,
            Precedence::Equality => 4,
            Precedence::Comparison => 5,
            Precedence::Term => 6,
            Precedence::Factor => 7,
            Precedence::Unary => 8,
            Precedence::Call => 9,
            Precedence::Primary => 10,
        }
    }

    fn from_order(order: u8) -> Precedence {
        match order {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => panic!("Unrecognized order {}", order),
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
        Compiler {
            scanner: Scanner::new(&source),
            previous: TokenResult::invalid(),
            current: TokenResult::invalid(),
            had_error: false,
            panic_mode: false,
            chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self) {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");

        self.chunk.emit(Operation::Return);

        #[cfg(feature="debug_print_code")]
        if !ret.had_error {
            ret.chunk.disassemble("code");
        }
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

        while precedence <= &Compiler::get_precedence(self.current.token_type) {
            self.advance();
            self.infix_rule(self.previous.token_type);
        }
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

    fn infix_rule(&mut self, operator_type: TokenType) {
        match operator_type {
            TokenType::Minus => self.binary(),
            TokenType::Plus => self.binary(),
            TokenType::Slash => self.binary(),
            TokenType::Star => self.binary(),
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
