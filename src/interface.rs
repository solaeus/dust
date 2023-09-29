//! The top level of Dust's API with functions in interpret Dust code.

use std::ops::Range;

use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::{language, Error, Result, Value, VariableMap};

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
pub fn eval_with_context(source: &str, context: &mut VariableMap) -> Result<Value> {
    let mut parser = Parser::new();

    parser.set_language(language()).unwrap();

    let tree = parser.parse(source, None).unwrap();
    let sexp = tree.root_node().to_sexp();

    println!("{sexp}");

    let evaluator = Evaluator::new(tree.clone(), source).unwrap();
    let mut cursor = tree.walk();

    let results = evaluator.run(context, &mut cursor, source);

    println!("{evaluator:?}");
    println!("{results:?}");

    Ok(Value::Empty)
}

#[derive(Debug)]
struct Evaluator {
    items: Vec<Item>,
}

impl Evaluator {
    fn new(tree: Tree, source: &str) -> Result<Self> {
        let mut cursor = tree.walk();
        let root_node = cursor.node();
        let mut items = Vec::new();

        for node in root_node.children(&mut cursor) {
            let item = Item::new(node, source)?;
            items.push(item);
        }

        Ok(Evaluator { items })
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Vec<Result<Value>> {
        let mut results = Vec::with_capacity(self.items.len());

        for root in &self.items {
            match root {
                Item::Comment(comment) => results.push(Ok(Value::String(comment.clone()))),
                Item::Statement(statement) => {
                    results.push(statement.run(context, &mut cursor, source))
                }
            }
        }

        results
    }
}

#[derive(Debug)]
enum Item {
    Comment(String),
    Statement(Statement),
}

impl Item {
    fn new(node: Node, source: &str) -> Result<Self> {
        if node.kind() == "comment" {
            let byte_range = node.byte_range();
            let value_string = &source[byte_range];

            Ok(Item::Comment(value_string.to_string()))
        } else if node.kind() == "statement" {
            Ok(Item::Statement(Statement::new(node, source)?))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "comment or statement",
                actual: node.kind(),
            })
        }
    }
}

#[derive(Debug)]
enum Statement {
    Closed(Expression),
}

impl Statement {
    fn new(node: Node, source: &str) -> Result<Self> {
        if node.kind() == "statement" {
            Ok(Statement::Closed(Expression::new(
                node.child(0).unwrap(),
                source,
            )?))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "statement",
                actual: node.kind(),
            })
        }
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Result<Value> {
        match self {
            Statement::Closed(expression) => expression.run(context, &mut cursor, source),
        }
    }
}

#[derive(Debug)]
enum Expression {
    Identifier(&'static str),
    Value(Value),
    Operation(Operation),
}

impl Expression {
    fn new(node: Node, source: &str) -> Result<Self> {
        if node.kind() != "expression" {
            return Err(Error::UnexpectedSourceNode {
                expected: "expression",
                actual: node.kind(),
            });
        }

        let child = node.child(0).unwrap();

        if child.kind() == "identifier" {
            todo!()
        } else if child.kind() == "value" {
            Ok(Expression::Value(Value::new(child, source)?))
        } else if child.kind() == "operation" {
            Ok(Expression::Operation(Operation::new(child, source)?))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "identifier, operation or value",
                actual: child.kind(),
            })
        }
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Result<Value> {
        match self {
            Expression::Identifier(identifier) => {
                let value = context.get_value(&identifier)?;

                if let Some(value) = value {
                    Ok(value)
                } else {
                    Ok(Value::Empty)
                }
            }
            Expression::Value(value) => Ok(value.clone()),
            Expression::Operation(operation) => operation.run(context, &mut cursor, source),
        }
    }
}

#[derive(Debug)]
struct Operation {
    left: Box<Expression>,
    operator: &'static str,
    right: Box<Expression>,
}

impl Operation {
    fn new(node: Node, source: &str) -> Result<Self> {
        let first_child = node.child(0).unwrap();
        let second_child = node.child(1).unwrap();
        let third_child = node.child(2).unwrap();
        let left = { Box::new(Expression::new(first_child, source)?) };
        let operator = { second_child.child(0).unwrap().kind() };
        let right = { Box::new(Expression::new(third_child, source)?) };

        Ok(Operation {
            left,
            operator,
            right,
        })
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Result<Value> {
        let left = self.left.run(context, &mut cursor, source)?;
        let right = self.right.run(context, &mut cursor, source)?;

        match self.operator {
            "+" => {
                let integer_result = left.as_int()? + right.as_int()?;

                Ok(Value::Integer(integer_result))
            }
            "-" => {
                let integer_result = left.as_int()? - right.as_int()?;

                Ok(Value::Integer(integer_result))
            }
            _ => Ok(Value::Empty),
        }
    }
}
