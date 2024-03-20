use std::cmp::Ordering;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type, WithPosition};

#[derive(Clone, Debug)]
pub struct Loop {
    statements: Vec<WithPosition<Statement>>,
}

impl Loop {
    pub fn new(statements: Vec<WithPosition<Statement>>) -> Self {
        Self { statements }
    }
}

impl AbstractNode for Loop {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.node.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        loop {
            for statement in &self.statements {
                let action = statement.node.clone().run(_context)?;

                match action {
                    Action::Return(_) => {}
                    Action::None => {}
                    Action::Break => return Ok(Action::Break),
                }
            }
        }
    }
}

impl Eq for Loop {}

impl PartialEq for Loop {
    fn eq(&self, other: &Self) -> bool {
        self.statements.eq(&other.statements)
    }
}

impl PartialOrd for Loop {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Loop {
    fn cmp(&self, other: &Self) -> Ordering {
        self.statements.cmp(&other.statements)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{
            Assignment, AssignmentOperator, Block, Expression, Identifier, IfElse, Logic, ValueNode,
        },
        Value,
    };

    use super::*;

    #[test]
    fn basic_loop() {
        let action = Loop::new(vec![Statement::Break.with_position((0, 0))])
            .run(&Context::new())
            .unwrap();

        assert_eq!(action, Action::Break)
    }

    #[test]
    fn complex_loop() {
        let action = Block::new(vec![
            Statement::Assignment(Assignment::new(
                Identifier::new("i").with_position((0, 0)),
                None,
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::Integer(1)))
                    .with_position((0, 0)),
            ))
            .with_position((0, 0)),
            Statement::Loop(Loop::new(vec![Statement::IfElse(IfElse::new(
                Expression::Logic(Box::new(Logic::Greater(
                    Expression::Identifier(Identifier::new("i")).with_position((10, 11)),
                    Expression::Value(ValueNode::Integer(2)).with_position((14, 15)),
                )))
                .with_position((10, 15)),
                Block::new(vec![Statement::Break.with_position((18, 24))]),
                Some(Block::new(vec![Statement::Assignment(Assignment::new(
                    Identifier::new("i").with_position((0, 0)),
                    None,
                    AssignmentOperator::AddAssign,
                    Statement::Expression(Expression::Value(ValueNode::Integer(1)))
                        .with_position((38, 39)),
                ))
                .with_position((33, 39))])),
            ))
            .with_position((0, 0))]))
            .with_position((0, 0)),
            Statement::Expression(Expression::Identifier(Identifier::new("i")))
                .with_position((0, 0)),
        ])
        .run(&Context::new())
        .unwrap();

        assert_eq!(action, Action::Return(Value::integer(3)))
    }
}
