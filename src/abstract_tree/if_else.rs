use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Block, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: WithPosition<Expression>,
    if_block: Block,
    else_block: Option<Block>,
}

impl IfElse {
    pub fn new(
        if_expression: WithPosition<Expression>,
        if_block: Block,
        else_block: Option<Block>,
    ) -> Self {
        Self {
            if_expression,
            if_block,
            else_block,
        }
    }
}

impl AbstractTree for IfElse {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        self.if_block.expected_type(_context)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let Type::Boolean = self.if_expression.node.expected_type(context)? {
            if let Some(else_block) = &self.else_block {
                let expected = self.if_block.expected_type(context)?;
                let actual = else_block.expected_type(context)?;

                expected
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: self.if_block.last_statement().position,
                        expected_position: self.if_expression.position,
                    })?;
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedBoolean)
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let if_boolean = self
            .if_expression
            .node
            .run(_context)?
            .as_return_value()?
            .as_boolean()?;

        if if_boolean {
            self.if_block.run(_context)
        } else if let Some(else_statement) = self.else_block {
            else_statement.run(_context)
        } else {
            Ok(Action::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Statement, ValueNode},
        Value,
    };

    use super::*;

    #[test]
    fn simple_if() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("foo".to_string())
                ))
                .with_position((0, 0))]),
                None
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("foo".to_string())))
        )
    }

    #[test]
    fn simple_if_else() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(false)).with_position((0, 0)),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("foo".to_string())
                ))
                .with_position((0, 0))]),
                Some(Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("bar".to_string())
                ))
                .with_position((0, 0))]))
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("bar".to_string())))
        )
    }
}
