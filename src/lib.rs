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
    MissingParentheses
}

#[derive(Debug)]
pub struct Expansion {
    name: String,
    args: Vec<String>,
    body: Vec<Token>,
    replace_map: HashMap<String, Vec<usize>>
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
        let mut i = 4;

        loop {
            if i+2 >= expansion_build_tokens.len() { return Err(ExpansionBuildError::MissingClosingParentheses); }
            if matches!(expansion_build_tokens[i], Token::Dollar) {
                if let Token::Literal(expansion_arg_str) = &expansion_build_tokens[i+1] {
                    args.push(expansion_arg_str.clone());
                } else {
                    return Err(ExpansionBuildError::MissingName);
                }
            } else {
                return Err(ExpansionBuildError::MissingDollar);
            }
            if matches!(expansion_build_tokens[i+2], Token::Comma) {
                if matches!(expansion_build_tokens[i+3], Token::CloseParentheses) {
                    i += 4;
                    break;
                }
                i += 3;
            } else if matches!(expansion_build_tokens[i+2], Token::CloseParentheses) {
                i += 3;
                break;
            } else {
                return Err(ExpansionBuildError::MissingClosingParentheses);
            }
        }
        let mut body = Vec::new();
        if matches!(expansion_build_tokens[i+1], Token::OpenParentheses) {
            body = expansion_build_tokens[i+1..expansion_build_tokens.len()-3].to_vec();
        }

        let mut replace_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut body_idx = 1;
        while body_idx < body.len() {
            if !matches!(body[body_idx], Token::Literal(_)) { 
                body_idx += 1;
                continue;
            }
            for arg in &args {
                if let Token::Literal(body_literal) = &body[body_idx] {
                    if body_literal == arg && matches!(body[body_idx - 1], Token::Dollar) {
                        replace_map
                            .entry(arg.clone())
                            .or_insert_with(Vec::new)
                            .push(body_idx);
                        break;
                    }
                }
            }
            body_idx += 1;
        }

        Ok(Self { name: expansion_name, args, body, replace_map })
    }

    fn query(&self, query_tokens: Vec<Token>) -> Result<Vec<Token>, QueryError> {
        let body_clone = self.body.clone();
        let mut idx = 0;

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
        let mut query_idx = 0;
        while query_idx < query_tokens.len() {
            if let Token::Literal(ref literal) = query_tokens[query_idx] {
                if let Some(mac) = self.expansions.get(literal) {
                    let mut expansion_span = Vec::new();
                    if matches!(query_tokens[query_idx+1], Token::Colon) &&
                       matches!(query_tokens[query_idx+2], Token::OpenParentheses) 
                    { 
                        query_idx += 3;
                    } else {
                        return Err(QueryError::MissingParentheses);
                    }
                    let mut span_marker = 1; 
                    while query_idx < query_tokens.len() {
                        match query_tokens[query_idx] {
                            Token::OpenParentheses => span_marker += 1,
                            Token::CloseParentheses => span_marker -= 1,
                            _ => {},
                        }
                        expansion_span.push(query_tokens[query_idx].clone());
                        if span_marker == 0 { break; }
                    }
                    let expansion_tokens = mac.query(expansion_span);
                    let expansion_str = expansion_tokens.iter().map(|t| t.as_str()).collect::<String>();
                    ret_str.push_str(&expansion_str);
                }
            } else {
                ret_str.push_str(query_tokens[query_idx].as_str());
            }
            query_idx += 1;
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