//! The top level of Dust's API with functions to interpret Dust code.

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree as TSTree, TreeCursor};

use crate::{language, Error, Result, Value, VariableMap};

/// Evaluate the given source code.
///
/// Returns a vector of results from evaluating the source code. Each comment
/// and statemtent will have its own result.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// assert_eq!(eval("1 + 2 + 3"), vec![Ok(Value::from(6))]);
/// ```
pub fn eval(source: &str) -> Vec<Result<Value>> {
    let mut context = VariableMap::new();

    eval_with_context(source, &mut context)
}

/// Evaluate the given source code with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// let mut context = VariableMap::new();
///
/// context.set_value("one".into(), 1.into()).unwrap(); // Do proper error handling here
/// context.set_value("two".into(), 2.into()).unwrap(); // Do proper error handling here
/// context.set_value("three".into(), 3.into()).unwrap(); // Do proper error handling here
///
/// let dust_code = "four = 4 one + two + three + four";
///
/// assert_eq!(
///     eval_with_context(dust_code, &mut context),
///     vec![Ok(Value::Empty), Ok(Value::from(10))]
/// );
/// ```
pub fn eval_with_context(source: &str, context: &mut VariableMap) -> Vec<Result<Value>> {
    let mut parser = Parser::new();

    parser.set_language(language()).unwrap();

    let tree = parser.parse(source, None).unwrap();
    let mut cursor = tree.walk();
    let evaluator = Evaluator::new(tree.clone(), source).unwrap();
    let results = evaluator.run(context, &mut cursor, source);

    println!("{results:?}");

    results
}

pub trait EvaluatorTree: Sized {
    /// Interpret the syntax tree at the given node and return the abstraction.
    ///
    /// This function is used to convert nodes in the Tree Sitter concrete
    /// syntax tree into executable nodes in an abstract tree. This function is
    /// where the tree should be traversed by accessing sibling and child nodes.
    /// Each node in the CST should be traversed only once.
    ///
    /// If necessary, the source code can be accessed directly by getting the
    /// node's byte range.
    fn new(node: Node, source: &str) -> Result<Self>;

    /// Execute dust code by traversing the tree
    fn run(
        &self,
        context: &mut VariableMap,
        cursor: &mut TreeCursor,
        source: &str,
    ) -> Vec<Result<Value>>;
}

/// A collection of statements and comments interpreted from a syntax tree.
///
/// The Evaluator turns a tree sitter concrete syntax tree into a vector of
/// abstract trees called [Item][]s that can be run to execute the source code.
#[derive(Debug)]
pub struct Evaluator {
    items: Vec<Item>,
}

impl Evaluator {
    fn new(tree_sitter_tree: TSTree, source: &str) -> Result<Self> {
        let root_node = tree_sitter_tree.root_node();
        let mut cursor = tree_sitter_tree.walk();
        let mut items = Vec::new();

        for (index, node) in root_node.children(&mut cursor).enumerate() {
            let item = Item::new(node, source)?;

            items.push(item);

            // This iterator will run forever without this check.
            if index == root_node.child_count() - 1 {
                break;
            }
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

        for item in &self.items {
            match item {
                Item::Comment(comment) => results.push(Ok(Value::String(comment.clone()))),
                Item::Statement(statement) => {
                    results.push(statement.run(context, &mut cursor, source))
                }
            }
        }

        results
    }
}

/// An abstractiton of an independent unit of source code.
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with a context by creating or
/// referencing variables.
#[derive(Debug)]
pub enum Item {
    Comment(String),
    Statement(Statement),
}

impl Item {
    fn new(node: Node, source: &str) -> Result<Self> {
        if node.kind() != "item" {
            return Err(Error::UnexpectedSourceNode {
                expected: "item",
                actual: node.kind(),
            });
        }

        let child = node.child(0).unwrap();

        if child.kind() == "comment" {
            let byte_range = node.byte_range();
            let value_string = &source[byte_range];

            Ok(Item::Comment(value_string.to_string()))
        } else if child.kind() == "statement" {
            let child = node.child(0).unwrap();
            Ok(Item::Statement(Statement::new(child, source)?))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "comment or statement",
                actual: node.kind(),
            })
        }
    }
}

/// Representation of a statement in the .
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with a context by creating or
/// referencing variables.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Open(Expression),
}

