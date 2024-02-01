use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Format, Map, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogicOperator {
    Equal,
    NotEqual,
    And,
    Or,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

impl AbstractTree for LogicOperator {
    fn from_syntax(node: SyntaxNode, source: &str, _context: &Map) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "logic_operator", node)?;

        let operator_node = node.child(0).unwrap();
        let operator = match operator_node.kind() {
            "==" => LogicOperator::Equal,
            "!=" => LogicOperator::NotEqual,
            "&&" => LogicOperator::And,
            "||" => LogicOperator::Or,
            ">" => LogicOperator::Greater,
            "<" => LogicOperator::Less,
            ">=" => LogicOperator::GreaterOrEqual,
            "<=" => LogicOperator::LessOrEqual,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "==, !=, &&, ||, >, <, >= or <=".to_string(),
                    actual: operator_node.kind().to_string(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        Ok(operator)
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }
}

impl Format for LogicOperator {
    fn format(&self, output: &mut String, _indent_level: u8) {
        match self {
            LogicOperator::Equal => output.push('='),
            LogicOperator::NotEqual => output.push_str("!="),
            LogicOperator::And => output.push_str("&&"),
            LogicOperator::Or => output.push_str("||"),
            LogicOperator::Greater => output.push('>'),
            LogicOperator::Less => output.push('<'),
            LogicOperator::GreaterOrEqual => output.push_str(">="),
            LogicOperator::LessOrEqual => output.push_str("<="),
        }
    }
}
