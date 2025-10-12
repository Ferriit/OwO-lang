use regex::Regex;
use std::env;
use std::fs;

fn parse(code_raw: String) {
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
        .replace("{ ", "{");

    println!("{:?}", processed_code);
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }

    let filename = args[1].clone();
    let contents = fs::read_to_string(filename)?; // reads the whole file into a String

    parse(contents);

    Ok(())
}