impl Statement {
    pub fn new(node: Node, source: &str) -> Result<Self> {
        let node = if node.kind() == "statement" {
            node.child(0).unwrap()
        } else {
            node
        };
        let child = node.child(0).unwrap();

        match node.kind() {
            "open_statement" => Ok(Self::Open(Expression::new(child, source)?)),
            _ => Err(Error::UnexpectedSourceNode {
                expected: "open_statement",
                actual: node.kind(),
            }),
        }
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Result<Value> {
        match self {
            Statement::Open(expression) => expression.run(context, &mut cursor, source),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Identifier(String),
    Value(Value),
    Operation(Box<Operation>),
    ControlFlow(Box<ControlFlow>),
}

impl Expression {
    pub fn new(node: Node, source: &str) -> Result<Self> {
        let node = if node.kind() == "expression" {
            node.child(0).unwrap()
        } else {
            node
        };

        if node.kind() == "identifier" {
            let byte_range = node.byte_range();
            let identifier = &source[byte_range];

            Ok(Self::Identifier(identifier.to_string()))
        } else if node.kind() == "value" {
            Ok(Expression::Value(Value::new(node, source)?))
        } else if node.kind() == "operation" {
            Ok(Expression::Operation(Box::new(Operation::new(
                node, source,
            )?)))
        } else if node.kind() == "control_flow" {
            Ok(Expression::ControlFlow(Box::new(ControlFlow::new(
                node, source,
            )?)))
        } else {
            Err(Error::UnexpectedSourceNode {
                expected: "identifier, operation, control_flow or value",
                actual: node.kind(),
            })
        }
    }

    pub fn run(
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
            Expression::ControlFlow(control_flow) => control_flow.run(context, &mut cursor, source),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Operation {
    left: Expression,
    operator: String,
    right: Expression,
}

impl Operation {
    fn new(node: Node, source: &str) -> Result<Self> {
        let first_child = node.child(0).unwrap();
        let second_child = node.child(1).unwrap();
        let third_child = node.child(2).unwrap();
        let left = Expression::new(first_child, source)?;
        let operator = second_child.child(0).unwrap().kind().to_string();
        let right = Expression::new(third_child, source)?;

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

        match self.operator.as_str() {
            "+" => left + right,
            "-" => left - right,
            "=" => {
                if let Expression::Identifier(key) = &self.left {
                    context.set_value(key, right)?;
                }

                Ok(Value::Empty)
            }
            "==" => Ok(Value::Boolean(left == right)),
            _ => Err(Error::CustomMessage("Operator not supported.".to_string())),
        }
    }
}

/// Respresentation of an if-then-else logic gate.
///
/// A ControlFlow instance represents work to be done when the "run" method is
/// called.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct ControlFlow {
    if_expression: Expression,
    then_statement: Statement,
    else_statement: Option<Statement>,
}

impl ControlFlow {
    fn new(node: Node, source: &str) -> Result<Self> {
        // Skip the child nodes for the keywords "if", "then" and "else".
        let second_child = node.child(1).unwrap();
        let fourth_child = node.child(3).unwrap();
        let sixth_child = node.child(5);
        let else_statement = if let Some(child) = sixth_child {
            Some(Statement::new(child, source)?)
        } else {
            None
        };

        Ok(ControlFlow {
            if_expression: Expression::new(second_child, source)?,
            then_statement: Statement::new(fourth_child, source)?,
            else_statement,
        })
    }

    fn run(
        &self,
        context: &mut VariableMap,
        mut cursor: &mut TreeCursor,
        source: &str,
    ) -> Result<Value> {
        let if_boolean = self
            .if_expression
            .run(context, &mut cursor, source)?
            .as_boolean()?;

        if if_boolean {
            self.then_statement.run(context, &mut cursor, source)
        } else if let Some(statement) = &self.else_statement {
            statement.run(context, &mut cursor, source)
        } else {
            Ok(Value::Empty)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Table;

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(eval("()"), vec![Ok(Value::Empty)]);
        assert_eq!(eval("x = 9"), vec![Ok(Value::Empty)]);
        assert_eq!(eval("y = 'foobar'"), vec![Ok(Value::Empty)]);
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(eval("1"), vec![Ok(Value::Integer(1))]);
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(eval("\"one\""), vec![Ok(Value::String("one".to_string()))]);
        assert_eq!(eval("'one'"), vec![Ok(Value::String("one".to_string()))]);
        assert_eq!(eval("`one`"), vec![Ok(Value::String("one".to_string()))]);
        assert_eq!(
            eval("`'one'`"),
            vec![Ok(Value::String("'one'".to_string()))]
        );
        assert_eq!(
            eval("'`one`'"),
            vec![Ok(Value::String("`one`".to_string()))]
        );
        assert_eq!(
            eval("\"'one'\""),
            vec![Ok(Value::String("'one'".to_string()))]
        );
    }

    #[test]
    fn evaluate_list() {
        assert_eq!(
            eval("[1, 2, 'foobar']"),
            vec![Ok(Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::String("foobar".to_string()),
            ]))]
        );
    }

    #[test]
    fn evaluate_map() {
        let mut map = VariableMap::new();

        map.set_value("x", Value::Integer(1)).unwrap();
        map.set_value("foo", Value::String("bar".to_string()))
            .unwrap();

        assert_eq!(eval("map { x = 1 foo = 'bar' }"), vec![Ok(Value::Map(map))]);
    }

    #[test]
    fn evaluate_table() {
        let mut table = Table::new(vec!["messages".to_string(), "numbers".to_string()]);

        table
            .insert(vec![Value::String("hiya".to_string()), Value::Integer(42)])
            .unwrap();
        table
            .insert(vec![Value::String("foo".to_string()), Value::Integer(57)])
            .unwrap();
        table
            .insert(vec![Value::String("bar".to_string()), Value::Float(99.99)])
            .unwrap();

        assert_eq!(
            eval(
                "
                table <messages, numbers> {
                    ['hiya', 42]
                    ['foo', 57]
                    ['bar', 99.99]
                }
                "
            ),
            vec![Ok(Value::Table(table))]
        );
    }

    #[test]
    fn if_then() {
        assert_eq!(
            eval("if true then 'true'"),
            vec![Ok(Value::String("true".to_string()))]
        );
    }

    #[test]
    fn if_then_else() {
        assert_eq!(eval("if false then 1 else 2"), vec![Ok(Value::Integer(2))]);
        assert_eq!(
            eval("if true then 1.0 else 42.0"),
            vec![Ok(Value::Float(1.0))]
        );
    }
}
