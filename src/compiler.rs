use core::panic;
use std::rc::Rc;

use crate::{
    chunk::{Chunk, IdentifierName, LocalVarIndex, Operation},
    scanner::Scanner,
    token::{TokenResult, TokenType},
    value::{ObjString, Value},
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

struct Local {
    pub name: String,
    pub depth: i8,
}

pub struct Compiler<'a> {
    pub chunk: Chunk,

    pub had_error: bool,
    panic_mode: bool,

    scanner: Scanner<'a>,

    previous: TokenResult<'a>,
    current: TokenResult<'a>,

    locals: Vec<Local>,
    scope_depth: i8,
}

impl<'a> Compiler<'a> {
    pub fn from(source: &'a String) -> Compiler<'a> {
        Compiler {
            chunk: Chunk::new(),

            had_error: false,
            panic_mode: false,

            scanner: Scanner::new(&source),
            previous: TokenResult::invalid(),
            current: TokenResult::invalid(),

            locals: vec![],
            scope_depth: 0,
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
        // println!("Parsing number");
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
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
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
        if self.scope_depth == 0 {
            self.global_var_declaration();
        } else {
            self.local_var_declaration();
        }
    }

    fn global_var_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect variable name.");
        // TODO: see how can I remove this clone()
        let name = self.previous.clone().data.unwrap().lexeme.to_string();

        self.variable_expression();

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(name);
    }

    fn local_var_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect variable name.");
        // TODO: see how can I remove this clone()
        let name = self.previous.clone().data.unwrap().lexeme.to_string();

        self.variable_expression();
        self.declare_local(name);
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
    }

