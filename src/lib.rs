use std::collections::{HashMap, HashSet};
mod crawler;
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

// data transformation:
// add macros to macro engine which will store its args and body. it will also store a replace_map which maps 
// an arg to the indices in the body that it needs to replace.
// when we are queried, go over the query until it encounters an function that has the name of one of the expansion functions.
// query will return a string that has the properly expanded function. add this string to the string of the prefix. 
// continue finding other expansion functions and keep building the query string until it is fully formed.