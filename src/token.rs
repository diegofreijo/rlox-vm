#[derive(Clone)]
pub struct TokenResult<'a> {
    pub line: i32,
    pub token_type: TokenType,
    pub data: Result<Token<'a>, String>,
}

impl<'a> TokenResult<'a> {
    pub fn invalid() -> Self {
        TokenResult{ line: -1, token_type: TokenType::Error, data: Err(String::from("Invalid")) }
    }

    // pub fn error_message(&self) -> String {
    //     match self.data {
    //         Ok(_) => "".to_string(),
    //         Err(m) => m,
    //     }
    // }

    // pub fn is_type(&self, expected: TokenType) -> bool {
    //     match self.data {
    //         Ok(_) => todo!(),
    //         Err(_) => todo!(),
    //     }
    // }
}

#[derive(Clone)]
pub struct Token<'a> {
    pub start: usize,
    pub end: usize,
    pub lexeme: &'a str,
}


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Slash, Star,

    // One or two character s.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,

    Error, Eof
} 
