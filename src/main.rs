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

fn generate_asm_target(tokens: Vec<Token>, double: bool) -> String {
    let mut asm = String::new();
    let mut i = 0;

    let reg_size = if double { 8 } else { 4 };
    let mut stack_offset: i32 = 0;
    let mut var_stack_offsets = std::collections::HashMap::new();

    let mut label_counter = 0;
    let mut loop_stack: Vec<(String, String)> = Vec::new(); // (loop_start, loop_end)
    let mut if_stack: Vec<String> = Vec::new(); // store endif labels for nested ifs

    while i < tokens.len() {
        match &tokens[i] {
            Token::Identifier(name) => {
                match name.as_str() {
                    "ret" => {
                        if let Some(next) = tokens.get(i + 1) {
                            match next {
                                Token::Number(n) => asm += &format!("    mov rax, {}\n", n),
                                Token::Identifier(id) => {
                                    if let Some(off) = var_stack_offsets.get(id) {
                                        asm += &format!("    mov rax, [rbp{}]\n", off);
                                    }
                                }
                                _ => {}
                            }
                        }
                        asm += "    leave\n    ret\n";
                        i += 2;
                        continue;
                    }

                    "mem" => {
                        if let Some(Token::Identifier(var)) = tokens.get(i + 1) {
                            stack_offset -= reg_size;
                            var_stack_offsets.insert(var.clone(), stack_offset);
                            if let Some(Token::Number(n)) = tokens.get(i + 2) {
                                asm += &format!(
                                    "    mov rax, {}\n    mov [rbp{}], rax\n",
                                    n, stack_offset
                                );
                                i += 1;
                            }
                            i += 2;
                            continue;
                        }
                    }

                    "ref" => {
                        if let Some(Token::Identifier(var)) = tokens.get(i + 1) {
                            if let Some(off) = var_stack_offsets.get(var) {
                                asm += &format!("    mov rax, [rbp{}]\n", off);
                            }
                            i += 2;
                            continue;
                        }
                    }

                    "loop" => {
                        let start_label = format!(".loop_start{}", label_counter);
                        let end_label = format!(".loop_end{}", label_counter);
                        label_counter += 1;
                        loop_stack.push((start_label.clone(), end_label.clone()));
                        asm += &format!("{}:\n", start_label);
                        i += 1;
                        continue;
                    }

                    "brk" => {
                        if let Some((_, end_label)) = loop_stack.last() {
                            asm += &format!("    jmp {}\n", end_label);
                        }
                        i += 1;
                        continue;
                    }

                    "jump" => {
                        if let Some(Token::Identifier(label)) = tokens.get(i + 1) {
                            asm += &format!("    jmp {}\n", label);
                            i += 2;
                            continue;
                        }
                    }

                    "imp" => {
                        if let Some(Token::Identifier(path)) = tokens.get(i + 1) {
                            asm += &format!("    ; import {}\n", path);
                            i += 2;
                            continue;
                        }
                    }

                    "asm" => {
                        // Insert raw assembly until semicolon
                        let mut j = i + 1;
                        while let Some(tok) = tokens.get(j) {
                            match tok {
                                Token::SemiColon => break,
                                Token::Identifier(s) | Token::Number(s) => {
                                    asm += &format!("{} ", s)
                                }
                                _ => {}
                            }
                            j += 1;
                        }
                        asm += "\n";
                        i = j + 1;
                        continue;
                    }

                    _ => {
                        // Function definitions
                        let mut args = Vec::new();
                        let mut j = i + 1;
                        while let Some(Token::Identifier(arg)) = tokens.get(j) {
                            args.push(arg.clone());
                            j += 1;
                        }
                        if let Some(Token::BraceOpen) = tokens.get(j) {
                            asm += &format!("{}:\n    push rbp\n    mov rbp, rsp\n", name);
                            stack_offset = 0;
                            for arg in &args {
                                stack_offset -= reg_size;
                                var_stack_offsets.insert(arg.clone(), stack_offset);
                                asm += &format!("    ; arg {} at [rbp{}]\n", arg, stack_offset);
                            }
                            i = j + 1;
                            continue;
                        }
                    }
                }
            }

            Token::Operation(op) => {
                match op.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if i > 0 && i + 1 < tokens.len() {
                            let lhs = &tokens[i - 1];
                            let rhs = &tokens[i + 1];
                            let lhs_offset = if let Token::Identifier(id) = lhs {
                                var_stack_offsets.get(id).copied()
                            } else {
                                None
                            };
                            // Load lhs
                            if let Some(off) = lhs_offset {
                                asm += &format!("    mov rax, [rbp{}]\n", off);
                            } else if let Token::Number(n) = lhs {
                                asm += &format!("    mov rax, {}\n", n);
                            }
                            // Load rhs and operate
                            if let Token::Identifier(id) = rhs {
                                if let Some(off) = var_stack_offsets.get(id) {
                                    asm += &format!("    mov rbx, [rbp{}]\n", off);
                                }
                            } else if let Token::Number(n) = rhs {
                                asm += &format!("    mov rbx, {}\n", n);
                            }
                            let instr = match op.as_str() {
                                "+" => "add rax, rbx",
                                "-" => "sub rax, rbx",
                                "*" => "imul rax, rbx",
                                "/" => "cqo\n    idiv rbx",
                                _ => "",
                            };
                            asm += &format!("    {}\n", instr);
                            // Store result back if lhs is a variable
                            if let Some(off) = lhs_offset {
                                asm += &format!("    mov [rbp{}], rax\n", off);
                            }
                            i += 2;
                            continue;
                        }
                    }

                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        if i > 0 && i + 1 < tokens.len() {
                            let lhs = &tokens[i - 1];
                            let rhs = &tokens[i + 1];
                            let lhs_val = if let Token::Identifier(id) = lhs {
                                var_stack_offsets.get(id).copied()
                            } else {
                                None
                            };
                            if let Some(off) = lhs_val {
                                asm += &format!("    mov rax, [rbp{}]\n", off);
                            }
                            let rhs_val = if let Token::Identifier(id) = rhs {
                                var_stack_offsets.get(id).copied()
                            } else {
                                None
                            };
                            if let Some(off) = rhs_val {
                                asm += &format!("    mov rbx, [rbp{}]\n", off);
                            }
                            let instr = match op.as_str() {
                                "==" => "cmp rax, rbx\n    sete al\n    movzx rax, al",
                                "!=" => "cmp rax, rbx\n    setne al\n    movzx rax, al",
                                "<" => "cmp rax, rbx\n    setl al\n    movzx rax, al",
                                ">" => "cmp rax, rbx\n    setg al\n    movzx rax, al",
                                "<=" => "cmp rax, rbx\n    setle al\n    movzx rax, al",
                                ">=" => "cmp rax, rbx\n    setge al\n    movzx rax, al",
                                _ => "",
                            };
                            asm += &format!("    {}\n", instr);
                            i += 2;
                            continue;
                        }
                    }

                    "->" => {
                        // if statement
                        let label_id = label_counter;
                        label_counter += 1;
                        let end_label = format!(".if_end{}", label_id);
                        if_stack.push(end_label.clone());
                        asm += &format!("    cmp rax, 0\n    je {}\n", end_label);
                    }

                    "!->" => {
                        // else: jump to end of previous if
                        if let Some(end_label) = if_stack.pop() {
                            asm += &format!("    jmp {}\n", end_label);
                        }
                    }

                    _ => {}
                }
            }

            Token::BraceClose => {
                // Close loops or if statements
                if let Some((_, end_label)) = loop_stack.pop() {
                    asm += &format!("{}:\n", end_label);
                }
                if let Some(end_label) = if_stack.pop() {
                    asm += &format!("{}:\n", end_label);
                }
            }

            Token::Number(n) => {
                asm += &format!("    mov rax, {}\n", n);
            }

            Token::StringLiteral(s) => {
                // Strings: reserve static label
                let label = format!(".str{}", label_counter);
                label_counter += 1;
                asm += &format!("{}: db '{}', 0\n", label, s);
            }

            _ => {}
        }

        i += 1;
    }

    asm
}

fn parse(code_raw: String, args: Vec<String>) -> String {
    let cleaned = clean_code(code_raw);
    let tokens = tokenize(&cleaned);

    println!("{:?}", tokens);

    if args.contains(&"-compat".to_string()) {
        return generate_asm_target(tokens.clone(), false);
    } else if args.contains(&"-fmbyas".to_string()) {
        println!("Build target -FMBYAS not implemented yet");
    } else {
        return generate_asm_target(tokens.clone(), true);
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
