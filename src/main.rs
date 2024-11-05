mod lexer;
mod parser;

use lexer::lex;
use lexer::Token;
use parser::{parse, ASTNode};
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: juul <source_file>");
        return;
    }

    let filename = &args[1];
    let code = fs::read_to_string(filename).expect("Could not read file");

    let tokens = lex(&code);
    // Uncomment the following line to debug tokens
    // println!("{:?}", tokens);

    match parse(&tokens) {
        Ok(ast_nodes) => {
            interpret(ast_nodes);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn interpret(ast_nodes: Vec<ASTNode>) {
    let mut global_env = Environment::new(None);
    let mut functions = HashMap::new();

    for node in ast_nodes {
        execute(node, &mut global_env, &mut functions);
    }
}

#[derive(Clone)]
struct Environment {
    values: HashMap<String, ASTNode>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    fn new(enclosing: Option<Box<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    fn define(&mut self, name: String, value: ASTNode) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<ASTNode> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    fn assign(&mut self, name: &str, value: ASTNode) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name, value)
        } else {
            false
        }
    }
}

fn execute(node: ASTNode, env: &mut Environment, functions: &mut HashMap<String, ASTNode>) {
    match node {
        ASTNode::PrintStatement(expr) => {
            let value = evaluate(*expr, env, functions);
            match value {
                ASTNode::StringLiteral(s) => println!("{}", s),
                ASTNode::NumberLiteral(n) => println!("{}", n),
                _ => println!("Cannot print value"),
            }
        }
        ASTNode::VariableAssignment(name, expr) => {
            let value = evaluate(*expr, env, functions);
            if !env.assign(&name, value.clone()) {
                env.define(name, value);
            }
        }
        ASTNode::IfStatement {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_value = evaluate(*condition, env, functions);
            if is_truthy(&cond_value) {
                let mut then_env = Environment::new(Some(Box::new(env.clone())));
                for stmt in then_branch {
                    execute(stmt, &mut then_env, functions);
                }
            } else if let Some(else_branch) = else_branch {
                let mut else_env = Environment::new(Some(Box::new(env.clone())));
                for stmt in else_branch {
                    execute(stmt, &mut else_env, functions);
                }
            }
        }
        ASTNode::FunctionDeclaration { name, parameters, body } => {
            functions.insert(name.clone(), ASTNode::FunctionDeclaration { name, parameters, body });
        }
        ASTNode::FunctionCall { name, arguments } => {
            let function = functions.get(&name).cloned();
            if let Some(ASTNode::FunctionDeclaration { parameters, body, .. }) = function {
                if arguments.len() != parameters.len() {
                    eprintln!("Error: Incorrect number of arguments for function '{}'", name);
                    return;
                }
                let mut local_env = Environment::new(Some(Box::new(env.clone())));
                for (param, arg) in parameters.iter().zip(arguments) {
                    let arg_value = evaluate(arg, env, functions);
                    local_env.define(param.clone(), arg_value);
                }
                for stmt in body {
                    execute(stmt, &mut local_env, functions);
                }
            } else {
                eprintln!("Error: Undefined function '{}'", name);
            }
        }
        _ => {
            // Handle other nodes if necessary
        }
    }
}

