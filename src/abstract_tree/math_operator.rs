use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Map, Result, SyntaxNode, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl AbstractTree for MathOperator {
    fn from_syntax(node: SyntaxNode, source: &str, _context: &Map) -> Result<Self> {
        let operator_node = node.child(0).unwrap();
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

        Ok(operator)
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
    }
}

impl Format for MathOperator {
    fn format(&self, output: &mut String, _indent_level: u8) {
        let char = match self {
            MathOperator::Add => '+',
            MathOperator::Subtract => '-',
            MathOperator::Multiply => '*',
            MathOperator::Divide => '/',
            MathOperator::Modulo => '%',
        };

        output.push(char);
    }
}
