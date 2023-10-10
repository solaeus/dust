use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Math {
    left: Expression,
    operator: MathOperator,
    right: Expression,
}

impl AbstractTree for Math {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let left_node = node.child(0).unwrap();
        let left = Expression::from_syntax_node(source, left_node)?;

        let operator_node = node.child(1).unwrap().child(0).unwrap();
        let operator = match operator_node.kind() {
            "+" => MathOperator::Add,
            "-" => MathOperator::Subtract,
            "*" => MathOperator::Multiply,
            "/" => MathOperator::Divide,
            "%" => MathOperator::Modulo,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "+, -, *, / or %",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let right_node = node.child(2).unwrap();
        let right = Expression::from_syntax_node(source, right_node)?;

        Ok(Math {
            left,
            operator,
            right,
        })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        match self.operator {
            MathOperator::Add | MathOperator::Subtract | MathOperator::Multiply => {
                let left_value = self.left.run(source, context)?.as_int()?;
                let right_value = self.right.run(source, context)?.as_int()?;
                let outcome = match &self.operator {
                    MathOperator::Add => left_value + right_value,
                    MathOperator::Subtract => left_value - right_value,
                    MathOperator::Multiply => left_value * right_value,
                    _ => panic!("Unreachable"),
                };

                Ok(Value::Integer(outcome))
            }
            MathOperator::Divide | MathOperator::Modulo => {
                let left_value = self.left.run(source, context)?.as_number()?;
                let right_value = self.right.run(source, context)?.as_number()?;
                let outcome = match self.operator {
                    MathOperator::Divide => left_value / right_value,
                    MathOperator::Modulo => left_value % right_value,
                    _ => panic!("Unreachable"),
                };

                Ok(Value::Float(outcome))
            }
        }
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
