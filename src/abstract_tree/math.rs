use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Expression, Format, Map, MathOperator, SyntaxNode, Type, Value,
};

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
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "math", node)?;

        let left_node = node.child(0).unwrap();
        let left = Expression::from_syntax(left_node, source, context)?;

        let operator_node = node.child(1).unwrap();
        let operator = MathOperator::from_syntax(operator_node, source, context)?;

        let right_node = node.child(2).unwrap();
        let right = Expression::from_syntax(right_node, source, context)?;

        Ok(Math {
            left,
            operator,
            right,
        })
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        self.left.expected_type(context)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        self.left.check_type(_source, _context)?;
        self.right.check_type(_source, _context)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
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
}

impl Format for Math {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.left.format(output, indent_level);
        output.push(' ');
        self.operator.format(output, indent_level);
        output.push(' ');
        self.right.format(output, indent_level);
    }
}
