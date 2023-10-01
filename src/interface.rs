//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree as TSTree};

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
/// context.set_value("one".into(), 1.into());
/// context.set_value("two".into(), 2.into());
/// context.set_value("three".into(), 3.into());
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

    Evaluator::new(parser, context, source).run()
}

/// This trait is implemented by the Evaluator's internal types.
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
    fn run(&self, context: &mut VariableMap) -> Result<Value>;
}

/// A collection of statements and comments interpreted from a syntax tree.
///
/// The Evaluator turns a tree sitter concrete syntax tree into a vector of
/// abstract trees called [Item][]s that can be run to execute the source code.
pub struct Evaluator<'context, 'code> {
    _parser: Parser,
    context: &'context mut VariableMap,
    source: &'code str,
    tree: TSTree,
}

impl Debug for Evaluator<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Evaluator context: {}", self.context)
    }
}

impl<'context, 'code> Evaluator<'context, 'code> {
    fn new(mut parser: Parser, context: &'context mut VariableMap, source: &'code str) -> Self {
        let tree = parser.parse(source, None).unwrap();

        Evaluator {
            _parser: parser,
            context,
            source,
            tree,
        }
    }

    fn run(self) -> Vec<Result<Value>> {
        let root_node = self.tree.root_node();
        let mut cursor = self.tree.walk();
        let mut items = Vec::new();
        let mut results = Vec::new();

        for (index, node) in root_node.children(&mut cursor).enumerate() {
            match Item::new(node, self.source) {
                Ok(item) => items.push(item),
                Err(error) => results.push(Err(error)),
            }

            // This iterator will run forever without this check.
            if index == root_node.child_count() - 1 {
                break;
            }
        }

        for item in &items {
            match item {
                Item::Comment(comment) => results.push(Ok(Value::String(comment.to_string()))),
                Item::Statement(statement) => results.push(statement.run(self.context)),
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
            return Err(Error::UnexpectedSyntax {
                expected: "item",
                actual: node.kind(),
                location: node.start_position(),
            });
        }

        let child = node.child(0).unwrap();

        if child.kind() == "comment" {
            let byte_range = node.byte_range();
            let comment_text = &source[byte_range];

            Ok(Item::Comment(comment_text.to_string()))
        } else if child.kind() == "statement" {
            let grandchild = child.child(0).unwrap();

            Ok(Item::Statement(Statement::new(grandchild, source)?))
        } else {
            Err(Error::UnexpectedSyntax {
                expected: "comment or statement",
                actual: node.kind(),
                location: node.start_position(),
            })
        }
    }
}

/// Abstract representation of a statement.
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with a context by creating or
/// referencing variables.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Expression(Expression),
}

