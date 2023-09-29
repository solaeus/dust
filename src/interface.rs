//! The top level of Dust's API with functions to interpret Dust code.

use tree_sitter::{Node, Parser, Tree, TreeCursor};

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
/// let dust_code = "one + two + three; one - two - three;";
///
/// assert_eq!(
///     eval_with_context(dust_code, &mut context),
///     vec![Ok(Value::from(6)), Ok(Value::from(-4))]
/// );
/// ```
pub fn eval_with_context(source: &str, context: &mut VariableMap) -> Vec<Result<Value>> {
    let mut parser = Parser::new();

    parser.set_language(language()).unwrap();

    let tree = parser.parse(source, None).unwrap();
    let sexp = tree.root_node().to_sexp();
    let evaluator = Evaluator::new(tree.clone(), source).unwrap();
    let mut cursor = tree.walk();
    let results = evaluator.run(context, &mut cursor, source);

    println!("{sexp}");
    println!("{evaluator:?}");
    println!("{results:?}");
    println!("{context:?}");

    results
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

#[derive(Debug)]
enum Statement {
    Closed(Expression),
    Open(Expression),
}

impl Statement {
    fn new(node: Node, source: &str) -> Result<Self> {
        let child = node.child(0).unwrap();

        match node.kind() {
            "closed_statement" => Ok(Statement::Closed(Expression::new(child, source)?)),
            "open_statement" => Ok(Self::Open(Expression::new(child, source)?)),
            _ => Err(Error::UnexpectedSourceNode {
                expected: "closed_statement or open_statement",
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
            Statement::Closed(expression) => {
                expression.run(context, &mut cursor, source)?;

                Ok(Value::Empty)
            }
            Statement::Open(expression) => expression.run(context, &mut cursor, source),
        }
    }
}

#[derive(Debug)]
enum Expression {
    Identifier(String),
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
            let byte_range = child.byte_range();
            let identifier = &source[byte_range];

            Ok(Self::Identifier(identifier.to_string()))
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
    operator: String,
    right: Box<Expression>,
}

impl Operation {
    fn new(node: Node, source: &str) -> Result<Self> {
        let first_child = node.child(0).unwrap();
        let second_child = node.child(1).unwrap();
        let third_child = node.child(2).unwrap();
        let left = { Box::new(Expression::new(first_child, source)?) };
        let operator = { second_child.child(0).unwrap().kind().to_string() };
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
        let result = match self.operator.as_str() {
            "+" => left + right,
            "=" => {
                if let Expression::Identifier(key) = self.left.as_ref() {
                    context.set_value(key, right)?;
                }

                Ok(Value::Empty)
            }
            _ => return Err(Error::CustomMessage("Operator not supported.".to_string())),
        };

        Ok(result?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(eval("()"), vec![Ok(Value::Empty)]);
        assert_eq!(eval("1;"), vec![Ok(Value::Empty)]);
        assert_eq!(eval("'foobar';"), vec![Ok(Value::Empty)]);
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(eval("1"), vec![Ok(Value::Integer(1))]);
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(eval("'one'"), vec![Ok(Value::String("one".to_string()))]);
    }
}
