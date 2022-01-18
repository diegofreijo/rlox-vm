use std::rc::Rc;

use crate::{
    chunk::{Chunk, Operation, IdentifierName},
    scanner::Scanner,
    token::{TokenResult, TokenType},
    value::{ObjString, Value},
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
        let ret_order = (self.order() + 1).min(10);
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

        while !self.matches(TokenType::Eof) {
            self.declaration();
        }

        self.chunk.emit(Operation::Return);

        #[cfg(feature = "debug_print_code")]
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
        self.chunk.emit_constant(Value::Number(val));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type;

        self.parse_precedence(&Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.chunk.emit(Operation::Negate),
            TokenType::Bang => self.chunk.emit(Operation::Not),
            _ => todo!(),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.token_type;

        let next_precedence = Compiler::get_precedence(operator_type).next();
        self.parse_precedence(&next_precedence);

        match operator_type {
            TokenType::BangEqual => {
                self.chunk.emit(Operation::Equal);
                self.chunk.emit(Operation::Not);
            }
            TokenType::EqualEqual => self.chunk.emit(Operation::Equal),
            TokenType::Greater => self.chunk.emit(Operation::Greater),
            TokenType::GreaterEqual => {
                self.chunk.emit(Operation::Less);
                self.chunk.emit(Operation::Not);
            }
            TokenType::Less => self.chunk.emit(Operation::Less),
            TokenType::LessEqual => {
                self.chunk.emit(Operation::Greater);
                self.chunk.emit(Operation::Not);
            }
            TokenType::Plus => self.chunk.emit(Operation::Add),
            TokenType::Minus => self.chunk.emit(Operation::Substract),
            TokenType::Star => self.chunk.emit(Operation::Multiply),
            TokenType::Slash => self.chunk.emit(Operation::Divide),
            _ => todo!(),
        }
    }

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::True => self.chunk.emit(Operation::True),
            TokenType::False => self.chunk.emit(Operation::False),
            TokenType::Nil => self.chunk.emit(Operation::Nil),
            tt => panic!("Expected a literal, found {:?}", tt),
        }
    }

    fn string(&mut self) {
        let s = self.previous.data.clone().unwrap().lexeme;
        let obj_str = ObjString::from(s);
        self.chunk.emit_constant(Value::String(Rc::from(obj_str)));
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.chunk.emit(Operation::Pop);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.chunk.emit(Operation::Print);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.chunk.emit(Operation::Nil);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn parse_variable(&mut self, error_message: &str) -> IdentifierName {
        self.consume(TokenType::Identifier, error_message);
        // TODO: see how can I remove this clone()
        self.previous.clone().data.unwrap().lexeme.to_string()
    }

    // fn identifier_constant(&mut self, name: &str) -> IdentifierId {
    //     self.chunk.add_constant(Value::new_string(name))
    // }


    fn define_variable(&mut self, global: IdentifierName) {
        self.chunk.emit(Operation::DefineGlobal(global));
    }

    fn variable(&mut self, can_assign: bool) {
        // self.named_variable(self.previous);
        // let iid = self.identifier_constant(self.previous.clone().data.unwrap().lexeme);
        let name = self.previous.clone().data.unwrap().lexeme.to_string();
        self.named_variable(name, can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.chunk.emit(Operation::SetGlobal(name));
        } else {
            self.chunk.emit(Operation::GetGlobal(name));
        }
    }


    fn consume(&mut self, expected: TokenType, message: &str) {
        if self.current.token_type == expected {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn matches(&mut self, expected: TokenType) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: TokenType) -> bool {
        self.current.token_type == expected
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

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                break;
            }

            match self.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    break;
                }
                _ => {}
            }

            self.advance();
        }
    }

    fn parse_precedence(&mut self, precedence: &Precedence) {
        self.advance();

        let can_assign = *precedence <= Precedence::Assignment;
        self.prefix_rule(self.previous.token_type, can_assign);

        while precedence <= &Compiler::get_precedence(self.current.token_type) {
            self.advance();
            self.infix_rule(self.previous.token_type);
        }
    }

    fn prefix_rule(&mut self, operator_type: TokenType, can_assign: bool) {
        match operator_type {
            TokenType::LeftParen => self.grouping(),
            TokenType::Minus => self.unary(),
            TokenType::Number => self.number(),
            TokenType::True => self.literal(),
            TokenType::False => self.literal(),
            TokenType::Nil => self.literal(),
            TokenType::Bang => self.unary(),
            TokenType::String => self.string(),
            TokenType::Identifier => self.variable(can_assign),
            tt => panic!("Expected expresion, got {:?}", tt),
        }
    }

    fn infix_rule(&mut self, operator_type: TokenType) {
        match operator_type {
            TokenType::Minus => self.binary(),
            TokenType::Plus => self.binary(),
            TokenType::Slash => self.binary(),
            TokenType::Star => self.binary(),
            TokenType::BangEqual => self.binary(),
            TokenType::EqualEqual => self.binary(),
            TokenType::Greater => self.binary(),
            TokenType::GreaterEqual => self.binary(),
            TokenType::Less => self.binary(),
            TokenType::LessEqual => self.binary(),
            _ => (),//panic!("Expect expresion"),
        }
    }

    fn get_precedence(operator_type: TokenType) -> Precedence {
        match operator_type {
            TokenType::Minus => Precedence::Term,
            TokenType::Plus => Precedence::Term,
            TokenType::Slash => Precedence::Factor,
            TokenType::Star => Precedence::Factor,
            TokenType::BangEqual => Precedence::Equality,
            TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater => Precedence::Comparison,
            TokenType::GreaterEqual => Precedence::Comparison,
            TokenType::Less => Precedence::Comparison,
            TokenType::LessEqual => Precedence::Comparison,
            _ => Precedence::None,
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::{chunk::Operation, value::Value};

    use super::Compiler;

    #[test]
    fn constants() {
        assert_expression("2;", vec![Operation::Constant(0)], vec![Value::Number(2.0)]);
        assert_expression(
            "42;",
            vec![Operation::Constant(0)],
            vec![Value::Number(42.0)],
        );
        assert_expression(
            "0.1;",
            vec![Operation::Constant(0)],
            vec![Value::Number(0.1)],
        );
        assert_expression(
            "\"pepe\";",
            vec![Operation::Constant(0)],
            vec![Value::new_string("pepe")],
        );
    }

    #[test]
    fn unary() {
        assert_expression(
            "-3;",
            vec![Operation::Constant(0), Operation::Negate],
            vec![Value::Number(3.0)],
        );
        assert_expression(
            "-99.000000;",
            vec![Operation::Constant(0), Operation::Negate],
            vec![Value::Number(99.0)],
        );
    }

    #[test]
    fn binary() {
        assert_expression(
            "3+2;",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Add,
            ],
            vec![Value::Number(3.0), Value::Number(2.0)],
        );
        assert_expression(
            "0-1;",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Substract,
            ],
            vec![Value::Number(0.0), Value::Number(1.0)],
        );
        assert_expression(
            "5/5;",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Divide,
            ],
            vec![Value::Number(5.0), Value::Number(5.0)],
        );
    }

    #[test]
    fn parens() {
        assert_expression(
            "2 * (3+2);",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Constant(2),
                Operation::Add,
                Operation::Multiply,
            ],
            vec![Value::Number(2.0), Value::Number(3.0), Value::Number(2.0)],
        );
        assert_expression(
            "(3+2)-(2+2);",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Add,
                Operation::Constant(2),
                Operation::Constant(3),
                Operation::Add,
                Operation::Substract,
            ],
            vec![
                Value::Number(3.0),
                Value::Number(2.0),
                Value::Number(2.0),
                Value::Number(2.0),
            ],
        );
    }

    #[test]
    fn precedence() {
        assert_expression(
            "-3 + 2 * 2;",
            vec![
                Operation::Constant(0),
                Operation::Negate,
                Operation::Constant(1),
                Operation::Constant(2),
                Operation::Multiply,
                Operation::Add,
            ],
            vec![Value::Number(3.0), Value::Number(2.0), Value::Number(2.0)],
        );
        assert_expression(
            "(-1 + 2) * 3 - -4;",
            vec![
                Operation::Constant(0),
                Operation::Negate,
                Operation::Constant(1),
                Operation::Add,
                Operation::Constant(2),
                Operation::Multiply,
                Operation::Constant(3),
                Operation::Negate,
                Operation::Substract,
            ],
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ],
        );
    }


    #[test]
    fn global_vars() {
        assert_chunk(
            "var a;",
            vec![
                Operation::Nil,
                Operation::DefineGlobal("a".to_string()),
            ],
            vec![            ],
        );
        assert_chunk(
            "var a = 1;",
            vec![
                Operation::Constant(0),
                Operation::DefineGlobal("a".to_string()),
            ],
            vec![Value::Number(1.0)],
        );
        assert_chunk(
            "var a = 1; a;",
            vec![
                Operation::Constant(0),
                Operation::DefineGlobal("a".to_string()),
                Operation::GetGlobal("a".to_string()),
                Operation::Pop,
            ],
            vec![Value::Number(1.0)],
        );
        assert_chunk(
            "var a = 1; a = 2;",
            vec![
                Operation::Constant(0),
                Operation::DefineGlobal("a".to_string()),
                Operation::Constant(1),
                Operation::SetGlobal("a".to_string()),
                Operation::Pop,
            ],
            vec![Value::Number(1.0),Value::Number(2.0)],
        );
        assert_chunk(
            "var a = 1; var b = a;",
            vec![
                Operation::Constant(0),
                Operation::DefineGlobal("a".to_string()),
                Operation::GetGlobal("a".to_string()),
                Operation::DefineGlobal("b".to_string()),
            ],
            vec![Value::Number(1.0)],
        );
    }

    fn assert_expression(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        operations.push(Operation::Pop);
        assert_chunk(source, operations, constants);
    }

    fn assert_chunk(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        let source2 = String::from(source);
        operations.push(Operation::Return);

        let mut compiler = Compiler::from(&source2);
        compiler.compile();

        assert!(!compiler.had_error);
        assert_eq!(compiler.chunk.code, operations);
        assert_eq!(compiler.chunk.constants, constants);
    }
}