impl EvaluatorTree for Statement {
    fn new(node: Node, source: &str) -> Result<Self> {
        let node = if node.kind() == "statement" {
            node.child(0).unwrap()
        } else {
            node
        };

        match node.kind() {
            "expression" => Ok(Self::Expression(Expression::new(node, source)?)),
            _ => Err(Error::UnexpectedSyntax {
                expected: "expression",
                actual: node.kind(),
                location: node.start_position(),
            }),
        }
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Statement::Expression(expression) => expression.run(context),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Identifier(Identifier),
    Value(Value),
    ControlFlow(Box<ControlFlow>),
    Assignment(Box<Assignment>),
    Math(Box<Math>),
}

impl EvaluatorTree for Expression {
    fn new(node: Node, source: &str) -> Result<Self> {
        let node = if node.kind() == "expression" {
            node.child(0).unwrap()
        } else {
            node
        };

        let expression = match node.kind() {
            "identifier" => Self::Identifier(Identifier::new(node, source)?),
            "value" => Expression::Value(Value::new(node, source)?),
            "control_flow" => Expression::ControlFlow(Box::new(ControlFlow::new(node, source)?)),
            "assignment" => Expression::Assignment(Box::new(Assignment::new(node, source)?)),
            "math" => Expression::Math(Box::new(Math::new(node, source)?)),
            _ => {
                return Err(Error::UnexpectedSyntax {
                    expected: "identifier, operation, control_flow, assignment, math or value",
                    actual: node.kind(),
                    location: node.start_position(),
                })
            }
        };

        Ok(expression)
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Expression::Value(value) => Ok(value.clone()),
            Expression::Identifier(identifier) => identifier.run(context),
            Expression::ControlFlow(control_flow) => control_flow.run(context),
            Expression::Assignment(assignment) => assignment.run(context),
            Expression::Math(math) => math.run(context),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn take_inner(self) -> String {
        self.0
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl EvaluatorTree for Identifier {
    fn new(node: Node, source: &str) -> Result<Self> {
        let byte_range = node.byte_range();
        let identifier = &source[byte_range];

        Ok(Identifier(identifier.to_string()))
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let value = context.get_value(&self.0)?.unwrap_or_default();

        Ok(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct ControlFlow {
    if_expression: Expression,
    then_statement: Statement,
    else_statement: Option<Statement>,
}

impl EvaluatorTree for ControlFlow {
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

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let if_boolean = self.if_expression.run(context)?.as_boolean()?;

        if if_boolean {
            self.then_statement.run(context)
        } else if let Some(statement) = &self.else_statement {
            statement.run(context)
        } else {
            Ok(Value::Empty)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    statement: Statement,
}

impl EvaluatorTree for Assignment {
    fn new(node: Node, source: &str) -> Result<Self> {
        let sexp = node.to_sexp();
        println!("{sexp}");

        let identifier_node = node.child(0).unwrap();
        let statement_node = node.child(2).unwrap();
        let identifier = Identifier::new(identifier_node, source)?;
        let statement = Statement::new(statement_node, source)?;

        Ok(Assignment {
            identifier,
            statement,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let value = self.statement.run(context)?;

        context.set_value(self.identifier.inner(), value)?;

        Ok(Value::Empty)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Math {
    left: Expression,
    operator: MathOperator,
    right: Expression,
}

impl EvaluatorTree for Math {
    fn new(node: Node, source: &str) -> Result<Self> {
        let left_node = node.child(0).unwrap();
        let operator_node = node.child(1).unwrap().child(0).unwrap();
        let right_node = node.child(2).unwrap();
        let operator = match operator_node.kind() {
            "+" => MathOperator::Add,
            "-" => MathOperator::Subtract,
            "*" => MathOperator::Multiply,
            "/" => MathOperator::Divide,
            "%" => MathOperator::Modulo,
            _ => {
                return Err(Error::UnexpectedSyntax {
                    expected: "+, -, *, / or %",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                })
            }
        };

        Ok(Math {
            left: Expression::new(left_node, source)?,
            operator,
            right: Expression::new(right_node, source)?,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let left_value = self.left.run(context)?.as_number()?;
        let right_value = self.right.run(context)?.as_number()?;
        let outcome = match self.operator {
            MathOperator::Add => left_value + right_value,
            MathOperator::Subtract => left_value - right_value,
            MathOperator::Multiply => left_value * right_value,
            MathOperator::Divide => left_value / right_value,
            MathOperator::Modulo => left_value % right_value,
        };

        Ok(Value::Float(outcome))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[cfg(test)]
mod tests {
    use crate::Table;

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(eval(""), vec![]);
        assert_eq!(eval("x = 9"), vec![]);
        assert_eq!(eval("'foo' + 'bar'"), vec![]);
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(eval("1"), vec![Ok(Value::Integer(1))]);
        assert_eq!(eval("123"), vec![Ok(Value::Integer(123))]);
        assert_eq!(eval("-666"), vec![Ok(Value::Integer(-666))]);
    }

    #[test]
    fn evaluate_float() {
        assert_eq!(eval("0.1"), vec![Ok(Value::Float(0.1))]);
        assert_eq!(eval("12.3"), vec![Ok(Value::Float(12.3))]);
        assert_eq!(eval("-6.66"), vec![Ok(Value::Float(-6.66))]);
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
