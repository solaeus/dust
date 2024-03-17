use super::{AbstractTree, Expression, Positioned, Statement};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Item {
    Expression(Positioned<Expression>),
    Statement(Positioned<Statement>),
}

impl Item {
    pub fn position(&self) -> &(usize, usize) {
        match self {
            Item::Expression((_, position)) => position,
            Item::Statement((_, position)) => position,
        }
    }
}

impl AbstractTree for Item {
    fn expected_type(
        &self,
        context: &crate::context::Context,
    ) -> Result<super::Type, crate::error::ValidationError> {
        todo!()
    }

    fn validate(
        &self,
        context: &crate::context::Context,
    ) -> Result<(), crate::error::ValidationError> {
        todo!()
    }

    fn run(
        self,
        context: &crate::context::Context,
    ) -> Result<super::Action, crate::error::RuntimeError> {
        todo!()
    }
}
