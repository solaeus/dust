//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use std::fmt::{self, Debug, Formatter};

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
pub fn evaluate(source: &str) -> Vec<Result<Value>> {
    let mut context = VariableMap::new();

    evaluate_with_context(source, &mut context)
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
///     vec![Ok(Value::Primitive(Primitive::Empty)), Ok(Value::from(10))]
/// );
/// ```
pub fn evaluate_with_context(source: &str, context: &mut VariableMap) -> Vec<Result<Value>> {
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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self>;

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
        let mut cursor = self.tree.walk();
        let root_node = cursor.node();
        let item_count = root_node.child_count();
        let mut results = Vec::with_capacity(item_count);

        println!("{}", root_node.to_sexp());

        for item_node in root_node.children(&mut cursor) {
            let item_result = Item::from_syntax_node(item_node, self.source);

            match item_result {
                Ok(item) => {
                    let eval_result = item.run(self.context);

                    results.push(eval_result);
                }
                Err(error) => results.push(Err(error)),
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

impl EvaluatorTree for Item {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!(node.kind(), "item");

        let child = node.child(0).unwrap();

        if child.kind() == "comment" {
            let byte_range = child.byte_range();
            let comment_text = &source[byte_range];

            Ok(Item::Comment(comment_text.to_string()))
        } else if child.kind() == "statement" {
            Ok(Item::Statement(Statement::from_syntax_node(child, source)?))
        } else {
            Err(Error::UnexpectedSyntax {
                expected: "comment or statement",
                actual: child.kind(),
                location: child.start_position(),
            })
        }
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Item::Comment(text) => Ok(Value::String(text.clone())),
            Item::Statement(statement) => statement.run(context),
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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!(node.kind(), "statement");

        let child = node.child(0).unwrap();

        match child.kind() {
            "expression" => Ok(Self::Expression(Expression::from_syntax_node(
                child, source,
            )?)),
            _ => Err(Error::UnexpectedSyntax {
                expected: "expression",
                actual: child.kind(),
                location: child.start_position(),
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
    FunctionCall(FunctionCall),
}

impl EvaluatorTree for Expression {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!(node.kind(), "expression");

        let child = node.child(0).unwrap();

        let expression = match child.kind() {
            "identifier" => Self::Identifier(Identifier::from_syntax_node(child, source)?),
            "value" => Expression::Value(Value::from_syntax_node(child, source)?),
            "control_flow" => {
                Expression::ControlFlow(Box::new(ControlFlow::from_syntax_node(child, source)?))
            }
            "assignment" => {
                Expression::Assignment(Box::new(Assignment::from_syntax_node(child, source)?))
            }
            "math" => Expression::Math(Box::new(Math::from_syntax_node(child, source)?)),
            "function_call" => {
                Expression::FunctionCall(FunctionCall::from_syntax_node(child, source)?)
            }
            _ => return Err(Error::UnexpectedSyntax {
                expected:
                    "identifier, operation, control_flow, assignment, math, function_call or value",
                actual: child.kind(),
                location: child.start_position(),
            }),
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
            Expression::FunctionCall(function_call) => function_call.run(context),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(inner: String) -> Self {
        Identifier(inner)
    }

    pub fn take_inner(self) -> String {
        self.0
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl EvaluatorTree for Identifier {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        assert_eq!(node.kind(), "identifier");

        let identifier = &source[node.byte_range()];

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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        assert_eq!(node.kind(), "control_flow");

        let if_node = node.child_by_field_name("if_expression").unwrap();
        let if_expression = Expression::from_syntax_node(if_node, source)?;

        let then_node = node.child_by_field_name("then_statement").unwrap();
        let then_statement = Statement::from_syntax_node(then_node, source)?;

        let else_node = node.child_by_field_name("else_statement");
        let else_statement = if let Some(node) = else_node {
            Some(Statement::from_syntax_node(node, source)?)
        } else {
            None
        };

        Ok(ControlFlow {
            if_expression,
            then_statement,
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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        assert_eq!(node.kind(), "assignment");

        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(identifier_node, source)?;

        let statement_node = node.child(2).unwrap();
        let statement = Statement::from_syntax_node(statement_node, source)?;

        Ok(Assignment {
            identifier,
            statement,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let key = self.identifier.clone().take_inner();
        let value = self.statement.run(context)?;

        context.set_value(key, value)?;

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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        assert_eq!(node.kind(), "math");

        let left_node = node.child(0).unwrap();
        let left = Expression::from_syntax_node(left_node, source)?;

        let operator_node = left_node.next_sibling().unwrap();
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

        let right_node = operator_node.next_sibling().unwrap();
        let right = Expression::from_syntax_node(right_node, source)?;

        Ok(Math {
            left,
            operator,
            right,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    identifier: Identifier,
    expressions: Vec<Expression>,
}

impl EvaluatorTree for FunctionCall {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        assert_eq!(node.kind(), "function_call");

        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(identifier_node, source)?;

        let mut expressions = Vec::new();

        todo!();

        Ok(FunctionCall {
            identifier,
            expressions,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let mut arguments = Vec::with_capacity(self.expressions.len());

        for expression in &self.expressions {
            let value = expression.run(context)?;

            arguments.push(value);
        }

        context.call_function(self.identifier.inner(), &Value::List(arguments))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Function, Table};

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(evaluate("x = 9"), vec![Ok(Value::Empty)]);
        assert_eq!(evaluate("x = 'foo' + 'bar'"), vec![Ok(Value::Empty)]);
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(evaluate("1"), vec![Ok(Value::Integer(1))]);
        assert_eq!(evaluate("123"), vec![Ok(Value::Integer(123))]);
        assert_eq!(evaluate("-666"), vec![Ok(Value::Integer(-666))]);
    }

    #[test]
    fn evaluate_float() {
        assert_eq!(evaluate("0.1"), vec![Ok(Value::Float(0.1))]);
        assert_eq!(evaluate("12.3"), vec![Ok(Value::Float(12.3))]);
        assert_eq!(evaluate("-6.66"), vec![Ok(Value::Float(-6.66))]);
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(
            evaluate("\"one\""),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("'one'"),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("`one`"),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("`'one'`"),
            vec![Ok(Value::String("'one'".to_string()))]
        );
        assert_eq!(
            evaluate("'`one`'"),
            vec![Ok(Value::String("`one`".to_string()))]
        );
        assert_eq!(
            evaluate("\"'one'\""),
            vec![Ok(Value::String("'one'".to_string()))]
        );
    }

    #[test]
    fn evaluate_list() {
        assert_eq!(
            evaluate("[1, 2, 'foobar']"),
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

        map.set_value("x".to_string(), Value::Integer(1)).unwrap();
        map.set_value("foo".to_string(), Value::String("bar".to_string()))
            .unwrap();

        assert_eq!(evaluate("{ x = 1 foo = 'bar' }"), vec![Ok(Value::Map(map))]);
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
            evaluate(
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
    fn evaluate_if_then() {
        assert_eq!(
            evaluate("if true then 'true'"),
            vec![Ok(Value::String("true".to_string()))]
        );
    }

    #[test]
    fn evaluate_if_then_else() {
        assert_eq!(
            evaluate("if false then 1 else 2"),
            vec![Ok(Value::Integer(2))]
        );
        assert_eq!(
            evaluate("if true then 1.0 else 42.0"),
            vec![Ok(Value::Float(1.0))]
        );
    }

    #[test]
    fn evaluate_function() {
        let function = Function::new(
            vec![Identifier::new("output".to_string())],
            vec![Statement::Expression(Expression::Identifier(
                Identifier::new("output".to_string()),
            ))],
        );

        assert_eq!(
            evaluate("function <output> { output }"),
            vec![Ok(Value::Function(function))]
        );
    }
}
