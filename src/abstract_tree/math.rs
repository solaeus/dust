use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Map, Result, Type, Value};

/// Abstract representation of a math operation.
///
/// Dust currently supports the four basic operations and the modulo (or
/// remainder) operator.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Math {
    left: Expression,
    operator: MathOperator,
    right: Expression,
}

impl AbstractTree for Math {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "math", node)?;

        let left_node = node.child(0).unwrap();
        let left = Expression::from_syntax_node(source, left_node, context)?;

        let operator_node = node.child(1).unwrap().child(0).unwrap();
        let operator = match operator_node.kind() {
            "+" => MathOperator::Add,
            "-" => MathOperator::Subtract,
            "*" => MathOperator::Multiply,
            "/" => MathOperator::Divide,
            "%" => MathOperator::Modulo,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "+, -, *, / or %".to_string(),
                    actual: operator_node.kind().to_string(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let right_node = node.child(2).unwrap();
        let right = Expression::from_syntax_node(source, right_node, context)?;

        Ok(Math {
            left,
            operator,
            right,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let left = self.left.run(source, context)?;
        let right = self.right.run(source, context)?;
        let value = match self.operator {
            MathOperator::Add => left + right,
            MathOperator::Subtract => left - right,
            MathOperator::Multiply => left * right,
            MathOperator::Divide => left / right,
            MathOperator::Modulo => left % right,
        }?;

        Ok(value)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<()> {
        Ok(())
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.left.expected_type(context)
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
