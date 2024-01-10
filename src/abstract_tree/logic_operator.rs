use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Map, Result, SyntaxNode, Type, Value};

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
    fn from_syntax(node: SyntaxNode, source: &str, _context: &Map) -> crate::Result<Self> {
        Error::expect_syntax_node(source, "logic_operator", node)?;

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
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "==, !=, &&, ||, >, <, >= or <=".to_string(),
                    actual: operator_node.kind().to_string(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        Ok(operator)
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
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