fn evaluate(node: ASTNode, env: &mut Environment, functions: &mut HashMap<String, ASTNode>) -> ASTNode {
    match node {
        ASTNode::NumberLiteral(n) => ASTNode::NumberLiteral(n),
        ASTNode::StringLiteral(s) => ASTNode::StringLiteral(s),
        ASTNode::Identifier(name) => {
            if let Some(value) = env.get(&name) {
                value
            } else {
                eprintln!("Error: Undefined variable '{}'", name);
                ASTNode::NumberLiteral(0.0) // Or handle appropriately
            }
        }
        ASTNode::BinaryExpression { left, operator, right } => {
            let left_value = evaluate(*left, env, functions);
            let right_value = evaluate(*right, env, functions);
            match (left_value, right_value) {
                (ASTNode::NumberLiteral(l), ASTNode::NumberLiteral(r)) => match operator {
                    Token::Plus => ASTNode::NumberLiteral(l + r),
                    Token::Minus => ASTNode::NumberLiteral(l - r),
                    Token::Star => ASTNode::NumberLiteral(l * r),
                    Token::Slash => ASTNode::NumberLiteral(l / r),
                    Token::EqualEqual => ASTNode::NumberLiteral((l == r) as i32 as f64),
                    Token::NotEqual => ASTNode::NumberLiteral((l != r) as i32 as f64),
                    Token::LessThan => ASTNode::NumberLiteral((l < r) as i32 as f64),
                    Token::GreaterThan => ASTNode::NumberLiteral((l > r) as i32 as f64),
                    Token::LessEqual => ASTNode::NumberLiteral((l <= r) as i32 as f64),
                    Token::GreaterEqual => ASTNode::NumberLiteral((l >= r) as i32 as f64),
                    _ => {
                        eprintln!("Error: Unsupported operator");
                        ASTNode::NumberLiteral(0.0)
                    }
                },
                (ASTNode::StringLiteral(l), ASTNode::StringLiteral(r)) => match operator {
                    Token::Plus => ASTNode::StringLiteral(l + &r),
                    Token::EqualEqual => ASTNode::NumberLiteral((l == r) as i32 as f64),
                    Token::NotEqual => ASTNode::NumberLiteral((l != r) as i32 as f64),
                    _ => {
                        eprintln!("Error: Unsupported operator for strings");
                        ASTNode::NumberLiteral(0.0)
                    }
                },
                (ASTNode::StringLiteral(l), ASTNode::NumberLiteral(r)) => match operator {
                    Token::Plus => ASTNode::StringLiteral(l + &r.to_string()),
                    _ => {
                        eprintln!("Error: Unsupported operator for string and number");
                        ASTNode::NumberLiteral(0.0)
                    }
                },
                (ASTNode::NumberLiteral(l), ASTNode::StringLiteral(r)) => match operator {
                    Token::Plus => ASTNode::StringLiteral(l.to_string() + &r),
                    _ => {
                        eprintln!("Error: Unsupported operator for number and string");
                        ASTNode::NumberLiteral(0.0)
                    }
                },
                _ => {
                    eprintln!("Error: Invalid operands");
                    ASTNode::NumberLiteral(0.0)
                }
            }
        }
        ASTNode::FunctionCall { name, arguments } => {
            let function = functions.get(&name).cloned();
            if let Some(ASTNode::FunctionDeclaration { parameters, body, .. }) = function {
                if arguments.len() != parameters.len() {
                    eprintln!("Error: Incorrect number of arguments for function '{}'", name);
                    return ASTNode::NumberLiteral(0.0);
                }
                let mut local_env = Environment::new(Some(Box::new(env.clone())));
                for (param, arg) in parameters.iter().zip(arguments) {
                    let arg_value = evaluate(arg, env, functions);
                    local_env.define(param.clone(), arg_value);
                }
                for stmt in body {
                    execute(stmt, &mut local_env, functions);
                }
                // For simplicity, functions do not return values in this example
                ASTNode::NumberLiteral(0.0)
            } else {
                eprintln!("Error: Undefined function '{}'", name);
                ASTNode::NumberLiteral(0.0)
            }
        }
        ASTNode::UnaryExpression { operator, operand } => {
            let operand_value = evaluate(*operand, env, functions);
            match operand_value {
                ASTNode::NumberLiteral(n) => match operator {
                    Token::Minus => ASTNode::NumberLiteral(-n),
                    Token::Bang => ASTNode::NumberLiteral((n == 0.0) as i32 as f64),
                    _ => {
                        eprintln!("Error: Unsupported unary operator");
                        ASTNode::NumberLiteral(0.0)
                    }
                },
                _ => {
                    eprintln!("Error: Invalid operand for unary operator");
                    ASTNode::NumberLiteral(0.0)
                }
            }
        }
        _ => {
            eprintln!("Error: Unsupported AST node in evaluation");
            ASTNode::NumberLiteral(0.0)
        }
    }
}

fn is_truthy(value: &ASTNode) -> bool {
    match value {
        ASTNode::NumberLiteral(n) => *n != 0.0,
        ASTNode::StringLiteral(s) => !s.is_empty(),
        _ => false,
    }
}
