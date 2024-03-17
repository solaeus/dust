use crate::{
    error::{RuntimeError, ValidationError},
    Context,
};

use super::{AbstractTree, Action, Identifier, Positioned, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    r#type: Option<Positioned<Type>>,
    operator: AssignmentOperator,
    statement: Box<Positioned<Statement>>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
}

impl Assignment {
    pub fn new(
        identifier: Identifier,
        r#type: Option<Positioned<Type>>,
        operator: AssignmentOperator,
        statement: Positioned<Statement>,
    ) -> Self {
        Self {
            identifier,
            r#type,
            operator,
            statement: Box::new(statement),
        }
    }
}

impl AbstractTree for Assignment {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let statement_type = self.statement.node.expected_type(context)?;

        if let Some(Positioned {
            node: expected_type,
            position: expected_position,
        }) = &self.r#type
        {
            expected_type.check(&statement_type).map_err(|conflict| {
                ValidationError::TypeCheck {
                    conflict,
                    actual_position: self.statement.position,
                    expected_position: expected_position.clone(),
                }
            })?;

            context.set_type(self.identifier.clone(), expected_type.clone())?;
        } else {
            context.set_type(self.identifier.clone(), statement_type)?;
        }

        self.identifier.validate(context)?;
        self.statement.node.validate(context)?;

        Ok(())
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let action = self.statement.node.run(context)?;
        let value = match action {
            Action::Return(value) => value,
            r#break => return Ok(r#break),
        };

        match self.operator {
            AssignmentOperator::Assign => {
                context.set_value(self.identifier, value)?;
            }
            AssignmentOperator::AddAssign => {
                if let Some(previous_value) = context.use_value(&self.identifier)? {
                    let new_value = previous_value.add(&value)?;

                    context.set_value(self.identifier, new_value)?;
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound(self.identifier),
                    ));
                }
            }
            AssignmentOperator::SubAssign => {
                if let Some(previous_value) = context.use_value(&self.identifier)? {
                    let new_value = previous_value.subtract(&value)?;

                    context.set_value(self.identifier, new_value)?;
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound(self.identifier),
                    ));
                }
            }
        }

        Ok(Action::None)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode},
        error::TypeConflict,
        Value,
    };

    use super::*;

    #[test]
    fn assign_value() {
        let context = Context::new();

        Assignment::new(
            Identifier::new("foobar"),
            None,
            AssignmentOperator::Assign,
            Positioned {
                node: Statement::Expression(Expression::Value(ValueNode::Integer(42))),
                position: (0, 0),
            },
        )
        .run(&context)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn add_assign_value() {
        let context = Context::new();

        context
            .set_value(Identifier::new("foobar"), Value::integer(1))
            .unwrap();

        Assignment::new(
            Identifier::new("foobar"),
            None,
            AssignmentOperator::AddAssign,
            Positioned {
                node: Statement::Expression(Expression::Value(ValueNode::Integer(41))),
                position: (0, 0),
            },
        )
        .run(&context)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn subtract_assign_value() {
        let context = Context::new();

        context
            .set_value(Identifier::new("foobar"), Value::integer(43))
            .unwrap();

        Assignment::new(
            Identifier::new("foobar"),
            None,
            AssignmentOperator::SubAssign,
            Positioned {
                node: Statement::Expression(Expression::Value(ValueNode::Integer(1))),
                position: (0, 0),
            },
        )
        .run(&context)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn type_check() {
        let validation = Assignment::new(
            Identifier::new("foobar"),
            Some(Positioned {
                node: Type::Boolean,
                position: (0, 0),
            }),
            AssignmentOperator::Assign,
            Positioned {
                node: Statement::Expression(Expression::Value(ValueNode::Integer(42))),
                position: (0, 0),
            },
        )
        .validate(&Context::new());

        assert_eq!(
            validation,
            Err(ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::Integer,
                    expected: Type::Boolean
                },
                actual_position: (0, 0),
                expected_position: (0, 0),
            })
        )
    }
}
