use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Action, Block, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    if_block: Block,
    else_ifs: Option<Vec<(Expression, Block)>>,
    else_block: Option<Block>,
}

impl IfElse {
    pub fn new(
        if_expression: Expression,
        if_block: Block,
        else_ifs: Option<Vec<(Expression, Block)>>,
        else_block: Option<Block>,
    ) -> Self {
        Self {
            if_expression,
            if_block,
            else_ifs,
            else_block,
        }
    }
}

impl AbstractNode for IfElse {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        self.if_block.expected_type(_context)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        self.if_expression.validate(context)?;
        self.if_block.validate(context)?;

        let if_expression_type = self.if_expression.expected_type(context)?;

        if let Type::Boolean = if_expression_type {
            if let Some(else_block) = &self.else_block {
                else_block.validate(context)?;

                let expected = self.if_block.expected_type(context)?;
                let actual = else_block.expected_type(context)?;

                expected
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: self.if_block.last_statement().position(),
                        expected_position: self.if_expression.position(),
                    })?;
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedBoolean {
                actual: if_expression_type,
                position: self.if_expression.position(),
            })
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let if_position = self.if_expression.position();
        let action = self.if_expression.run(context)?;
        let value = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(if_position),
            ));
        };

        if let ValueInner::Boolean(if_boolean) = value.inner().as_ref() {
            if *if_boolean {
                self.if_block.run(context)
            } else if let Some(else_statement) = self.else_block {
                else_statement.run(context)
            } else {
                Ok(Action::None)
            }
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedBoolean {
                    actual: value.r#type(context)?,
                    position: if_position,
                },
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Statement, ValueNode, WithPos},
        Value,
    };

    use super::*;

    #[test]
    fn simple_if() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(true).with_position((0, 0))),
                Block::new(vec![Statement::Expression(Expression::Value(
                    ValueNode::String("foo".to_string()).with_position((0, 0))
                ))]),
                None,
                None
            )
            .run(&Context::new())
            .unwrap(),
            Action::Return(Value::string("foo".to_string()))
        )
    }
}
