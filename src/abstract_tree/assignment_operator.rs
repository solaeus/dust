use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Error, Format, Map, SyntaxNode, Type, Value,
};

/// Operators that be used in an assignment statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
}

impl AbstractTree for AssignmentOperator {
    fn from_syntax(
        node: SyntaxNode,
        source: &str,
        _context: &crate::Map,
    ) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "assignment_operator", node)?;

        let operator_node = node.child(0).unwrap();
        let operator = match operator_node.kind() {
            "=" => AssignmentOperator::Equal,
            "+=" => AssignmentOperator::PlusEqual,
            "-=" => AssignmentOperator::MinusEqual,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "=, += or -=".to_string(),
                    actual: operator_node.kind().to_string(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        Ok(operator)
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl Format for AssignmentOperator {
    fn format(&self, output: &mut String, _indent_level: u8) {
        match self {
            AssignmentOperator::Equal => output.push('='),
            AssignmentOperator::PlusEqual => output.push_str("+="),
            AssignmentOperator::MinusEqual => output.push_str("-="),
        }
    }
}
