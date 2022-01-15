pub struct Token {
	pub token_type: TokenType,
	pub start: usize,
    pub end: usize,
	pub line: i32,
}

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
