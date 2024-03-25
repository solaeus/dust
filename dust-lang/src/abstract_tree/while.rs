use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Action, Expression, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    statements: Vec<Statement>,
}

impl While {
    pub fn new(expression: Expression, statements: Vec<Statement>) -> Self {
        Self {
            expression,
            statements,
        }
    }
}

impl AbstractNode for While {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        self.expression.validate(_context)?;

        for statement in &self.statements {
            statement.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let get_boolean = || -> Result<Value, RuntimeError> {
            let expression_position = self.expression.position();
            let action = self.expression.clone().run(_context)?;

            if let Action::Return(value) = action {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ))
            }
        };

        while let ValueInner::Boolean(true) = get_boolean()?.inner().as_ref() {
            for statement in &self.statements {
                let action = statement.clone().run(_context)?;

                match action {
                    Action::Return(_) => {}
                    Action::None => {}
                    Action::Break => return Ok(Action::Break),
                }
            }
        }

        Ok(Action::None)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Assignment, AssignmentOperator, Block, Logic, ValueNode, WithPos},
        identifier::Identifier,
    };

    use super::*;

    #[test]
    fn simple_while_loop() {
        let action = Statement::Block(
            Block::new(vec![
                Statement::Assignment(
                    Assignment::new(
                        Identifier::new("i").with_position((0, 0)),
                        None,
                        AssignmentOperator::Assign,
                        Statement::Expression(Expression::Value(
                            ValueNode::Integer(3).with_position((0, 0)),
                        )),
                    )
                    .with_position((0, 0)),
                ),
                Statement::While(
                    While::new(
                        Expression::Logic(
                            Box::new(Logic::Less(
                                Expression::Identifier(Identifier::new("i").with_position((0, 0))),
                                Expression::Value(ValueNode::Integer(3).with_position((0, 0))),
                            ))
                            .with_position((0, 0)),
                        ),
                        vec![Statement::Assignment(
                            Assignment::new(
                                Identifier::new("i").with_position((0, 0)),
                                None,
                                AssignmentOperator::AddAssign,
                                Statement::Expression(Expression::Value(
                                    ValueNode::Integer(1).with_position((0, 0)),
                                )),
                            )
                            .with_position((0, 0)),
                        )],
                    )
                    .with_position((0, 0)),
                ),
                Statement::Expression(Expression::Identifier(
                    Identifier::new("i").with_position((0, 0)),
                )),
            ])
            .with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap();

        assert_eq!(action, Action::Return(Value::integer(3)))
    }
}
