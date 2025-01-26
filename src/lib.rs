use std::collections::HashMap;
pub mod crawler;
use crawler::{Token, tokenize};

#[derive(Debug)]
pub enum ExpansionBuildError {
    MissingParentheses,
    MissingDollar,
    MissingName,
    MissingClosingParentheses
}

#[derive(Debug)]
pub enum QueryError {
    ExpansionDNE,
    MissingParentheses,
    IncorrectNumberOfArguments
}

#[derive(Debug)]
pub struct Expansion {
    name: String,
    args: Vec<String>,
    body: Vec<Token>,
    replace_map: Vec<Vec<usize>>
}

impl Expansion {
    pub fn build(expansion_build_tokens: Vec<Token>) -> Result<Self, ExpansionBuildError> {
        let mut expansion_name = String::new();
        let mut args = Vec::new();
        if matches!(expansion_build_tokens[0], Token::Dollar) {
            if let Token::Literal(expansion_name_str) = &expansion_build_tokens[1] {
                if matches!(expansion_build_tokens[2], Token::OpenParentheses) {
                    expansion_name.push_str(&expansion_name_str);
                } else {
                    return Err(ExpansionBuildError::MissingParentheses);
                }
            } else {
                return Err(ExpansionBuildError::MissingName);
            }
        } else {
            return Err(ExpansionBuildError::MissingDollar);
        }

        if !matches!(expansion_build_tokens[3], Token::OpenParentheses) { return Err(ExpansionBuildError::MissingParentheses); }
        let mut idx = 4;

        loop {
            if idx+2 >= expansion_build_tokens.len() { return Err(ExpansionBuildError::MissingClosingParentheses); }
            if matches!(expansion_build_tokens[idx], Token::Dollar) {
                if let Token::Literal(expansion_arg_str) = &expansion_build_tokens[idx+1] {
                    args.push(expansion_arg_str.clone());
                } else {
                    return Err(ExpansionBuildError::MissingName);
                }
            } else {
                return Err(ExpansionBuildError::MissingDollar);
            }
            if matches!(expansion_build_tokens[idx+2], Token::Comma) {
                if matches!(expansion_build_tokens[idx+3], Token::CloseParentheses) {
                    idx += 4;
                    break;
                }
                idx += 3;
            } else if matches!(expansion_build_tokens[idx+2], Token::CloseParentheses) {
                idx += 3;
                break;
            } else {
                return Err(ExpansionBuildError::MissingClosingParentheses);
            }
        }
        let mut body = Vec::new();
        if matches!(expansion_build_tokens[idx+1], Token::OpenParentheses) {
            body = expansion_build_tokens[idx+1..expansion_build_tokens.len()-3].to_vec();
        }

        let mut replace_map: Vec<Vec<usize>> = Vec::with_capacity(args.len());
        let mut body_idx = 1;
        while body_idx < body.len() {
            if !matches!(body[body_idx], Token::Literal(_)) { 
                body_idx += 1;
                continue;
            }
            for (arg_idx, arg) in args.iter().enumerate() {
                if let Token::Literal(body_literal) = &body[body_idx] {
                    if body_literal == arg && matches!(body[body_idx - 1], Token::Dollar) {
                        replace_map[arg_idx].push(body_idx);
                        break;
                    }
                }
            }
            body_idx += 1;
        }

        Ok(Self { name: expansion_name, args, body, replace_map })
    }

    fn query(&self, query_tokens: Vec<Token>) -> Result<Vec<Token>, QueryError> {
        let mut body_clone = self.body.clone();
        let mut idx = 0;
        let mut literal_count = 0;
        for t in query_tokens {
            if matches!(t, Token::Literal(_)) { literal_count += 1; }
        }
        // go over all literals, make sure they have dollar before them, comma between args
        // we are using only positional arguments here, first ensure that len of self.args and number of literals
        // provided in query are same.
        // for each Literal, have an index side by side for args and access that index in self.replace_map,
        // in body_clone at the indices given by replace_map, replace that literal with whatever is in the query
        if self.args.len() != literal_count - 1 {
            return Err(QueryError::IncorrectNumberOfArguments);
        }
        Ok(body_clone)
    }
}

#[derive(Debug)]
pub struct ExpansionEngine {
    expansions: HashMap<String, Expansion>,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        Self { expansions: HashMap::new() }
    }

    pub fn add_expansion(&mut self, expansion_build_str: &str) -> Result<(), ExpansionBuildError> {
        let expansion_build_tokens = tokenize(expansion_build_str);
        let mac = Expansion::build(expansion_build_tokens)?;
        self.expansions.insert(mac.name.clone(), mac);
        Ok(())
    }

    pub fn query(&self, query_str: &str) -> Result<String, QueryError> {
        let mut ret_str = String::new();
        let query_tokens = tokenize(query_str);
        let mut idx = 0;
        while idx < query_tokens.len() {
            if let Token::Literal(ref literal) = query_tokens[idx] {
                if let Some(mac) = self.expansions.get(literal) {
                    let mut expansion_span = Vec::new();
                    if matches!(query_tokens[idx+1], Token::Colon) &&
                       matches!(query_tokens[idx+2], Token::OpenParentheses) 
                    { 
                        idx += 3;
                    } else {
                        return Err(QueryError::MissingParentheses);
                    }
                    let mut span_marker = 1; 
                    while idx < query_tokens.len() {
                        match query_tokens[idx] {
                            Token::OpenParentheses => span_marker += 1,
                            Token::CloseParentheses => span_marker -= 1,
                            _ => {},
                        }
                        expansion_span.push(query_tokens[idx].clone());
                        if span_marker == 0 { break; }
                    }
                    let expansion_tokens = mac.query(expansion_span)?;
                    let expansion_str = expansion_tokens.iter().map(|t| t.as_str()).collect::<String>();
                    ret_str.push_str(&expansion_str);
                }
            } else {
                ret_str.push_str(query_tokens[idx].as_str());
            }
            idx += 1;
        }
        Ok(ret_str)
    }
}

// data transformation:
// add expansions to expansion engine which will store its args and body. it will also store a replace_map which maps 
// an arg to the indices in the body that it needs to replace.
// when we are queried, go over the query until it encounters an function that has the name of one of the expansion functions.
// query will return a string that has the properly expanded function. add this string to the string of the prefix. 
// continue finding other expansion functions and keep building the query string until it is fully formed.