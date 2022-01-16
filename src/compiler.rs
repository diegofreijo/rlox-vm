use crate::{
    scanner::Scanner,
    token::TokenType,
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a String) -> Self {
        Compiler {
            scanner: Scanner::new(&source),
        }
    }

    // pub fn compile(&mut self) -> Vec<TokenResult> {
    //     let mut ret = vec![];
    //     loop {
    //         let res = self.scanner.scan_token();
	// 		ret.push(res);
    //         // if let Ok(token) = res.data {
    //         //     if token.token_type == TokenType::Eof {
    //         //         break;
    //         //     }
    //         // }
    //     }
    //     ret
    // }

    pub fn test_scanner(&mut self) {
        let mut line = -1;
        loop {
            let res = self.scanner.scan_token();
            if res.line != line {
                print!("{:4} ", res.line);
                line = res.line;
            } else {
                print!("   | ");
            }

            match res.data {
                Ok(token) => {
                    println!("{:?}		'{}'", token.token_type, token.lexeme);
                    if token.token_type == TokenType::Eof {
                        break;
                    }
                }
                Err(message) => println!("Error '{}'", message),
            }
        }
    }
}
