use crate::{Token};

#[derive(Debug)]
pub struct Crawler<'a> {
    pub s: &'a str,
}

impl<'a> Crawler<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut i = 0;
        let s_bytes = self.s.as_bytes();
        let s_len = s_bytes.len();
        while i < s_len {
            let curr = s_bytes[i] as char;
            if curr.is_ascii_alphanumeric() || curr == '_' {
                let mut buffer = String::new();
                let nx_c = s_bytes[i] as char; 
                while i < s_len && (nx_c.is_ascii_alphanumeric() || nx_c == '_') {
                    let nx_c_literal = s_bytes[i] as char;
                    if nx_c_literal.is_alphanumeric() || nx_c_literal == '_' {
                        buffer.push(nx_c_literal);
                        i += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Literal(buffer));
            } else {
                let nx_c = s_bytes[i] as char; 
                match nx_c {
                    '{' => {
                        tokens.push(Token::OpenParentheses);
                    }
                    '}' => {
                        tokens.push(Token::CloseParentheses);
                    }
                    '$' => {
                        tokens.push(Token::Dollar);
                    }
                    ',' => {
                        tokens.push(Token::Comma);
                    }
                    _ if nx_c.is_whitespace() => {
                    }
                    _ => {
                        tokens.push(Token::NonLiteral(nx_c.to_string()));
                    }
                }
                i += 1;
            }
        }
        tokens
    }
}