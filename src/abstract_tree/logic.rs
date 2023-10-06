use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Logic {
    left: Expression,
    operator: LogicOperator,
    right: Expression,
}

impl AbstractTree for Logic {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        let left_node = node.child(0).unwrap();
        let left = Expression::from_syntax_node(left_node, source)?;

        let operator_node = node.child(1).unwrap().child(0).unwrap();
        let operator = match operator_node.kind() {
            "==" => LogicOperator::Equal,
            "&&" => LogicOperator::And,
            "||" => LogicOperator::Or,
            _ => {
                return Err(Error::UnexpectedSyntax {
                    expected: "==, && or ||",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let right_node = node.child(2).unwrap();
        let right = Expression::from_syntax_node(right_node, source)?;

        Ok(Logic {
            left,
            operator,
            right,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let left_value = self.left.run(context)?;
        let right_value = self.right.run(context)?;
        let outcome = match self.operator {
            LogicOperator::Equal => left_value == right_value,
            LogicOperator::And => left_value.as_boolean()? && right_value.as_boolean()?,
            LogicOperator::Or => left_value.as_boolean()? || right_value.as_boolean()?,
        };

        Ok(Value::Boolean(outcome))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogicOperator {
    Equal,
    And,
    Or,
}
