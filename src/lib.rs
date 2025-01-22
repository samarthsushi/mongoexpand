use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Literal(String),
    NonLiteral(char),
    OpenParentheses,
    CloseParentheses,
    Dollar,
    Comma
}

impl Token {
    pub fn as_str(&self) -> String {
        match self {
            Token::Literal(s) => s.to_string(),
            Token::NonLiteral(c) => c.to_string(),
            Token::OpenParentheses => "{".to_string(),
            Token::CloseParentheses => "}".to_string(),
            Token::Dollar => "$".to_string(),
            Token::Comma => ",".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    start: usize,
    end: usize
}

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
        let mut start = 0;
        let mut i = 0;
        let s_bytes = self.s.as_bytes();
        let s_len = s_bytes.len();
        while i < s_len {
            let curr = s_bytes[i] as char;
            if curr.is_ascii_alphanumeric() || curr == '_' {
                let mut buffer = String::new();
                let nx_c = s_bytes[i] as char; 
                start = i;
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
                        tokens.push(Token::NonLiteral(nx_c));
                    }
                }
                i += 1;
            }
        }
        tokens
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum MacroErrorT {
    InvalidNumberOfDollars,
    SuspendedDollar,
    MissingComma,
    MissingParentheses,
    MissingName,
    Unexpected,
    MacroNotFound
}

#[derive(Debug)]
pub struct MacroError {
    ty: MacroErrorT,
    token: Token
}

#[derive(Debug)]
pub struct MacroProcessor {
    name: String,
    pub tokens: Vec<Token>,
    macro_args: Vec<Token>,
    replace_map: HashMap<Token, Vec<usize>>,
    macro_body_start_idx: usize
}

