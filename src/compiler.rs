use core::panic;
use std::rc::Rc;

use crate::{
    chunk::{IdentifierName, LocalVarIndex, Operation},
    object::{ObjFunction, ObjString},
    scanner::Scanner,
    token::{TokenResult, TokenType},
    value::Value,
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

#[derive(Debug)]
struct Local {
    pub name: String,
    pub depth: i8,
}

enum FunctionType {
    Function,
    Script,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    // frames: Vec<ObjFunction>,
    // function_type: FunctionType,
    pub had_error: bool,
    panic_mode: bool,

    scanner: Scanner<'a>,

    previous: TokenResult<'a>,
    current: TokenResult<'a>,

    locals: Vec<Local>,
    scope_depth: i8,
}

impl<'a> Compiler<'a> {
    pub fn from_source(source: &'a String) -> Compiler<'a> {
        Compiler {
            // frames: vec![],
            // function_type: FunctionType::Script,
            had_error: false,
            panic_mode: false,

            scanner: Scanner::new(source),
            previous: TokenResult::invalid(),
            current: TokenResult::invalid(),

            locals: vec![],
            scope_depth: 0,
        }
    }

    pub fn compile(&mut self) -> ObjFunction {
        let mut frame = ObjFunction::new("main");
        self.advance();

        while !self.matches(TokenType::Eof) {
            self.declaration(&mut frame);
        }

        self.emit_return(&mut frame);

        #[cfg(feature = "debug_print_code")]
        if !ret.had_error {
            ret.chunk.disassemble("code");
        }

        frame
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

    fn expression(&mut self, frame: &mut ObjFunction) {
        self.parse_precedence(&Precedence::Assignment, frame);
    }

    fn number(&mut self, frame: &mut ObjFunction) {
        // println!("Parsing number");
        let token_data = self.previous.data.as_ref().unwrap();
        let val = token_data.lexeme.parse::<f64>().unwrap();
        frame.chunk.emit_constant(Value::Number(val));
    }

    fn grouping(&mut self, frame: &mut ObjFunction) {
        self.expression(frame);
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self, frame: &mut ObjFunction) {
        let operator_type = self.previous.token_type;

        self.parse_precedence(&Precedence::Unary, frame);

        match operator_type {
            TokenType::Minus => frame.chunk.emit(Operation::Negate),
            TokenType::Bang => frame.chunk.emit(Operation::Not),
            _ => todo!(),
        }
    }

    fn binary(&mut self, frame: &mut ObjFunction) {
        let operator_type = self.previous.token_type;

        let next_precedence = Compiler::get_precedence(operator_type).next();
        self.parse_precedence(&next_precedence, frame);

        match operator_type {
            TokenType::BangEqual => {
                frame.chunk.emit(Operation::Equal);
                frame.chunk.emit(Operation::Not);
            }
            TokenType::EqualEqual => frame.chunk.emit(Operation::Equal),
            TokenType::Greater => frame.chunk.emit(Operation::Greater),
            TokenType::GreaterEqual => {
                frame.chunk.emit(Operation::Less);
                frame.chunk.emit(Operation::Not);
            }
            TokenType::Less => frame.chunk.emit(Operation::Less),
            TokenType::LessEqual => {
                frame.chunk.emit(Operation::Greater);
                frame.chunk.emit(Operation::Not);
            }
            TokenType::Plus => frame.chunk.emit(Operation::Add),
            TokenType::Minus => frame.chunk.emit(Operation::Substract),
            TokenType::Star => frame.chunk.emit(Operation::Multiply),
            TokenType::Slash => frame.chunk.emit(Operation::Divide),
            _ => todo!(),
        }
    }

    fn literal(&mut self, frame: &mut ObjFunction) {
        match self.previous.token_type {
            TokenType::True => frame.chunk.emit(Operation::True),
            TokenType::False => frame.chunk.emit(Operation::False),
            TokenType::Nil => frame.chunk.emit(Operation::Nil),
            tt => panic!("Expected a literal, found {:?}", tt),
        }
    }

    fn string(&mut self, frame: &mut ObjFunction) {
        let s = self.previous.data.clone().unwrap().lexeme;
        let obj_str = ObjString::from(s);
        frame.chunk.emit_constant(Value::String(Rc::from(obj_str)));
    }

    fn declaration(&mut self, frame: &mut ObjFunction) {
        if self.matches(TokenType::Fun) {
            self.fun_declaration(frame);
        } else if self.matches(TokenType::Var) {
            self.var_declaration(frame);
        } else {
            self.statement(frame);
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self, frame: &mut ObjFunction) {
        if self.matches(TokenType::Print) {
            self.print_statement(frame);
        } else if self.matches(TokenType::If) {
            self.if_statement(frame);
        } else if self.matches(TokenType::Return) {
            self.return_statement(frame);
        } else if self.matches(TokenType::While) {
            self.while_statement(frame);
        } else if self.matches(TokenType::For) {
            self.for_statement(frame);
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope(frame);
            self.block(frame);
            self.end_scope(frame, true);
        } else {
            self.expression_statement(frame);
        }
    }

    fn expression_statement(&mut self, frame: &mut ObjFunction) {
        self.expression(frame);
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        frame.chunk.emit(Operation::Pop);
    }

    fn print_statement(&mut self, frame: &mut ObjFunction) {
        self.expression(frame);
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        frame.chunk.emit(Operation::Print);
    }

    fn var_declaration(&mut self, frame: &mut ObjFunction) {
        if self.scope_depth == 0 {
            self.global_var_declaration(frame);
        } else {
            self.local_var_declaration(frame);
        }
    }

    fn global_var_declaration(&mut self, frame: &mut ObjFunction) {
        self.parse_variable("Expect variable name.");
        // TODO: see how can I remove this clone()
        let name = self.previous.clone().data.unwrap().lexeme.to_string();

        self.variable_expression(frame);

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(name, frame);
    }

    fn parse_variable(&mut self, error_message: &str) -> IdentifierName {
        self.consume(TokenType::Identifier, error_message);
        self.previous.data.as_ref().unwrap().lexeme.to_string()
    }

    fn local_var_declaration(&mut self, frame: &mut ObjFunction) {
        self.consume(TokenType::Identifier, "Expect variable name.");
        // TODO: see how can I remove this clone()
        let name = self.previous.clone().data.unwrap().lexeme.to_string();

        self.variable_expression(frame);
        self.declare_local(name);
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
    }

    fn variable_expression(&mut self, frame: &mut ObjFunction) {
        if self.matches(TokenType::Equal) {
            self.expression(frame);
        } else {
            frame.chunk.emit(Operation::Nil);
        }
    }

    fn define_variable(&mut self, name: IdentifierName, frame: &mut ObjFunction) {
        if self.scope_depth > 0 {
            self.mark_initialized();
        } else {
            frame.chunk.emit(Operation::DefineGlobal(name));
        }
    }

    fn declare_local(&mut self, name: IdentifierName) {
        self.validate_local(&name);

        let local = Local {
            name,
            depth: self.scope_depth,
        };

        self.locals.push(local);
    }

    fn variable(&mut self, can_assign: bool, frame: &mut ObjFunction) {
        let name = self.previous.clone().data.unwrap().lexeme.to_string();
        self.named_variable(name, can_assign, frame);
    }

    fn named_variable(&mut self, name: String, can_assign: bool, frame: &mut ObjFunction) {
        // TODO: clean up this chain of ifs without creating operations are are not going to be used.
        if let Some(i) = self.resolve_local(&name, frame) {
            // self.emit_variable(Operation::GetLocal(i), Operation::SetLocal(i), can_assign);
            if can_assign && self.matches(TokenType::Equal) {
                self.expression(frame);
                frame.chunk.emit(Operation::SetLocal(i));
            } else {
                frame.chunk.emit(Operation::GetLocal(i));
            }
        } else {
            if can_assign && self.matches(TokenType::Equal) {
                self.expression(frame);
                frame.chunk.emit(Operation::SetGlobal(name));
            } else {
                frame.chunk.emit(Operation::GetGlobal(name));
            }
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
            println!(
                "
[line {}] Error: {}
Compiler state: {:#?}
",
                line, message, self
            );
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

    fn parse_precedence(&mut self, precedence: &Precedence, frame: &mut ObjFunction) {
        self.advance();

        let can_assign = *precedence <= Precedence::Assignment;
        self.prefix_rule(self.previous.token_type, can_assign, frame);

        // println!("checking precedence {:?} <= {:?} == {:?}", precedence, &Compiler::get_precedence(self.current.token_type), precedence <= &Compiler::get_precedence(self.current.token_type));
        while precedence <= &Compiler::get_precedence(self.current.token_type) {
            self.advance();
            self.infix_rule(self.previous.token_type, frame);
        }
    }

    fn prefix_rule(&mut self, operator_type: TokenType, can_assign: bool, frame: &mut ObjFunction) {
        match operator_type {
            TokenType::LeftParen => self.grouping(frame),
            TokenType::Minus => self.unary(frame),
            TokenType::Number => self.number(frame),
            TokenType::True => self.literal(frame),
            TokenType::False => self.literal(frame),
            TokenType::Nil => self.literal(frame),
            TokenType::Bang => self.unary(frame),
            TokenType::String => self.string(frame),
            TokenType::Identifier => self.variable(can_assign, frame),
            tt => panic!("Expected expresion, got {:?}", tt),
        }
    }

    fn infix_rule(&mut self, operator_type: TokenType, frame: &mut ObjFunction) {
        match operator_type {
            TokenType::Minus => self.binary(frame),
            TokenType::Plus => self.binary(frame),
            TokenType::Slash => self.binary(frame),
            TokenType::Star => self.binary(frame),
            TokenType::BangEqual => self.binary(frame),
            TokenType::EqualEqual => self.binary(frame),
            TokenType::Greater => self.binary(frame),
            TokenType::GreaterEqual => self.binary(frame),
            TokenType::Less => self.binary(frame),
            TokenType::LessEqual => self.binary(frame),
            TokenType::And => self.and(frame),
            TokenType::Or => self.or(frame),
            TokenType::LeftParen => self.call(frame),
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
            TokenType::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }

    fn block(&mut self, frame: &mut ObjFunction) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration(frame);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn begin_scope(&mut self, frame: &mut ObjFunction) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, frame: &mut ObjFunction, undefine_locals: bool) {
        self.scope_depth -= 1;

        if undefine_locals {
            while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.scope_depth {
                self.locals.pop();
                frame.chunk.emit(Operation::Pop);
            }
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

    fn resolve_local(&self, name: &str, frame: &mut ObjFunction) -> Option<LocalVarIndex> {
        let mut ret = None;
        for (i, local) in self.locals.iter().rev().enumerate() {
            if local.name == name {
                ret = Some(i);
                break;
            }
        }
        ret
    }

    fn if_statement(&mut self, frame: &mut ObjFunction) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression(frame);
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.");

        let then_jump = self.emit_jump(Operation::JumpIfFalse(0), frame);
        frame.chunk.emit(Operation::Pop);
        self.statement(frame);

        let else_jump = self.emit_jump(Operation::Jump(0), frame);
        self.patch_jump(then_jump, frame);
        frame.chunk.emit(Operation::Pop);

        if self.matches(TokenType::Else) {
            self.statement(frame);
        }
        self.patch_jump(else_jump, frame);
    }

    fn emit_jump(&mut self, op: Operation, frame: &mut ObjFunction) -> usize {
        frame.chunk.emit(op);
        frame.chunk.op_count() - 1
    }

    fn patch_jump(&mut self, op_offset: usize, frame: &mut ObjFunction) {
        let jump = frame.chunk.op_count() - 1 - op_offset;
        let old_op = frame
            .chunk
            .op_get(op_offset)
            .expect("Tried patching an unexisting operation");
        let new_op = match old_op {
            Operation::JumpIfFalse(_) => Operation::JumpIfFalse(jump),
            Operation::Jump(_) => Operation::Jump(jump),
            _ => panic!("Tried to patch_jump a non-jump operation"),
        };
        frame.chunk.op_patch(op_offset, new_op.clone());
    }

    fn and(&mut self, frame: &mut ObjFunction) {
        let end_jump = self.emit_jump(Operation::JumpIfFalse(0), frame);
        frame.chunk.emit(Operation::Pop);
        self.parse_precedence(&Precedence::And, frame);
        self.patch_jump(end_jump, frame);
    }

    fn or(&mut self, frame: &mut ObjFunction) {
        let else_jump = self.emit_jump(Operation::JumpIfFalse(0), frame);
        let end_jump = self.emit_jump(Operation::Jump(0), frame);

        self.patch_jump(else_jump, frame);
        frame.chunk.emit(Operation::Pop);

        self.parse_precedence(&Precedence::Or, frame);
        self.patch_jump(end_jump, frame);
    }

    fn while_statement(&mut self, frame: &mut ObjFunction) {
        let loop_start = frame.chunk.op_count();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression(frame);
        self.consume(TokenType::RightParen, "Expect ')' after 'condition'.");

        let exit_jump = self.emit_jump(Operation::JumpIfFalse(0), frame);
        frame.chunk.emit(Operation::Pop);
        self.statement(frame);
        self.emit_loop(loop_start, frame);

        self.patch_jump(exit_jump, frame);
        frame.chunk.emit(Operation::Pop);
    }

    fn emit_loop(&mut self, loop_start: usize, frame: &mut ObjFunction) {
        let offset = frame.chunk.op_count() - loop_start + 1;
        frame.chunk.emit(Operation::Loop(offset));
    }

    fn for_statement(&mut self, frame: &mut ObjFunction) {
        self.begin_scope(frame);
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        // Initializer
        if self.matches(TokenType::Semicolon) {
            // No initializer
        } else if self.matches(TokenType::Var) {
            self.var_declaration(frame);
        } else {
            self.expression_statement(frame);
        }

        // Conditional
        let mut loop_start = frame.chunk.op_count();
        let mut exit_jump = None;
        if !self.matches(TokenType::Semicolon) {
            self.expression(frame);
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false
            exit_jump = Some(self.emit_jump(Operation::JumpIfFalse(0), frame));
            frame.chunk.emit(Operation::Pop);
        }

        // Increment
        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(Operation::Jump(0), frame);
            let increment_start = frame.chunk.op_count();
            self.expression(frame);

            frame.chunk.emit(Operation::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start, frame);
            loop_start = increment_start;
            self.patch_jump(body_jump, frame);
        }

        self.statement(frame);
        self.emit_loop(loop_start, frame);

        if let Some(offset) = exit_jump {
            self.patch_jump(offset, frame);
            frame.chunk.emit(Operation::Pop);
        }

        self.end_scope(frame, true);
    }

    fn fun_declaration(&mut self, frame: &mut ObjFunction) {
        let global = self.parse_variable("Expect function name.");
        // self.mark_initialized();
        let new_frame = self.function(global.clone());

        let function_value = Value::Function(Rc::from(new_frame));
        let constant = frame.chunk.add_constant(function_value);
        frame.chunk.emit(Operation::Constant(constant));

        self.define_variable(global, frame);
    }

    fn function(&mut self, name: String) -> ObjFunction {
        let mut frame = ObjFunction::new(&name);

        self.begin_scope(&mut frame);

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");

        // Function parameters
        if !self.check(TokenType::RightParen) {
            loop {
                frame.arity += 1;

                self.consume(TokenType::Identifier, "Expect parameter name.");
                let name = self.previous.clone().data.unwrap().lexeme.to_string();
                self.declare_local(name);

                // self.define_variable(parameter_name, &mut frame);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");

        self.block(&mut frame);
        self.emit_return(&mut frame);

        self.end_scope(&mut frame, false);

        frame
    }

    fn call(&mut self, frame: &mut ObjFunction) {
        let arg_count = self.argument_list(frame);
        frame.chunk.emit(Operation::Call(arg_count));
    }

    fn argument_list(&mut self, frame: &mut ObjFunction) -> u8 {
        let mut ret = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression(frame);
                ret += 1;
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        ret
    }

    fn return_statement(&mut self, frame: &mut ObjFunction) {
        // Contrary of the book, I do allow return statements outside a function.
        if self.matches(TokenType::Semicolon) {
            self.emit_return(frame);
        } else {
            self.expression(frame);
            self.consume(TokenType::Semicolon, "Expected ';' after return value.");
            frame.chunk.emit(Operation::Return);
        }
    }

    fn emit_return(&self, frame: &mut ObjFunction) {
        frame.chunk.emit(Operation::Nil);
        frame.chunk.emit(Operation::Return);
    }

    fn mark_initialized(&self) {
        // TODO: I'm not validating local variables with markInitialized like the book does.
    }
}

#[cfg(test)]
mod tests {
    use super::Compiler;
    use crate::{chunk::Operation, object::ObjFunction, value::Value};
    use std::rc::Rc;

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
                //
                Operation::Pop,
                Operation::Constant(0),
                Operation::Print,
                Operation::Jump(1),
                //
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
                //
                Operation::Pop,
                Operation::Constant(0),
                Operation::Print,
                Operation::Jump(3),
                //
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
                //
                Operation::Pop,
                Operation::Constant(2),
                Operation::SetGlobal("a".to_string()),
                Operation::Pop,
                Operation::Jump(4),
                //
                Operation::Pop,
                Operation::Constant(3),
                Operation::SetGlobal("a".to_string()),
                //
                Operation::Pop,
                Operation::GetGlobal("a".to_string()),
                Operation::Print,
            ],
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::new_string("true"),
                Value::new_string("false"),
            ],
        );
    }

    #[test]
    fn whiles() {
        assert_chunk(
            "while(true) { print 1; }",
            vec![
                Operation::True,
                Operation::JumpIfFalse(4),
                //
                Operation::Pop,
                Operation::Constant(0),
                Operation::Print,
                Operation::Loop(6),
                //
                Operation::Pop,
            ],
            vec![Value::Number(1.0)],
        );

        assert_chunk(
            "var a = 0; while(a < 5) { print a; a = a + 1; }",
            vec![
                Operation::Constant(0),
                Operation::DefineGlobal("a".to_string()),
                // Condition
                Operation::GetGlobal("a".to_string()),
                Operation::Constant(1),
                Operation::Less,
                Operation::JumpIfFalse(9),
                // Loop
                Operation::Pop,
                Operation::GetGlobal("a".to_string()),
                Operation::Print,
                Operation::GetGlobal("a".to_string()),
                Operation::Constant(2),
                Operation::Add,
                Operation::SetGlobal("a".to_string()),
                Operation::Pop,
                Operation::Loop(13),
                // End
                Operation::Pop,
            ],
            vec![Value::Number(0.0), Value::Number(5.0), Value::Number(1.0)],
        );
    }

    #[test]
    fn fors() {
        assert_chunk(
            "for(var i = 0; i < 10; i = i + 1) { print i; }",
            vec![
                // Initialization
                Operation::Constant(0),
                // Condition
                Operation::GetLocal(0),
                Operation::Constant(1),
                Operation::Less,
                Operation::JumpIfFalse(11),
                Operation::Pop,
                Operation::Jump(6),
                // Increment
                Operation::GetLocal(0),
                Operation::Constant(2),
                Operation::Add,
                Operation::SetLocal(0),
                Operation::Pop,
                Operation::Loop(12),
                // Body
                Operation::GetLocal(0),
                Operation::Print,
                Operation::Loop(9),
                // Cleanup (var and condition)
                Operation::Pop,
                Operation::Pop,
            ],
            vec![Value::Number(0.0), Value::Number(10.0), Value::Number(1.0)],
        );
    }

    #[test]
    fn procedures() {
        // Definition of pepe, will use it on the tests
        let mut pepe = ObjFunction::new("pepe");
        pepe.chunk.emit_many(&mut vec![
            Operation::Constant(0),
            Operation::Print,
            Operation::Nil,
            Operation::Return,
        ]);
        pepe.chunk.add_constant(Value::Number(1.0));

        assert_chunk(
            "fun pepe() { print 1; }",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("pepe".to_string()),
            ],
            vec![Value::Function(Rc::from(pepe.clone()))],
        );

        assert_chunk(
            "fun pepe() { print 1; } pepe();",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("pepe".to_string()),
                // Call
                Operation::GetGlobal("pepe".to_string()),
                Operation::Call(0),
                Operation::Pop,
            ],
            vec![Value::Function(Rc::from(pepe.clone()))],
        );

        assert_chunk(
            "fun pepe() { print 1; } print pepe();",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("pepe".to_string()),
                // Call
                Operation::GetGlobal("pepe".to_string()),
                Operation::Call(0),
                Operation::Print,
            ],
            vec![Value::Function(Rc::from(pepe.clone()))],
        );
    }

    #[test]
    fn functions() {
        // Definition of add, will use it on the tests
        let mut add = ObjFunction::new("add");
        add.arity = 2;
        add.chunk.emit_many(&mut vec![
            Operation::GetLocal(1),
            Operation::GetLocal(0),
            Operation::Add,
            Operation::Return,
            Operation::Nil,
            Operation::Return,
        ]);

        assert_chunk(
            "fun add(a, b) { return a + b; }",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("add".to_string()),
            ],
            vec![Value::Function(Rc::from(add.clone()))],
        );

        assert_chunk(
            "fun add(a, b) { return a + b; } print add(2,3);",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("add".to_string()),
                // Call
                Operation::GetGlobal("add".to_string()),
                Operation::Constant(1),
                Operation::Constant(2),
                Operation::Call(2),
                // Print
                Operation::Print,
            ],
            vec![Value::Function(Rc::from(add.clone())), Value::Number(2.0), Value::Number(3.0)],
        );
    }


    #[test]
    fn recursive_functions() {
        // Recursive definition of fact, will use it on the tests
        let mut fact = ObjFunction::new("fact");
        fact.arity = 1;
        fact.chunk.emit_many(&mut vec![
            // Condition
            Operation::GetLocal(0),
            Operation::Constant(0),
            Operation::Greater,
            Operation::Not,
            Operation::JumpIfFalse(4),
            // Then
            Operation::Pop,
            Operation::Constant(1),
            Operation::Return,
            // Else
            Operation::Jump(9),
            Operation::Pop,
            Operation::GetLocal(0),
            Operation::GetGlobal("fact".to_string()),
            Operation::GetLocal(0),
            Operation::Constant(2),
            Operation::Substract,
            Operation::Call(1),
            Operation::Multiply,
            Operation::Return,
            // Cleanup
            Operation::Nil,
            Operation::Return,
        ]);
        fact.chunk.add_constant(Value::Number(1.0));
        fact.chunk.add_constant(Value::Number(1.0));
        fact.chunk.add_constant(Value::Number(1.0));

        assert_chunk(
            "
fun fact(n) {
    if(n <= 1) { 
        return 1; 
    } else { 
        return n * fact(n-1); 
    } 
} 

print fact(5);
            ",
            vec![
                // Definition
                Operation::Constant(0),
                Operation::DefineGlobal("fact".to_string()),
                // Call
                Operation::GetGlobal("fact".to_string()),
                Operation::Constant(1),
                Operation::Call(1),
                // Print
                Operation::Print,
            ],
            vec![Value::Function(Rc::from(fact.clone())), Value::Number(5.0)],
        );
    }


    #[test]
    fn native_functions() {
        assert_chunk(
            "print clock();",
            vec![
                // Definition
                Operation::GetGlobal("clock".to_string()),
                Operation::Call(0),
                Operation::Print,
            ],
            vec![],
        );
    }

    //////////////////////////

    fn assert_expression(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        operations.push(Operation::Pop);
        assert_chunk(source, operations, constants);
    }

    fn assert_chunk(source: &str, mut operations: Vec<Operation>, constants: Vec<Value>) {
        let source2 = String::from(source);
        operations.push(Operation::Nil);
        operations.push(Operation::Return);

        let mut compiler = Compiler::from_source(&source2);
        let frame = compiler.compile();

        assert!(
            !compiler.had_error,
            "\nCOMPILER ERROR for source: {}",
            source
        );
        assert_eq!(
            frame.chunk.code, operations,
            "\nOPERATIONS failed for source: {}",
            source
        );
        assert_eq!(
            frame.chunk.constants, constants,
            "\nCONSTANTS failed for source:\n\t{}",
            source
        );
    }
}
