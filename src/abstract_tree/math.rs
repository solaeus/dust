use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Format, Map, MathOperator, Result, Type, Value};

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

        let operator_node = node.child(1).unwrap();
        let operator = MathOperator::from_syntax_node(source, operator_node, context)?;

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

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.left.expected_type(context)
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
