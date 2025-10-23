use regex::Regex;
use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    Operation(String),
    StringLiteral(String),
    Symbol(char),
    Char(char),
    Number(String),
    SemiColon,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    SquareOpen,
    SquareClose,
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

            '[' => tokens.push(Token::SquareOpen),
            ']' => tokens.push(Token::SquareClose),

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

            '\'' => {
                tokens.push(Token::Char(chars[i + 1]));
                i += 1;
            }
            '<' => {
                let mut include_string = "<".to_string();
                let start_i = i;
                i += 1;

                while i < chars.len() {
                    let c = chars[i];
                    include_string.push(c);
                    if c == '>' {
                        tokens.push(Token::Identifier(include_string.clone()));
                        break;
                    }
                    i += 1;
                }

                // If we never found '>', treat it as an operation
                if !include_string.ends_with('>') {
                    i = start_i;
                    tokens.push(Token::Operation("<".to_string()));
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
                }
            }
            '+' => {
                if chars[i + 1] != '=' || chars[i + 1] != '+' {
                    tokens.push(Token::Operation("+".to_string()));
                } else {
                    tokens.push(Token::Operation(format!("+{}", chars[i + 1])));
                    i += 1;
                }
            }
            '*' => {
                if chars[i + 1] != '=' {
                    tokens.push(Token::Operation("*".to_string()));
                } else {
                    tokens.push(Token::Operation("*=".to_string()));
                    i += 1;
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

fn generate_c_target(tokens: Vec<Token>, double: bool) -> String {
    let mut code = String::new();
    let mut i = 0;

    let int_type = if double { "int64_t" } else { "int32_t" };

    while i < tokens.len() {
        match &tokens[i] {
            Token::Identifier(name) => {
                if name == "imp" {
                    // Handle includes
                    if let Some(Token::Identifier(header)) = tokens.get(i + 1) {
                        code += &format!("#include {}\n", header);
                        i += 2;
                        continue;
                    }
                } else if name == "ret" {
                    // Handle return
                    if let Some(next) = tokens.get(i + 1) {
                        let ret_val = match next {
                            Token::Number(n) => n.clone(),
                            Token::Identifier(id) => id.clone(),
                            Token::StringLiteral(s) => format!("\"{}\"", s),
                            _ => "0".to_string(),
                        };
                        code += &format!("return {};\n", ret_val);
                        i += 2;
                        continue;
                    }
                } else {
                    // Check for function definitions: name arg1 arg2 { ...
                    let mut j = i + 1;
                    let mut args = Vec::new();
                    while let Some(Token::Identifier(arg)) = tokens.get(j) {
                        args.push(format!("{} {}", int_type, arg));
                        j += 1;
                    }
                    if let Some(Token::BraceOpen) = tokens.get(j) {
                        code += &format!("{} {}({}) {{\n", int_type, name, args.join(", "));
                        i = j + 1;
                        continue;
                    }

                    // Check for variable declaration: x := value
                    if let Some(Token::Operation(op)) = tokens.get(i + 1) {
                        if op == ":=" {
                            if let Some(value_token) = tokens.get(i + 2) {
                                let value_str = match value_token {
                                    Token::Number(n) => n.clone(),
                                    Token::Identifier(id) => id.clone(),
                                    Token::StringLiteral(s) => format!("\"{}\"", s),
                                    _ => "0".to_string(),
                                };
                                code += &format!("{} {} = {};\n", int_type, name, value_str);
                                i += 3;
                                continue;
                            }
                        }
                    }

                    // Function calls
                    if let Some(Token::ParenOpen) = tokens.get(i + 1) {
                        code += &format!("{}(", name);
                        i += 2;
                        let mut first = true;
                        while let Some(tok) = tokens.get(i) {
                            match tok {
                                Token::ParenClose => {
                                    code += ");\n";
                                    i += 1;
                                    break;
                                }
                                Token::Identifier(id) if id == "mem" || id == "ref" => {
                                    if let Some(Token::Identifier(var)) = tokens.get(i + 1) {
                                        if !first {
                                            code += ", ";
                                        }
                                        let s = if id == "mem" {
                                            format!("&{}", var)
                                        } else {
                                            format!("*{}", var)
                                        };
                                        code += &s;
                                        first = false;
                                        i += 2;
                                        continue;
                                    }
                                }
                                Token::Identifier(id) => {
                                    if !first {
                                        code += ", ";
                                    }
                                    code += id;
                                    first = false;
                                    i += 1;
                                }
                                Token::Number(n) => {
                                    if !first {
                                        code += ", ";
                                    }
                                    code += n;
                                    first = false;
                                    i += 1;
                                }
                                Token::StringLiteral(s) => {
                                    if !first {
                                        code += ", ";
                                    }
                                    code += &format!("\"{}\"", s);
                                    first = false;
                                    i += 1;
                                }
                                _ => {
                                    i += 1;
                                }
                            }
                        }
                        continue;
                    }
                }
            }

            Token::Operation(op) if op == "->" => {
                // if statement
                if i > 0 {
                    let cond = &tokens[i - 1];
                    let cond_str = match cond {
                        Token::Identifier(id) | Token::Number(id) => id.clone(),
                        _ => "".to_string(),
                    };
                    code += &format!("if ({}) {{\n", cond_str);
                } else {
                    code += "{\n";
                }
            }

            Token::Operation(op) if op == "!->" => {
                // else if or else
                if let Some(Token::Identifier(_)) = tokens.get(i + 1) {
                    if let Some(Token::Operation(arrow)) = tokens.get(i + 2) {
                        if arrow == "->" {
                            // else if
                            let mut cond_tokens = Vec::new();
                            let mut k = i + 1;
                            while let Some(tok) = tokens.get(k) {
                                if let Token::Operation(op) = tok {
                                    if op == "->" {
                                        break;
                                    }
                                }
                                cond_tokens.push(tok.clone());
                                k += 1;
                            }
                            let cond_str = cond_tokens
                                .iter()
                                .map(|t| match t {
                                    Token::Identifier(id) | Token::Number(id) => id.clone(),
                                    Token::Operation(op) => op.clone(),
                                    _ => "".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(" ");
                            code += &format!("else if ({}) {{\n", cond_str);
                            i += cond_tokens.len() + 2;
                            continue;
                        }
                    }
                }
                // else
                code += "else {\n";
            }

            Token::BraceOpen => code += "{\n",
            Token::BraceClose => code += "}\n",
            Token::Number(n) => code += n,
            Token::StringLiteral(s) => code += &format!("\"{}\"", s),
            Token::Char(c) => code += &format!("'{}'", c),
            Token::SemiColon => code += ";\n",
            _ => {}
        }

        i += 1;
    }

    code
}

fn parse(code_raw: String, args: Vec<String>) -> String {
    let cleaned = clean_code(code_raw);
    let tokens = tokenize(&cleaned);

    println!("{:?}", tokens);

    if args.contains(&"-compat".to_string()) {
        return generate_c_target(tokens.clone(), false);
    } else if args.contains(&"-fmbyas".to_string()) {
        println!("Build target -FMBYAS not implemented yet");
    } else {
        return generate_c_target(tokens.clone(), true);
    }

    cleaned
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }

    let filename = args[1].clone();
    let contents = fs::read_to_string(filename)?; // reads the whole file into a String

    println!("{}", parse(contents, args));

    Ok(())
}
