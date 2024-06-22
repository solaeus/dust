use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Block, Evaluation, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IfElse {
    if_expression: Expression,
    if_block: WithPosition<Block>,
    else_ifs: Option<Vec<(Expression, WithPosition<Block>)>>,
    else_block: Option<WithPosition<Block>>,
}

impl IfElse {
    pub fn new(
        if_expression: Expression,
        if_block: WithPosition<Block>,
        else_ifs: Option<Vec<(Expression, WithPosition<Block>)>>,
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
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        self.if_expression.define_types(_context)?;
        self.if_block.define_type(_context)?;

        if let Some(else_ifs) = self.else_ifs {
            for (expression, block) in else_ifs {
                expression.define_types(_context)?;
                block.node.define_types(_context)?;
            }
        }

        if let Some(else_block) = self.else_block {
            else_block.node.define_types(_context)?;
        }

        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.if_expression.validate(context, manage_memory)?;
        self.if_block.node.validate(context, manage_memory)?;

        let expected_type = self.if_block.node.expected_type(context)?;
        let if_expression_type = self.if_expression.expected_type(context)?;

        if let Type::Boolean = if_expression_type {
            if let Some(else_block) = &self.else_block {
                else_block.node.validate(context, manage_memory)?;

                let actual = else_block.node.expected_type(context)?;

                expected_type
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: else_block.node.last_statement().position(),
                        expected_position: Some(self.if_block.node.first_statement().position()),
                    })?;
            }
        } else {
            return Err(ValidationError::ExpectedBoolean {
                actual: if_expression_type,
                position: self.if_expression.position(),
            });
        }

        if let Some(else_ifs) = &self.else_ifs {
            for (expression, block) in else_ifs {
                let expression_type = expression.expected_type(context)?;

                if let Type::Boolean = expression_type {
                    block.node.validate(context, manage_memory)?;

                    let actual = block.node.expected_type(context)?;

                    expected_type.check(&actual).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: self.if_block.node.last_statement().position(),
                            expected_position: Some(self.if_expression.position()),
                        }
                    })?;
                } else {
                    return Err(ValidationError::ExpectedBoolean {
                        actual: if_expression_type,
                        position: self.if_expression.position(),
                    });
                }
            }
        }

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let if_position = self.if_expression.position();
        let action = self.if_expression.evaluate(context, _manage_memory)?;
        let value = if let Evaluation::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(if_position),
            ));
        };

        if let ValueInner::Boolean(if_boolean) = value.inner().as_ref() {
            if *if_boolean {
                return self.if_block.node.run(context, _manage_memory);
            }

            if let Some(else_ifs) = self.else_ifs {
                for (expression, block) in else_ifs {
                    let expression_position = expression.position();
                    let action = expression.evaluate(context, _manage_memory)?;
                    let value = if let Evaluation::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    if let ValueInner::Boolean(else_if_boolean) = value.inner().as_ref() {
                        if *else_if_boolean {
                            return block.node.run(context, _manage_memory);
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
            }

            if let Some(else_statement) = self.else_block {
                else_statement.node.run(context, _manage_memory)
            } else {
                Ok(None)
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

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.if_block.node.expected_type(_context)
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
                Some(Vec::with_capacity(0)),
                None
            )
            .run(&mut Context::new(None), true)
            .unwrap(),
            Some(Evaluation::Return(Value::string("foo".to_string())))
        )
    }
}
