use crate::{
    error::{RuntimeError, ValidationError},
    Context,
};

use super::{AbstractTree, Action, Identifier, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    r#type: Option<Type>,
    operator: AssignmentOperator,
    statement: Box<Statement>,
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
        r#type: Option<Type>,
        operator: AssignmentOperator,
        statement: Statement,
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
        let statement_type = self.statement.expected_type(context)?;

        if let Some(expected) = &self.r#type {
            expected.check(&statement_type)?;

            context.set_type(self.identifier.clone(), expected.clone())?;
        } else {
            context.set_type(self.identifier.clone(), statement_type)?;
        }

        self.identifier.validate(context)?;
        self.statement.validate(context)?;

        Ok(())
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let action = self.statement.run(context)?;
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
        error::TypeCheckError,
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
            Statement::expression(Expression::Value(ValueNode::Integer(42)), (0..0).into()),
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
            Statement::expression(Expression::Value(ValueNode::Integer(41)), (0..0).into()),
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
            Statement::expression(Expression::Value(ValueNode::Integer(1)), (0..0).into()),
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
            Some(Type::Boolean),
            AssignmentOperator::Assign,
            Statement::expression(Expression::Value(ValueNode::Integer(42)), (0..0).into()),
        )
        .validate(&Context::new());

        assert_eq!(
            validation,
            Err(ValidationError::TypeCheck(TypeCheckError {
                actual: Type::Integer,
                expected: Type::Boolean
            }))
        )
    }
}
