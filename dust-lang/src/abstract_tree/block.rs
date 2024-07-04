use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, SourcePosition, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn first_statement(&self) -> &Statement {
        self.statements.first().unwrap()
    }

    pub fn last_statement(&self) -> &Statement {
        self.statements.last().unwrap()
    }
}

impl AbstractNode for Block {
    fn define_and_validate(
        &self,
        _context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_and_validate(_context, _manage_memory, scope)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let mut previous = None;

        for statement in self.statements {
            previous = statement.evaluate(_context, _manage_memory, scope)?;
        }

        Ok(previous)
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        self.last_statement().expected_type(_context)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{ ")?;

        for statement in &self.statements {
            write!(f, "{statement} ")?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode, WithPos},
        Value,
    };

    use super::*;

    #[test]
    fn run_returns_value_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(
                ValueNode::Integer(1).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(2).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block
                .evaluate(&Context::new(), true, SourcePosition(0, 0))
                .unwrap(),
            Some(Evaluation::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block.expected_type(&Context::new()),
            Ok(Some(Type::Integer))
        )
    }
}
