use regex::Regex;
use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    Operation(String),
    StringLiteral(String),
    Symbol(char),
    Number(String),
    SemiColon,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Array(String),
}

fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let chars: Vec<char> = code.chars().collect();

    let mut i = 0;

    while i < chars.len() {
        let mut c = chars[i];
        match c {
            '(' => tokens.push(Token::ParenOpen),
            ')' => tokens.push(Token::ParenClose),
            '{' => tokens.push(Token::BraceOpen),
            '}' => tokens.push(Token::BraceClose),
            ';' => tokens.push(Token::SemiColon),

            '"' => {
                i += 1;
                c = chars[i];
                let mut string_literal: String = String::new();
                while (c != '"' || chars[i - 1] == '\\') && i < chars.len() {
                    string_literal += c.to_string().as_str();
                    i += 1;
                    c = chars[i];
                }
                tokens.push(Token::StringLiteral(string_literal));
            }

            '<' => {
                let mut include = false;
                let mut found_space = false;

                let mut include_string = "<".to_string();

                let j = i;

                i += 1;
                c = chars[i];
                while c != ';' && i < chars.len() {
                    include_string += c.to_string().as_str();
                    if c == '>' && !found_space {
                        include = true;
                        break;
                    } else if c.is_whitespace() {
                        found_space = true;
                    }
                    i += 1;
                    c = chars[i];
                }

                if include {
                    tokens.push(Token::Identifier(include_string));
                } else {
                    i = j;
                    c = chars[i + 1];
                    if c == '=' || c == '<' {
                        tokens.push(Token::Operation(format!("<{}", c)));
                    } else {
                        tokens.push(Token::Operation("<".to_string()));
                    }
                }
            }
            '>' => {
                if chars[i + 1] != '=' || chars[i + 1] != '>' {
                    tokens.push(Token::Operation(">".to_string()));
                } else {
                    tokens.push(Token::Operation(format!(">{}", c)));
                }
            }
            '-' => {
                if chars[i + 1] != '=' && chars[i + 1] != '>' && chars[i + 1] != '-' {
                    tokens.push(Token::Operation("-".to_string()));
                } else {
                    tokens.push(Token::Operation(format!("-{}", chars[i + 1])));
                    i += 1;
                    c = chars[i];
                }
            }
            '+' => {
                if chars[i + 1] != '=' || chars[i + 1] != '+' {
                    tokens.push(Token::Operation("+".to_string()));
                } else {
                    tokens.push(Token::Operation(format!("+{}", chars[i + 1])));
                    i += 1;
                    c = chars[i];
                }
            }
            '*' => {
                if chars[i + 1] != '=' {
                    tokens.push(Token::Operation("*".to_string()));
                } else {
                    tokens.push(Token::Operation("*=".to_string()));
                    i += 1;
                    c = chars[i];
                }
            }
            '/' => {
                if chars[i + 1] != '=' {
                    tokens.push(Token::Operation("/".to_string()));
                } else {
                    tokens.push(Token::Operation("/=".to_string()));
                }
            }
            '=' => {
                if chars[i + 1] != '=' {
                    tokens.push(Token::Operation("=".to_string()));
                } else {
                    tokens.push(Token::Operation("==".to_string()));
                    i += 1;
                    c = chars[i];
                }
            }
            '!' => {
                if chars[i + 1] == '-' && chars[i + 2] == '>' {
                    tokens.push(Token::Operation("!->".to_string()));
                    i += 3;
                } else if chars[i + 1] != '=' {
                    tokens.push(Token::Operation("!".to_string()));
                    i += 1;
                } else {
                    tokens.push(Token::Operation("!=".to_string()));
                    i += 1;
                }
            }
            '&' => {
                if chars[i + 1] != '&' {
                    tokens.push(Token::Operation("&".to_string()));
                } else {
                    tokens.push(Token::Operation("&&".to_string()));
                    i += 1;
                }
            }
            '|' => {
                if chars[i + 1] != '|' {
                    tokens.push(Token::Operation("|".to_string()));
                } else {
                    tokens.push(Token::Operation("||".to_string()));
                    i += 1;
                }
            }
            ':' => {
                if chars[i + 1] == '=' {
                    tokens.push(Token::Operation(":=".to_string()));
                    i += 1;
                }
            }
            _ if c.is_whitespace() => {
                i += 1;
                continue;
            }

            _ if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::new();

                while c.is_alphabetic() || c == '_' {
                    identifier += c.to_string().as_str();
                    i += 1;
                    c = chars[i];
                }
                i -= 1;
                c = chars[i];
                tokens.push(Token::Identifier(identifier));
            }
            _ if c.is_ascii_digit() => {
                let mut number = String::new();

                while c.is_ascii_digit() {
                    number += c.to_string().as_str();
                    i += 1;
                    c = chars[i];
                }
                i -= 1;
                c = chars[i];
                tokens.push(Token::Number(number));
            }

            _ => tokens.push(Token::Symbol(c)),
        }
        i += 1;
    }

    tokens
}

fn clean_code(code_raw: String) -> String {
    // Split the code by newlines
    let code: Vec<String> = code_raw.split('\n').map(|s| s.to_string()).collect();

    let mut clean_code: Vec<String> = Vec::new();

    for line_str in code {
        let mut line = String::new();
        let mut chars = line_str.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '/' {
                if let Some('/') = chars.peek() {
                    break; // found "//", ignore the rest of the line
                } else {
                    line.push(c);
                }
            } else {
                line.push(c);
            }
        }

        clean_code.push(line);
    }

    let whitespace_re = Regex::new(r"\s{2,}").unwrap();
    let joined_code = &clean_code.join("");
    let processed_code = whitespace_re
        .replace_all(joined_code, " ")
        .replace("; ", ";")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace(" )", ")");

    processed_code
}

fn parse(code_raw: String) -> String {
    let cleaned = clean_code(code_raw);
    let tokens = tokenize(&cleaned);

    println!("{:?}", tokens);
    cleaned
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }

    let filename = args[1].clone();
    let contents = fs::read_to_string(filename)?; // reads the whole file into a String

    println!("{}", parse(contents));

    Ok(())
}