    fn variable_expression(&mut self) {
        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.chunk.emit(Operation::Nil);
        }
    }

    fn define_variable(&mut self, name: IdentifierName) {
        self.chunk.emit(Operation::DefineGlobal(name));
    }

    fn declare_local(&mut self, name: IdentifierName) {
        self.validate_local(&name);

        let local = Local {
            name,
            depth: self.scope_depth,
        };

        self.locals.push(local);
    }

    fn variable(&mut self, can_assign: bool) {
        let name = self.previous.clone().data.unwrap().lexeme.to_string();
        self.named_variable(name, can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        // TODO: clean up this chain of ifs without creating operations are are not going to be used.
        if let Some(i) = self.resolve_local(&name) {
            // self.emit_variable(Operation::GetLocal(i), Operation::SetLocal(i), can_assign);
            if can_assign && self.matches(TokenType::Equal) {
                self.expression();
                self.chunk.emit(Operation::SetLocal(i));
            } else {
                self.chunk.emit(Operation::GetLocal(i));
            }
        } else {
            if can_assign && self.matches(TokenType::Equal) {
                self.expression();
                self.chunk.emit(Operation::SetGlobal(name));
            } else {
                self.chunk.emit(Operation::GetGlobal(name));
            }
        }
    }

    // fn emit_variable<F>(&mut self, get_op: fn() -> Operation, set_op: Operation, can_assign: bool) {
    //     if can_assign && self.matches(TokenType::Equal) {
    //         self.expression();
    //         self.chunk.emit(set_op);
    //     } else {
    //         self.chunk.emit(get_op);
    //     }
    // }

    fn consume(&mut self, expected: TokenType, message: &str) {
        if self.current.token_type == expected {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn matches(&mut self, expected: TokenType) -> bool {
        if self.check(expected) {
            // println!("Matching {:?}", self.current);
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: TokenType) -> bool {
        // println!("checking {:?} with {:?}", self.current.token_type, expected);
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

        // println!("checking precedence {:?} <= {:?} == {:?}", precedence, &Compiler::get_precedence(self.current.token_type), precedence <= &Compiler::get_precedence(self.current.token_type));
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
            TokenType::And => self.and(),
            TokenType::Or => self.or(),
            _ => (), //panic!("Expect expresion"),
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
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            _ => Precedence::None,
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.scope_depth {
            self.locals.pop();
            self.chunk.emit(Operation::Pop);
        }
    }

    fn validate_local(&self, name: &String) {
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            } else if local.name == *name {
                panic!(
                    "There is already a local variable called '{}' in this scope.",
                    name
                );
            }
        }
    }

    fn resolve_local(&self, name: &str) -> Option<LocalVarIndex> {
        let mut ret = None;
        for (i, local) in self.locals.iter().rev().enumerate() {
            if local.name == name {
                ret = Some(i);
                break;
            }
        }
        ret
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.");

        let then_jump = self.emit_jump(Operation::JumpIfFalse(0));
        self.chunk.emit(Operation::Pop);
        self.statement();

        let else_jump = self.emit_jump(Operation::Jump(0));
        self.patch_jump(then_jump);
        self.chunk.emit(Operation::Pop);

        if self.matches(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn emit_jump(&mut self, op: Operation) -> usize {
        self.chunk.emit(op);
        self.chunk.op_count() - 1
    }

    fn patch_jump(&mut self, op_offset: usize) {
        let jump = self.chunk.op_count() - 1 - op_offset;
        let new_op = match self
            .chunk
            .op_get(op_offset)
            .expect("Tried patching an unexisting operation")
        {
            Operation::JumpIfFalse(_) => Operation::JumpIfFalse(jump),
            Operation::Jump(_) => Operation::Jump(jump),
            _ => panic!("Tried to patch_jump a non-jump operation"),
        };
        self.chunk.op_patch(op_offset, new_op);
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(Operation::JumpIfFalse(0));
        self.chunk.emit(Operation::Pop);
        self.parse_precedence(&Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(Operation::JumpIfFalse(0));
        let end_jump = self.emit_jump(Operation::Jump(0));
        
        self.patch_jump(else_jump);
        self.chunk.emit(Operation::Pop);
        
        self.parse_precedence(&Precedence::Or);
        self.patch_jump(end_jump);
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
            vec![Operation::Nil, Operation::DefineGlobal("a".to_string())],
            vec![],
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
            vec![Value::Number(1.0), Value::Number(2.0)],
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

    #[test]
    fn local_vars() {
        assert_chunk(
            "{ print 1; }",
            vec![Operation::Constant(0), Operation::Print],
            vec![Value::Number(1.0)],
        );
        assert_chunk("{ var a ; }", vec![Operation::Nil, Operation::Pop], vec![]);
        assert_chunk(
            "{ var a ; a = 1; }",
            vec![
                Operation::Nil,
                Operation::Constant(0),
                Operation::SetLocal(0),
                Operation::Pop,
                Operation::Pop,
            ],
            vec![Value::Number(1.0)],
        );
        assert_chunk(
            "{ var a ; a = 1; print a; }",
            vec![
                Operation::Nil,
                Operation::Constant(0),
                Operation::SetLocal(0),
                Operation::Pop,
                Operation::GetLocal(0),
                Operation::Print,
                Operation::Pop,
            ],
            vec![Value::Number(1.0)],
        );
        assert_chunk(
            "{ var a = 1+2; print a; }",
            vec![
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Add,
                Operation::GetLocal(0),
                Operation::Print,
                Operation::Pop,
            ],
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        // assert_chunk(
        //     "{ var a = 1; { var a = a; print a; } }",
        //     vec![
        //         Operation::Constant(0),
        //         Operation::GetLocal(0),
        //         Operation::GetLocal(0),
        //         Operation::Print,
        //         Operation::Pop,
        //     ],
        //     vec![Value::Number(1.0), Value::Number(2.0)],
        // );
    }

    #[test]
    fn ifs() {
        assert_chunk(
            "if(true) { print 1; } print 2;",
            vec![
                Operation::True,
                Operation::JumpIfFalse(4),
                
                Operation::Pop,
                Operation::Constant(0),
                Operation::Print,
                Operation::Jump(1),
                
                Operation::Pop,

                Operation::Constant(1),
                Operation::Print,
            ],
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        assert_chunk(
            "if(true) { print 1; } else { print 2; }",
            vec![
                Operation::True,
                Operation::JumpIfFalse(4),
                
                Operation::Pop,
                Operation::Constant(0),
                Operation::Print,
                Operation::Jump(3),

                Operation::Pop,
                Operation::Constant(1),
                Operation::Print,
            ],
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        assert_chunk(
            "var a; if(1 == 2) { a = \"true\"; } else { a = \"false\"; } print a;",
            vec![
                Operation::Nil,
                Operation::DefineGlobal("a".to_string()),
                Operation::Constant(0),
                Operation::Constant(1),
                Operation::Equal,

                Operation::JumpIfFalse(5),
                
                Operation::Pop,
                Operation::Constant(2),
                Operation::SetGlobal("a".to_string()),
                Operation::Pop,
                Operation::Jump(4),

                Operation::Pop,
                Operation::Constant(3),
                Operation::SetGlobal("a".to_string()),
                Operation::Pop,

                Operation::GetGlobal("a".to_string()),
                Operation::Print,
            ],
            vec![Value::Number(1.0), Value::Number(2.0), Value::new_string("true"), Value::new_string("false")],
        );
    }

    //////////////////////////

    fn assert_expression(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        operations.push(Operation::Pop);
        assert_chunk(source, operations, constants);
    }

    fn assert_chunk(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        let source2 = String::from(source);
        operations.push(Operation::Return);

        let mut compiler = Compiler::from(&source2);
        compiler.compile();

        assert!(!compiler.had_error, "\nsource: {}", source);
        assert_eq!(compiler.chunk.code, operations, "\nsource: {}", source);
        assert_eq!(compiler.chunk.constants, constants, "\nsource: {}", source);
    }
}
