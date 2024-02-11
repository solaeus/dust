use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Expression, Format, LogicOperator, SyntaxNode, Type, Value,
};

/// Abstract representation of a logic expression.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Logic {
    left: Expression,
    operator: LogicOperator,
    right: Expression,
}

impl AbstractTree for Logic {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "logic", node)?;

        let first_node = node.child(0).unwrap();
        let (left_node, operator_node, right_node) = {
            if first_node.is_named() {
                (first_node, node.child(1).unwrap(), node.child(2).unwrap())
            } else {
                (
                    node.child(1).unwrap(),
                    node.child(2).unwrap(),
                    node.child(3).unwrap(),
                )
            }
        };
        let left = Expression::from_syntax(left_node, source, context)?;
        let operator = LogicOperator::from_syntax(operator_node, source, context)?;
        let right = Expression::from_syntax(right_node, source, context)?;

        Ok(Logic {
            left,
            operator,
            right,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::Boolean)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        self.left.validate(_source, _context)?;
        self.right.validate(_source, _context)
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
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
            LogicOperator::LessOrEqual => left <= right,
        };

        Ok(Value::Boolean(result))
    }
}

impl Format for Logic {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.left.format(output, indent_level);
        output.push(' ');
        self.operator.format(output, indent_level);
        output.push(' ');
        self.right.format(output, indent_level);
    }
}
