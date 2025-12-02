use std::{
    error::Error,
    collections::{HashMap, VecDeque},
};

use regex::Regex;
use super::{tokens::{Token, TokenType}, read_lines};

// Regex patterns for different token types

pub fn tokenize(path: &str, keywords: HashMap<&str, TokenType>) -> Result<VecDeque<Token>, Box<dyn Error>> {
    let mut tokens: VecDeque<Token> = VecDeque::new();

    let number_regex = Regex::new(r"^-?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$").unwrap();
    let identifier_regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    let file_path_regex = Regex::new(r"^(\.{0,2}/)?([a-zA-Z0-9_\-./]*[a-zA-Z0-9_\-])?\.([a-zA-Z0-9]+)$").unwrap();

    let lines = read_lines(path).map_err(|_| format!("Script '{}' not found", path))?;

    let mut iterator = lines.map_while(Result::ok).enumerate();

    while let Some((line_number, line)) = iterator.next() {
        let line = line.trim();

        // ignore comments
        if line.starts_with("//") || line.starts_with("#") || line.is_empty() {
            continue;
        }

        // convert line to token strings
        let current = line.split_whitespace().collect::<Vec<&str>>();

        for token in current {
            // keyword
            if let Some(token_type) = keywords.get(token) {
                tokens.push_back(Token { 
                    value: token.to_string(), 
                    token_type: token_type.clone() 
                });
            } 
            
            // number
            else if number_regex.is_match(token) {
                tokens.push_back(Token {
                    value: token.to_string(),
                    token_type: TokenType::Number,
                });
            } 
            
            // file path
            else if file_path_regex.is_match(token) {
                tokens.push_back(Token {
                    value: token.to_string(),
                    token_type: TokenType::FilePath,
                });
            } 
            
            // identifier
            else if identifier_regex.is_match(token) {
                tokens.push_back(Token {
                    value: token.to_string(),
                    token_type: TokenType::Identifier,
                });
            } 
            
            else {
                return Err(format!("{}:{} Token not recognized: {}", path, line_number + 1, token).into());
            }
        }
    }

    Ok(tokens)
}
