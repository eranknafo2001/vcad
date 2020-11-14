use std::error::Error;

use crate::{Compiler, OptionalSymbols, PatternType};
use regex::escape;

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
enum Tokens {
    OpenBrackets,
    CloseBrackets,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Log,
    Ln,
    Cos,
    Sin,
    Comma,
    Float,
    Int,
    Var,
    Param,
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
enum Symbol {
    Program,
    Value,
}

#[derive(Debug, PartialEq)]
enum Operators {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, PartialEq)]
enum MathematicalFunctions {
    Ln,
    Cos,
    Sin,
}

#[derive(Debug, PartialEq)]
enum Node {
    Operator(Operators, Box<Node>, Box<Node>),
    Functions(MathematicalFunctions, Box<Node>),
    Var,
    Param(String),
    Scalar(f32),
}

#[derive(Debug)]
enum CompilerError {
    OperatorWithOperator,
    NotEnoughItems,
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::OperatorWithOperator => write!(f, "error"),
            CompilerError::NotEnoughItems => write!(f, "error"),
        }
    }
}
impl Error for CompilerError {}

fn operator_pattern(mut items: Vec<NodeResult>) -> Result<NodeResult, Box<dyn Error>> {
    if items.len() != 3 {
        return Err(Box::new(CompilerError::NotEnoughItems));
    }

    let value2 = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let op = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Operator(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let value1 = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    Ok(NodeResult::Node(Node::Operator(
        op,
        Box::new(value1),
        Box::new(value2),
    )))
}

fn log_to_node(val: Node, base: Node) -> Node {
    Node::Operator(
        Operators::Mul,
        Box::new(Node::Functions(MathematicalFunctions::Ln, Box::new(val))),
        Box::new(Node::Functions(MathematicalFunctions::Ln, Box::new(base))),
    )
}

fn log10_pattern(mut items: Vec<NodeResult>) -> Result<NodeResult, Box<dyn Error>> {
    if items.len() != 4 {
        return Err(Box::new(CompilerError::NotEnoughItems));
    }

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::CloseBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let value = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::OpenBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Log => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    Ok(NodeResult::Node(log_to_node(value, Node::Scalar(10f32))))
}
fn log_pattern(mut items: Vec<NodeResult>) -> Result<NodeResult, Box<dyn Error>> {
    if items.len() != 4 {
        return Err(Box::new(CompilerError::NotEnoughItems));
    }

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::CloseBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let base = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Comma => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let value = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::OpenBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Log => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    Ok(NodeResult::Node(log_to_node(value, base)))
}
fn func_pattern(mut items: Vec<NodeResult>) -> Result<NodeResult, Box<dyn Error>> {
    if items.len() != 4 {
        return Err(Box::new(CompilerError::NotEnoughItems));
    }

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::CloseBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let value = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Node(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::OpenBrackets => {}
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    let function = match items.pop().ok_or_else(|| CompilerError::NotEnoughItems)? {
        NodeResult::Function(e) => e,
        _ => return Err(Box::new(CompilerError::OperatorWithOperator)),
    };

    Ok(NodeResult::Node(Node::Functions(function, Box::new(value))))
}

#[derive(Debug)]
enum NodeResult {
    Function(MathematicalFunctions),
    Operator(Operators),
    Node(Node),
    Log,
    OpenBrackets,
    CloseBrackets,
    Comma,
}

fn create_compiler() -> Compiler<Tokens, Symbol, NodeResult> {
    let tokens: Vec<(Tokens, String)> = {
        use Tokens::*;
        vec![
            (CloseBrackets, escape(")")),
            (OpenBrackets, escape("(")),
            (Mul, escape("*")),
            (Div, escape("/")),
            (Add, escape("+")),
            (Sub, escape("-")),
            (Pow, escape("^")),
            (Log, escape("log")),
            (Ln, escape("ln")),
            (Cos, escape("cos")),
            (Sin, escape("sin")),
            (Comma, escape(",")),
            (Float, String::from("\\d+\\.\\d+")),
            (Int, String::from("\\d+")),
            (Var, escape("t")),
            (Param, String::from("[a-z|A-Z][a-z|A-z|0-9|_|-]*")),
        ]
    };

    let patterns: Vec<PatternType<Symbol, Tokens, NodeResult>> = {
        vec![
            (
                Symbol::Program,
                vec![OptionalSymbols::Symbol(Symbol::Value), OptionalSymbols::EOF],
                |mut items| Ok(items.swap_remove(0)),
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Add),
                    OptionalSymbols::Symbol(Symbol::Value),
                ],
                operator_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Sub),
                    OptionalSymbols::Symbol(Symbol::Value),
                ],
                operator_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Mul),
                    OptionalSymbols::Symbol(Symbol::Value),
                ],
                operator_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Div),
                    OptionalSymbols::Symbol(Symbol::Value),
                ],
                operator_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Pow),
                    OptionalSymbols::Symbol(Symbol::Value),
                ],
                operator_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::Log),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                log10_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::Log),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::Comma),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                log_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::Ln),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                func_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::Sin),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                func_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::Cos),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                func_pattern,
            ),
            (
                Symbol::Value,
                vec![
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                    OptionalSymbols::Symbol(Symbol::Value),
                    OptionalSymbols::Token(Tokens::OpenBrackets),
                ],
                |mut items| Ok(items.swap_remove(1)),
            ),
            (
                Symbol::Value,
                vec![OptionalSymbols::Token(Tokens::Int)],
                |mut items| Ok(items.swap_remove(0)),
            ),
            (
                Symbol::Value,
                vec![OptionalSymbols::Token(Tokens::Float)],
                |mut items| Ok(items.swap_remove(0)),
            ),
            (
                Symbol::Value,
                vec![OptionalSymbols::Token(Tokens::Param)],
                |mut items| Ok(items.swap_remove(0)),
            ),
            (
                Symbol::Value,
                vec![OptionalSymbols::Token(Tokens::Var)],
                |mut items| Ok(items.swap_remove(0)),
            ),
        ]
    };

    let complier = Compiler::new(
        tokens,
        vec![" ", "\n", "\t"],
        patterns,
        Symbol::Program,
        |token, word| match token {
            Tokens::OpenBrackets => NodeResult::OpenBrackets,
            Tokens::CloseBrackets => NodeResult::CloseBrackets,
            Tokens::Add => NodeResult::Operator(Operators::Add),
            Tokens::Sub => NodeResult::Operator(Operators::Sub),
            Tokens::Mul => NodeResult::Operator(Operators::Mul),
            Tokens::Div => NodeResult::Operator(Operators::Div),
            Tokens::Pow => NodeResult::Operator(Operators::Pow),
            Tokens::Log => NodeResult::Log,
            Tokens::Ln => NodeResult::Function(MathematicalFunctions::Ln),
            Tokens::Cos => NodeResult::Function(MathematicalFunctions::Cos),
            Tokens::Sin => NodeResult::Function(MathematicalFunctions::Sin),
            Tokens::Comma => NodeResult::Comma,
            Tokens::Int => {
                NodeResult::Node(Node::Scalar(word.trim().parse::<i32>().unwrap() as f32))
            }
            Tokens::Float => NodeResult::Node(Node::Scalar(word.trim().parse::<f32>().unwrap())),
            Tokens::Var => NodeResult::Node(Node::Var),
            Tokens::Param => NodeResult::Node(Node::Param(word.to_string())),
        },
    )
    .unwrap();
    complier
}

