use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Logic {
    left: Expression,
    operator: LogicOperator,
    right: Expression,
}

impl AbstractTree for Logic {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        let first_node = node.child(0).unwrap();
        let (left_node, operator_node, right_node) = {
            if first_node.is_named() {
                (
                    first_node,
                    node.child(1).unwrap().child(0).unwrap(),
                    node.child(2).unwrap(),
                )
            } else {
                (
                    node.child(1).unwrap(),
                    node.child(2).unwrap().child(0).unwrap(),
                    node.child(3).unwrap(),
                )
            }
        };
        let left = Expression::from_syntax_node(source, left_node, context)?;
        let operator = match operator_node.kind() {
            "==" => LogicOperator::Equal,
            "!=" => LogicOperator::NotEqual,
            "&&" => LogicOperator::And,
            "||" => LogicOperator::Or,
            ">" => LogicOperator::Greater,
            "<" => LogicOperator::Less,
            ">=" => LogicOperator::GreaterOrEqual,
            "<=" => LogicOperator::LessOrEqaul,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "==, !=, &&, ||, >, <, >= or <=",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };
        let right = Expression::from_syntax_node(source, right_node, context)?;

        Ok(Logic {
            left,
            operator,
            right,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let left = self.left.run(source, context)?;
        let right = self.right.run(source, context)?;
        let result = match self.operator {
            LogicOperator::Equal => {
                if let (Ok(left_num), Ok(right_num)) = (left.as_number(), right.as_number()) {
                    left_num == right_num
                } else {
                    left == right
                }
            }
            LogicOperator::NotEqual => {
                if let (Ok(left_num), Ok(right_num)) = (left.as_number(), right.as_number()) {
                    left_num != right_num
                } else {
                    left != right
                }
            }
            LogicOperator::And => left.as_boolean()? && right.as_boolean()?,
            LogicOperator::Or => left.as_boolean()? || right.as_boolean()?,
            LogicOperator::Greater => left > right,
            LogicOperator::Less => left < right,
            LogicOperator::GreaterOrEqual => left >= right,
            LogicOperator::LessOrEqaul => left <= right,
        };

        Ok(Value::Boolean(result))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::Boolean)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogicOperator {
    Equal,
    NotEqual,
    And,
    Or,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqaul,
}
