use std::collections::HashMap;
pub mod crawler;
use crawler::Crawler;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Literal(String),
    NonLiteral(String),
    OpenParentheses,
    CloseParentheses,
    Dollar,
    Comma,
    Colon
}

impl Token {
    pub fn as_str(&self) -> &str {
        match self {
            Token::Literal(s) => s,
            Token::NonLiteral(c) => c,
            Token::OpenParentheses => "{",
            Token::CloseParentheses => "}",
            Token::Dollar => "$",
            Token::Comma => ",",
            Token::Colon => ":"
        }
    }
}

#[derive(Debug)]
pub enum MacroBuildError {
    MissingParentheses,
    MissingDollar,
    MissingName,
    MissingClosingParentheses
}

#[derive(Debug)]
pub struct Macro {
    name: String,
    args: Vec<Token>,
    body: Vec<Token>,
    replace_map: HashMap<Token, Vec<usize>>
}

impl Macro {
    pub fn build(macro_build_tokens: Vec<Token>) -> Result<Self, MacroBuildError> {
        let mut macro_name = String::new();
        let mut args = Vec::new();
        if matches!(macro_build_tokens[0], Token::Dollar) {
            if let Token::Literal(macro_name_str) = &macro_build_tokens[1] {
                if matches!(macro_build_tokens[2], Token::OpenParentheses) {
                    macro_name.push_str(&macro_name_str);
                } else {
                    return Err(MacroBuildError::MissingParentheses);
                }
            } else {
                return Err(MacroBuildError::MissingName);
            }
        } else {
            return Err(MacroBuildError::MissingDollar);
        }

        if !matches!(macro_build_tokens[3], Token::OpenParentheses) { return Err(MacroBuildError::MissingParentheses); }
        let mut i = 4;

        loop {
            if (i+2 >= macro_build_tokens.len()) { return Err(MacroBuildError::MissingClosingParentheses); }
            if matches!(macro_build_tokens[i], Token::Dollar) {
                if let Token::Literal(macro_arg_str) = &macro_build_tokens[i+1] {
                    args.push(macro_build_tokens[i+1].clone());
                    println!("pushed arg: {:?}", macro_build_tokens[i+1]);
                } else {
                    return Err(MacroBuildError::MissingName);
                }
            } else {
                return Err(MacroBuildError::MissingDollar);
            }
            if matches!(macro_build_tokens[i+2], Token::Comma) {
                if matches!(macro_build_tokens[i+3], Token::CloseParentheses) {
                    i += 4;
                    break;
                }
                i += 3;
            } else if matches!(macro_build_tokens[i+2], Token::CloseParentheses) {
                i += 3;
                break;
            } else {
                return Err(MacroBuildError::MissingClosingParentheses);
            }
        }
        let mut body = Vec::new();
        if matches!(macro_build_tokens[i+1], Token::OpenParentheses) {
            body = macro_build_tokens[i+1..macro_build_tokens.len()-3].to_vec();
        }
        println!("body: {:?}", body);

        let mut replace_map: HashMap<Token, Vec<usize>> = HashMap::new();
        let mut body_idx = 1;
        while body_idx < body.len() {
            println!("processing: {:?}", body[body_idx]);
            if !matches!(body[body_idx], Token::Literal(_)) { 
                body_idx += 1;
                continue;
            }
            for arg in &args {
                if let Token::Literal(body_literal) = &body[body_idx] {
                    if let Token::Literal(arg_literal) = arg {
                        if body_literal == arg_literal && matches!(body[body_idx - 1], Token::Dollar) {
                            replace_map
                                .entry(arg.clone())
                                .or_insert_with(Vec::new)
                                .push(body_idx);
                            break;
                        }
                    }
                }
            }
            body_idx += 1;
        }

        Ok(Self { name: macro_name, args, body, replace_map })
    }
}

// data transformation:
// add macros to macro engine which will store its args and body. it will also store a replace_map which maps 
// an arg to the indices in the body that it needs to replace.
// when we are queried, go over the query until it encounters an function that has the name of one of the expansion functions.
// query will return a string that has the properly expanded function. add this string to the string of the prefix. 
// continue finding other expansion functions and keep building the query string until it is fully formed.