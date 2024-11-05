use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Print,
    If,
    Else,
    Function,
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Bang,
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    EOF, 
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next(); // Skip whitespace
            }
            c if c.is_alphabetic() || c == '_' => {
                // Identifiers and keywords
                let ident = collect_identifier(&mut chars);
                match ident.as_str() {
                    "print" => tokens.push(Token::Print),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "func" => tokens.push(Token::Function),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            c if c.is_digit(10) => {
                // Numbers
                let number = collect_number(&mut chars);
                tokens.push(Token::NumberLiteral(number));
            }
            '"' => {
                // String literals
                chars.next(); // Consume '"'
                let string_lit = collect_string_literal(&mut chars);
                tokens.push(Token::StringLiteral(string_lit));
            }
            '=' => {
                chars.next(); // Consume '='
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '!' => {
                chars.next(); // Consume '!'
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::NotEqual);
                } else {
                    tokens.push(Token::Bang);
                }
            }
            '<' => {
                chars.next(); // Consume '<'
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '>' => {
                chars.next(); // Consume '>'
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RightBrace);
            }
            _ => {
                // Unknown character
                chars.next(); // Consume the character to prevent infinite loop
                // Optionally, handle or report an error here
            }
        }
    }

    tokens.push(Token::EOF); // Add EOF token at the end
    tokens
}

fn collect_identifier(chars: &mut Peekable<Chars>) -> String {
    let mut ident = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
            chars.next();
        } else {
            break;
        }
    }
    ident
}

fn collect_number(chars: &mut Peekable<Chars>) -> f64 {
    let mut num_str = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }
    num_str.parse::<f64>().unwrap_or(0.0) // Handle parse errors appropriately
}

fn collect_string_literal(chars: &mut Peekable<Chars>) -> String {
    let mut string_lit = String::new();
    while let Some(&c) = chars.peek() {
        if c != '"' {
            string_lit.push(c);
            chars.next();
        } else {
            chars.next(); // Consume closing '"'
            break;
        }
    }
    string_lit
}