impl MacroProcessor {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { name: String::new(), tokens, macro_args: Vec::new(), replace_map: HashMap::new(), macro_body_start_idx: 0 }
    }

    pub fn process(&mut self) -> Result<(), MacroError> {
        let tokens_len = self.tokens.len();
        // check for macro name
        if matches!(self.tokens[0], Token::Dollar) {
            if let Token::Literal(s) = &self.tokens[1] {
                if matches!(self.tokens[2], Token::OpenParentheses) {
                    self.name = s.to_string();
                } else {
                    let e = MacroError { ty: MacroErrorT::MissingParentheses, token: self.tokens[2].clone()};
                    return Err(e);
                }
            } else {
                let e = MacroError { ty: MacroErrorT::MissingName, token: self.tokens[1].clone() };
                return Err(e);
            }
        } else if let Token::Literal(s) = &self.tokens[0] {
            self.name = s.to_string();
        } else {
            let e = MacroError { ty: MacroErrorT::MissingName, token: self.tokens[0].clone() };
            return Err(e);
        }

        // check for func args
        if !matches!(self.tokens[3], Token::OpenParentheses) {
            let e = MacroError { ty: MacroErrorT::MissingParentheses, token: self.tokens[2].clone()};
            return Err(e);
        }
        let mut i = 4;;
        
        loop {
            // make sure all tokens till '}' are present in the function args
            if i+3 >= tokens_len {
                let e = MacroError { ty: MacroErrorT::MissingParentheses, token: self.tokens[i-1].clone()};
                return Err(e); 
            }
            if matches!(self.tokens[i], Token::Dollar ) {
                if let Token::Literal(s) = &self.tokens[i + 1] {
                    self.macro_args.push(self.tokens[i + 1].clone());
                } else {
                    let e = MacroError { ty: MacroErrorT::MissingName, token: self.tokens[i+1].clone()};
                    return Err(e);
                }
            } else {
                let e = MacroError { ty: MacroErrorT::InvalidNumberOfDollars, token: self.tokens[i].clone()};
                return Err(e);
            }

            if matches!(self.tokens[i+2], Token::CloseParentheses) {
                i += 3;
                break;
            }
            if matches!(self.tokens[i+2], Token::Comma) {
                if matches!(self.tokens[i+3], Token::CloseParentheses) {
                    i += 4;
                    break;
                } else if matches!(self.tokens[i+3], Token::Dollar) {
                    i += 3;
                } else {
                    let e = MacroError { ty: MacroErrorT::Unexpected, token: self.tokens[i+3].clone()};
                    return Err(e);
                }
            }
        }
        self.macro_body_start_idx = i;

        for idx in i..tokens_len {
            for fields_idx in 0..self.macro_args.len() {
                if self.tokens[idx]  == self.macro_args[fields_idx] {
                    self.replace_map
                        .entry(self.tokens[idx].clone())
                        .or_insert_with(|| Vec::new())
                        .push(idx);
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn query(&mut self, s: &str) -> Result<String, MacroError> {
        let mut crawler = Crawler::new(s);
        let tokens = crawler.tokenize();
        let tokens_len = tokens.len();
        let macro_name = &self.name;
        let mut span_start = 0;
        let mut span_end = 0;
        let mut parentheses_counter: isize = 0;
        let mut idx = 0;
        let mut name_found = false;

        while idx < tokens.len() {
            match &tokens[idx]{
                Token::Literal(lit) => {
                    if lit == macro_name { 
                        span_start = idx-1;
                        idx+=1;
                        loop {
                            if matches!(tokens[idx+1], Token::NonLiteral(_)) { idx+=1; }
                            else { break; }
                        }
                        if matches!(tokens[idx+1], Token::OpenParentheses) { 
                            parentheses_counter = 1; 
                            idx+=2;
                        }
                        name_found = true;
                    } else {
                        idx+=1;
                    } 
                }
                Token::OpenParentheses => { 
                    parentheses_counter += 1;
                    idx += 1;
                }
                Token::CloseParentheses => { 
                    parentheses_counter -= 1; 
                    idx += 1;
                }
                _ => idx+=1,
            }
            if parentheses_counter == 0 && name_found {
                span_end = idx;
                break;
            }
        }
        if !name_found {
            let e = MacroError { ty: MacroErrorT::MissingName, token: tokens[tokens_len-1].clone()};
            return Err(e);
        }
        if span_end == 0 {
            let e = MacroError { ty: MacroErrorT::MissingParentheses, token: tokens[tokens_len-1].clone()};
            return Err(e);
        }
        if matches!(tokens[span_start], Token::Dollar) {
            if let Token::Literal(s) = &tokens[span_start+1] {
                if matches!(tokens[span_start+3], Token::OpenParentheses) {
                    if &self.name != s {
                        let e = MacroError { ty: MacroErrorT::MacroNotFound, token: tokens[span_start+2].clone()};
                        return Err(e);
                    }
                } else {
                    let e = MacroError { ty: MacroErrorT::MissingParentheses, token: tokens[span_start+1].clone()};
                    return Err(e);
                }
            } else {
                let e = MacroError { ty: MacroErrorT::InvalidNumberOfDollars, token: tokens[span_start].clone()};
                return Err(e);
            }
        }
        idx = span_start+4;
        let macro_args_len = self.macro_args.len();
        let mut macro_args_ptr = 0;
        while idx < span_end {
            if macro_args_ptr == macro_args_len { break; }
            if let Token::Literal(s) = &tokens[idx] {
                let macro_name = &self.macro_args[macro_args_ptr];
                let Token::Literal(macro_name_str) = macro_name else { unreachable!() };
                let affected_indices = self.replace_map[&macro_name].clone();
                for &i in &affected_indices { 
                    self.tokens[i] = Token::Literal(s.to_string());
                }
                macro_args_ptr+=1;
            }
            idx+=1
        }
        let mut ret_str = Self::token2str(&tokens[0..3]);
        let suffix = Self::token2str(&self.tokens[self.macro_body_start_idx+1..self.tokens.len()-1]);
        ret_str.push_str(&suffix);
        Ok(ret_str)
    }

    fn token2str(tokens: &[Token]) -> String {
        tokens.iter().map(|t| t.as_str()).collect::<String>()
    }
}