use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Action, Block, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    if_block: WithPosition<Block>,
    else_ifs: Vec<(Expression, WithPosition<Block>)>,
    else_block: Option<WithPosition<Block>>,
}

impl IfElse {
    pub fn new(
        if_expression: Expression,
        if_block: WithPosition<Block>,
        else_ifs: Vec<(Expression, WithPosition<Block>)>,
        else_block: Option<WithPosition<Block>>,
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
        self.if_block.item.expected_type(_context)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        self.if_expression.validate(context)?;
        self.if_block.item.validate(context)?;

        let expected_type = self.if_block.item.expected_type(context)?;
        let if_expression_type = self.if_expression.expected_type(context)?;

        if let Type::Boolean = if_expression_type {
            if let Some(else_block) = &self.else_block {
                else_block.item.validate(context)?;

                let actual = else_block.item.expected_type(context)?;

                expected_type
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: self.if_block.item.last_statement().position(),
                        expected_position: self.if_expression.position(),
                    })?;
            }
        } else {
            return Err(ValidationError::ExpectedBoolean {
                actual: if_expression_type,
                position: self.if_expression.position(),
            });
        }

        for (expression, block) in &self.else_ifs {
            let expression_type = expression.expected_type(context)?;

            if let Type::Boolean = expression_type {
                block.item.validate(context)?;

                let actual = block.item.expected_type(context)?;

                expected_type
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: self.if_block.item.last_statement().position(),
                        expected_position: self.if_expression.position(),
                    })?;
            } else {
                return Err(ValidationError::ExpectedBoolean {
                    actual: if_expression_type,
                    position: self.if_expression.position(),
                });
            }
        }

        Ok(())
    }

    fn run(self, context: &mut Context, _clear_variables: bool) -> Result<Action, RuntimeError> {
        let if_position = self.if_expression.position();
        let action = self.if_expression.run(context, _clear_variables)?;
        let value = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(if_position),
            ));
        };

        if let ValueInner::Boolean(if_boolean) = value.inner().as_ref() {
            if *if_boolean {
                self.if_block.item.run(context, _clear_variables)
            } else {
                for (expression, block) in self.else_ifs {
                    let expression_position = expression.position();
                    let action = expression.run(context, _clear_variables)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    if let ValueInner::Boolean(else_if_boolean) = value.inner().as_ref() {
                        if *else_if_boolean {
                            return block.item.run(context, _clear_variables);
                        }
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedBoolean {
                                actual: value.r#type(context)?,
                                position: expression_position,
                            },
                        ));
                    }
                }

                if let Some(else_statement) = self.else_block {
                    else_statement.item.run(context, _clear_variables)
                } else {
                    Ok(Action::None)
                }
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
                ))])
                .with_position((0, 0)),
                Vec::with_capacity(0),
                None
            )
            .run(&mut Context::new(), true)
            .unwrap(),
            Action::Return(Value::string("foo".to_string()))
        )
    }
}
