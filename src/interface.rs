//! The top level of Dust's API with functions in interpret Dust code.

use std::ops::Range;

use tree_sitter::{Parser, TreeCursor};

use crate::{language, token, tree, Error, Result, Value, VariableMap};

/// Evaluate the given expression string.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// assert_eq!(eval("1 + 2 + 3"), Ok(Value::from(6)));
/// ```
///
/// *See the [crate doc](index.html) for more examples and explanations of the expression format.*
pub fn eval(source: &str) -> Result<Value> {
    let mut context = VariableMap::new();

    eval_with_context(source, &mut context)
}

/// Evaluate the given expression string with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// let mut context = VariableMap::new();
/// context.set_value("one".into(), 1.into()).unwrap(); // Do proper error handling here
/// context.set_value("two".into(), 2.into()).unwrap(); // Do proper error handling here
/// context.set_value("three".into(), 3.into()).unwrap(); // Do proper error handling here
/// assert_eq!(eval_with_context("one + two + three", &mut context), Ok(Value::from(6)));
/// ```
pub fn eval_with_context(input: &str, context: &mut VariableMap) -> Result<Value> {
    let mut parser = Parser::new();

    parser.set_language(language()).unwrap();

    let tree = parser.parse(input, None).unwrap();
    let sexp = tree.root_node().to_sexp();

    println!("{sexp}");

    let mut cursor = tree.walk();

    cursor.goto_first_child();

    let statement = Statement::from_cursor(cursor);

    println!("{statement:?}");

    Ok(Value::Empty)
}

#[derive(Debug)]
struct EvalTree {
    root: Source,
}

#[derive(Debug)]
enum Source {
    Comment(String),
    Statement(Statement),
}

#[derive(Debug)]
enum Statement {
    Closed(Expression),
}

impl Statement {
    fn from_cursor(mut cursor: TreeCursor) -> Result<Self> {
        let node = cursor.node();

        cursor.goto_first_child();

        if node.kind() == "statement" {
            Ok(Statement::Closed(Expression::from_cursor(cursor)?))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "statement",
                actual: node.kind(),
            })
        }
    }
}

#[derive(Debug)]
enum Expression {
    Identifier(&'static str),
    Value(Range<usize>),
}

impl Expression {
    fn from_cursor(mut cursor: TreeCursor) -> Result<Self> {
        let parent = cursor.node();

        cursor.goto_first_child();

        let child = cursor.node();

        if parent.kind() == "expression" {
            if child.kind() == "identifier" {
                if let Some(name) = cursor.field_name() {
                    Ok(Expression::Identifier(name))
                } else {
                    Err(Error::ExpectedFieldName)
                }
            } else if child.kind() == "value" {
                Ok(Self::Value(child.byte_range()))
            } else {
                Err(Error::UnexpectedSourceNode {
                    expected: "identifier or value",
                    actual: child.kind(),
                })
            }
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "expression",
                actual: parent.kind(),
            })
        }
    }
}
