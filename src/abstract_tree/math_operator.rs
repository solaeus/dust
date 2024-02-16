use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl AbstractTree for MathOperator {
    fn from_syntax(
        node: SyntaxNode,
        _source: &str,
        _context: &Context,
    ) -> Result<Self, SyntaxError> {
        let operator_node = node.child(0).unwrap();
        let operator = match operator_node.kind() {
            "+" => MathOperator::Add,
            "-" => MathOperator::Subtract,
            "*" => MathOperator::Multiply,
            "/" => MathOperator::Divide,
            "%" => MathOperator::Modulo,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "+, -, *, / or %".to_string(),
                    actual: operator_node.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(operator)
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        Ok(Value::none())
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
