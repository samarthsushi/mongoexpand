use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Literal(String),
    NonLiteral(String),
    OpenParentheses,
    CloseParentheses,
    Dollar,
    Comma
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

#[derive(Debug, PartialEq, Eq)]
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
    idx: usize
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum QueryErrorT {
    InvalidNumberOfDollars,
    SuspendedDollar,
    MissingComma,
    MissingParentheses,
    MissingName,
    Unexpected,
    MacroNotFound
}

#[derive(Debug)]
pub struct QueryError {
    ty: QueryErrorT,
    idx: usize
}

#[derive(Debug)]
pub enum ErrorCore {
    Macro(MacroError),
    Query(QueryError)
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
    pub fn new(tokens: Vec<Token>) -> Result<Self, String> {
        let mut mp = Self { name: String::new(), tokens, macro_args: Vec::new(), replace_map: HashMap::new(), macro_body_start_idx: 0 };
        match mp.process() {
            Err(e) => Err(Self::fmt_err(ErrorCore::Macro(e), &mp.tokens)),
            Ok(_) => Ok(mp)
        }
    }

    pub fn process(&mut self) -> Result<(), MacroError> {
        let tokens_len = self.tokens.len();
        // check for macro name
        if matches!(self.tokens[0], Token::Dollar) {
            if let Token::Literal(s) = &self.tokens[1] {
                if matches!(self.tokens[2], Token::OpenParentheses) {
                    self.name = s.to_string();
                } else {
                    let e = MacroError { ty: MacroErrorT::MissingParentheses, idx: 2 };
                    return Err(e);
                }
            } else {
                let e = MacroError { ty: MacroErrorT::MissingName, idx: 2 };
                return Err(e);
            }
        } else if let Token::Literal(s) = &self.tokens[0] {
            self.name = s.to_string();
        } else {
            let e = MacroError { ty: MacroErrorT::MissingName, idx: 0 };
            return Err(e);
        }

        // check for func args
        if !matches!(self.tokens[3], Token::OpenParentheses) {
            let e = MacroError { ty: MacroErrorT::MissingParentheses, idx: 0 };
            return Err(e);
        }
        let mut i = 4;
        
        loop {
            // make sure all tokens till '}' are present in the function args
            if i+3 >= tokens_len {
                let e = MacroError { ty: MacroErrorT::MissingParentheses, idx: i-1};
                return Err(e); 
            }
            if matches!(self.tokens[i], Token::Dollar ) {
                if let Token::Literal(s) = &self.tokens[i + 1] {
                    self.macro_args.push(self.tokens[i + 1].clone());
                } else {
                    let e = MacroError { ty: MacroErrorT::MissingName, idx: i + 1};
                    return Err(e);
                }
            } else {
                let e = MacroError { ty: MacroErrorT::InvalidNumberOfDollars, idx: i};
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
                    let e = MacroError { ty: MacroErrorT::Unexpected, idx: i + 3};
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

    fn query_util(&mut self, tokens: &Vec<Token>) -> Result<String, ErrorCore> {
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
            let e = ErrorCore::Query(QueryError{ ty: QueryErrorT::MissingName, idx: tokens_len - 1 });
            return Err(e);
        }
        if span_end == 0 {
            let e = ErrorCore::Query(QueryError{ ty: QueryErrorT::MissingParentheses, idx: tokens_len - 1});
            return Err(e);
        }
        if matches!(tokens[span_start], Token::Dollar) {
            if let Token::Literal(s) = &tokens[span_start+1] {
                if matches!(tokens[span_start+3], Token::OpenParentheses) {
                    if &self.name != s {
                        let e = ErrorCore::Query(QueryError{ ty: QueryErrorT::MacroNotFound, idx: span_start + 2 });
                        return Err(e);
                    }
                } else {
                    let e = ErrorCore::Query(QueryError { ty: QueryErrorT::MissingParentheses, idx: span_start + 1});
                    return Err(e);
                }
            } else {
                let e = ErrorCore::Query(QueryError { ty: QueryErrorT::InvalidNumberOfDollars, idx: span_start});
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

    pub fn query(&mut self, s: &str) -> String {
        let mut crawler = Crawler::new(s);
        let tokens = crawler.tokenize();

        match self.query_util(&tokens) {
            Err(e) => {
                let ret_str = match e {
                    ErrorCore::Macro(_) => Self::fmt_err(e, &self.tokens),
                    ErrorCore::Query(_) => Self::fmt_err(e, &tokens)
                };
                ret_str
            }
            Ok(x) => {
                x
            }
        }
    }

    fn token2str(tokens: &[Token]) -> String {
        tokens.iter().map(|t| t.as_str()).collect::<String>()
    }

    fn errtoken2str(tokens: &[Token], e_idx: usize) -> (String, usize) {
        let mut current_pos = 0;
        let mut result = String::new();
        for (i, token) in tokens.iter().enumerate() {
            if i == e_idx {
                return (result.clone() + token.as_str(), current_pos);
            }
            current_pos += token.as_str().len();
            result.push_str(token.as_str());
        }
        (result, current_pos)
    }

    fn fmt_err(e: ErrorCore, tokens: &[Token]) -> String {
        let e_idx = match e {
            ErrorCore::Macro(MacroError{idx,..}) => idx,
            ErrorCore::Query(QueryError{idx,..}) => idx,
        };
        let (mut s, mut cursor) = Self::errtoken2str(tokens, e_idx);
        let lines: Vec<&str> = s.lines().collect();
        let mut line_num = 0;
        let mut col_num = 0;
        let mut current_index = 0;

        for (i, line) in lines.iter().enumerate() {
            if current_index + line.len() >= cursor {
                line_num = i + 1;
                col_num = cursor - current_index + 1;
                break;
            }
            current_index += line.len() + 1;
        }
        let error_line = lines.get(line_num - 1).unwrap_or(&"");
        let mut marker_line = String::new();
        marker_line.extend(" ".repeat(col_num - 1).chars());
        marker_line.push('^');
        format!(
            "::{:?}\n   --> line {}, column {}\n   |\n{:3}| {}\n   | {}\n",
            e,
            line_num,
            col_num,
            line_num,
            error_line,
            marker_line
        )
    }
}

// pub struct MacroEngine {
//     macros: Vec<MacroProcessor>
// }

// impl MacroEngine {
//     pub fn new() -> Self {
//         Self { macros: Vec::new() }
//     }

//     pub fn add(&mut self, s: &str) {
//         let mut c = Crawler::new(s);
//         let mut mp = match MacroProcessor::new(c.tokenize()) {
//             Ok(x) => x,
//             Err(e) =>{
//                 println!("{e}");
//                 return;
//             }
//         };
//         self.macros.push(mp);
//     }

//     pub fn query(&mut self, q: &str) {

//     }
// }