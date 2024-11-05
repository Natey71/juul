use crate::lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    PrintStatement(Box<ASTNode>),
    VariableAssignment(String, Box<ASTNode>),
    IfStatement {
        condition: Box<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<ASTNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
    },
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    BinaryExpression {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    UnaryExpression {
        operator: Token,
        operand: Box<ASTNode>,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Vec<ASTNode>, String> {
    let mut tokens = tokens.iter().peekable();
    let mut ast = Vec::new();

    while let Some(token) = tokens.peek() {
        if **token == Token::EOF {
            break;
        }
        let node = parse_statement(&mut tokens)?;
        ast.push(node);
    }
    Ok(ast)
}

fn parse_statement(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    match tokens.peek() {
        Some(Token::Print) => parse_print_statement(tokens),
        Some(Token::If) => parse_if_statement(tokens),
        Some(Token::Function) => parse_function_declaration(tokens),
        Some(Token::Identifier(_)) => parse_assignment_or_expression_statement(tokens),
        Some(Token::Semicolon) => {
            tokens.next(); // Consume ';'
            Ok(ASTNode::Program(Vec::new())) // Empty statement
        }
        Some(Token::EOF) => Ok(ASTNode::Program(Vec::new())),
        _ => Err("Unexpected token in statement.".into()),
    }
}

fn parse_print_statement(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    tokens.next(); // Consume 'print'
    let expr = parse_expression(tokens)?;
    expect_token(tokens, Token::Semicolon)?;
    Ok(ASTNode::PrintStatement(Box::new(expr)))
}

fn parse_if_statement(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    tokens.next(); // Consume 'if'
    expect_token(tokens, Token::LeftParen)?;
    let condition = parse_expression(tokens)?;
    expect_token(tokens, Token::RightParen)?;
    expect_token(tokens, Token::LeftBrace)?;
    let then_branch = parse_block(tokens)?;
    let else_branch = if let Some(Token::Else) = tokens.peek() {
        tokens.next(); // Consume 'else'
        expect_token(tokens, Token::LeftBrace)?;
        Some(parse_block(tokens)?)
    } else {
        None
    };
    Ok(ASTNode::IfStatement {
        condition: Box::new(condition),
        then_branch,
        else_branch,
    })
}

fn parse_function_declaration(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    tokens.next(); // Consume 'function'
    let name = if let Some(Token::Identifier(name)) = tokens.next() {
        name.clone()
    } else {
        return Err("Expected function name.".into());
    };
    expect_token(tokens, Token::LeftParen)?;
    let parameters = parse_parameters(tokens)?;
    expect_token(tokens, Token::RightParen)?;
    expect_token(tokens, Token::LeftBrace)?;
    let body = parse_block(tokens)?;
    Ok(ASTNode::FunctionDeclaration {
        name,
        parameters,
        body,
    })
}

fn parse_parameters(tokens: &mut Peekable<Iter<Token>>) -> Result<Vec<String>, String> {
    let mut params = Vec::new();
    while let Some(token) = tokens.peek() {
        match token {
            Token::Identifier(name) => {
                let name = name.clone();
                tokens.next(); // Consume identifier
                params.push(name);
                if let Some(Token::Comma) = tokens.peek() {
                    tokens.next(); // Consume ','
                } else {
                    break;
                }
            }
            Token::RightParen => break,
            _ => return Err("Unexpected token in parameter list.".into()),
        }
    }
    Ok(params)
}

fn parse_block(tokens: &mut Peekable<Iter<Token>>) -> Result<Vec<ASTNode>, String> {
    let mut statements = Vec::new();
    while let Some(token) = tokens.peek() {
        if **token == Token::RightBrace {
            tokens.next(); // Consume '}'
            break;
        }
        let stmt = parse_statement(tokens)?;
        statements.push(stmt);
    }
    Ok(statements)
}

fn parse_assignment_or_expression_statement(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let expr = parse_expression(tokens)?;
    if let Some(Token::Assign) = tokens.peek() {
        if let ASTNode::Identifier(name) = expr {
            tokens.next(); // Consume '='
            let value = parse_expression(tokens)?;
            expect_token(tokens, Token::Semicolon)?;
            Ok(ASTNode::VariableAssignment(name, Box::new(value)))
        } else {
            Err("Invalid assignment target.".into())
        }
    } else {
        expect_token(tokens, Token::Semicolon)?;
        Ok(expr)
    }
}

fn parse_expression(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    parse_equality(tokens)
}

fn parse_equality(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let mut expr = parse_comparison(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::EqualEqual | Token::NotEqual => {
                let operator = tokens.next().unwrap().clone();
                let right = parse_comparison(tokens)?;
                expr = ASTNode::BinaryExpression {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn parse_comparison(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let mut expr = parse_addition(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual => {
                let operator = tokens.next().unwrap().clone();
                let right = parse_addition(tokens)?;
                expr = ASTNode::BinaryExpression {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn parse_addition(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let mut expr = parse_multiplication(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Plus | Token::Minus => {
                let operator = tokens.next().unwrap().clone();
                let right = parse_multiplication(tokens)?;
                expr = ASTNode::BinaryExpression {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn parse_multiplication(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let mut expr = parse_unary(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Star | Token::Slash => {
                let operator = tokens.next().unwrap().clone();
                let right = parse_unary(tokens)?;
                expr = ASTNode::BinaryExpression {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Ok(expr)
}

fn parse_unary(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    if let Some(token) = tokens.peek() {
        match token {
            Token::Minus | Token::Bang => {
                let operator = tokens.next().unwrap().clone();
                let operand = parse_unary(tokens)?;
                return Ok(ASTNode::UnaryExpression {
                    operator,
                    operand: Box::new(operand),
                });
            }
            _ => {}
        }
    }
    parse_primary(tokens)
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    match tokens.next() {
        Some(Token::NumberLiteral(n)) => Ok(ASTNode::NumberLiteral(*n)),
        Some(Token::StringLiteral(s)) => Ok(ASTNode::StringLiteral(s.clone())),
        Some(Token::Identifier(name)) => {
            if let Some(Token::LeftParen) = tokens.peek() {
                tokens.next(); // Consume '('
                let arguments = parse_arguments(tokens)?;
                expect_token(tokens, Token::RightParen)?;
                Ok(ASTNode::FunctionCall {
                    name: name.clone(),
                    arguments,
                })
            } else {
                Ok(ASTNode::Identifier(name.clone()))
            }
        }
        Some(Token::LeftParen) => {
            let expr = parse_expression(tokens)?;
            expect_token(tokens, Token::RightParen)?;
            Ok(expr)
        }
        _ => Err("Expected an expression.".into()),
    }
}

fn parse_arguments(tokens: &mut Peekable<Iter<Token>>) -> Result<Vec<ASTNode>, String> {
    let mut args = Vec::new();
    while let Some(token) = tokens.peek() {
        if **token == Token::RightParen {
            break;
        }
        let arg = parse_expression(tokens)?;
        args.push(arg);
        if let Some(Token::Comma) = tokens.peek() {
            tokens.next(); // Consume ','
        } else {
            break;
        }
    }
    Ok(args)
}

fn expect_token(tokens: &mut Peekable<Iter<Token>>, expected: Token) -> Result<(), String> {
    if let Some(token) = tokens.next() {
        if *token == expected {
            Ok(())
        } else {
            Err(format!("Expected token {:?}, found {:?}", expected, token))
        }
    } else {
        Err(format!("Expected token {:?}, found end of input", expected))
    }
}
