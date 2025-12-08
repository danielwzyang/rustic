use std::{
    error::Error,
    collections::{HashMap, VecDeque},
};

use regex::Regex;
use super::{tokens::{Token, TokenType}, read_lines};

pub fn tokenize(path: &str, keywords: HashMap<&str, TokenType>) -> Result<VecDeque<Token>, Box<dyn Error>> {
    let token_regex = Regex::new(r"(?x)
        (?P<Comment>//) |
        (?P<WhiteSpace> \s+) |
        (?P<Number> -?(\d+\.?\d*|\.\d+)) |
        (?P<FilePath>(?:\./|\../|[A-Za-z0-9_\-]+/)*[A-Za-z0-9_\-]+\.[A-Za-z0-9]+) |
        (?P<Identifier> [a-zA-Z_][a-zA-Z0-9_]*) |
        (?P<Unknown> \S)"
    ).unwrap();

    let mut tokens: VecDeque<Token> = VecDeque::new();
    let lines = read_lines(path).map_err(|_| format!("Script '{}' not found", path))?; 

    for (line_number, line) in lines.map_while(Result::ok).enumerate() {
        let line = line.trim();

        for captures in token_regex.captures_iter(line) {
            if captures.name("Comment").is_some() {
                break;
            } else if captures.name("WhiteSpace").is_some() {
                continue;
            } else if let Some(number) = captures.name("Number") {
                tokens.push_back(Token {
                    value: number.as_str().to_string(),
                    token_type: TokenType::Number,
                });
            } else if let Some(file_path) = captures.name("FilePath") {
                tokens.push_back(Token {
                    value: file_path.as_str().to_string(),
                    token_type: TokenType::FilePath,
                });
            } else if let Some(identifier) = captures.name("Identifier") {
                let identifier = identifier.as_str();

                let token_type = keywords.get(identifier).cloned().unwrap_or(TokenType::Identifier);
                
                tokens.push_back(Token {
                    value: identifier.to_string(),
                    token_type,
                });
            } else if let Some(unknown) = captures.name("Unknown") {
                return Err(format!("{}:{} Token not recognized: {}", path, line_number + 1, unknown.as_str()).into());
            }
        }
    }

    Ok(tokens)
}