#[test]
fn test_compile_brackets() {
    let compiler = create_compiler();

    let t = compiler.compile("(5)").unwrap();
    let t = match t {
        NodeResult::Node(e) => e,
        _ => panic!("error"),
    };
    assert_eq!(t, Node::Scalar(5f32));
}

#[test]
fn test_compile_scalar() {
    let compiler = create_compiler();

    let t = compiler.compile("5").unwrap();
    let t = match t {
        NodeResult::Node(e) => e,
        _ => panic!("error"),
    };
    assert_eq!(t, Node::Scalar(5f32));
}

#[test]
fn test_compile_add() {
    let compiler = create_compiler();

    let t = compiler.compile("5.0+5.0").unwrap();
    let t = match t {
        NodeResult::Node(e) => e,
        _ => panic!("error"),
    };
    assert_eq!(
        t,
        Node::Operator(
            Operators::Add,
            Box::new(Node::Scalar(5f32)),
            Box::new(Node::Scalar(5f32))
        )
    );
}

#[test]
fn test_compile_complex() {
    let compiler = create_compiler();

    let t = compiler
        .compile("(x+t*5+2.2  )/ ( log(s,t)*cos(a)+sin(t)^2 )")
        .unwrap();
    let t = match t {
        NodeResult::Node(e) => e,
        _ => panic!("error"),
    };
    assert_eq!(
        t,
        Node::Operator(
            Operators::Div,
            Box::new(Node::Operator(
                Operators::Add,
                Box::new(Node::Param(String::from("x"))),
                Box::new(Node::Operator(
                    Operators::Add,
                    Box::new(Node::Operator(
                        Operators::Mul,
                        Box::new(Node::Var),
                        Box::new(Node::Scalar(5f32))
                    )),
                    Box::new(Node::Scalar(2.2f32))
                ))
            )),
            Box::new(Node::Operator(
                Operators::Add,
                Box::new(Node::Operator(
                    Operators::Mul,
                    Box::new(Node::Operator(
                        Operators::Mul,
                        Box::new(Node::Functions(
                            MathematicalFunctions::Ln,
                            Box::new(Node::Param(String::from("s")))
                        )),
                        Box::new(Node::Functions(
                            MathematicalFunctions::Ln,
                            Box::new(Node::Var)
                        ))
                    )),
                    Box::new(Node::Functions(
                        MathematicalFunctions::Cos,
                        Box::new(Node::Param(String::from("a")))
                    ))
                )),
                Box::new(Node::Operator(
                    Operators::Pow,
                    Box::new(Node::Functions(
                        MathematicalFunctions::Sin,
                        Box::new(Node::Var)
                    )),
                    Box::new(Node::Scalar(2f32))
                ))
            ))
        )
    );
}
