use crate::abstract_tree::Identifier;

use super::AbstractTree;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Boolean,
    Custom(Identifier),
    Float,
    Integer,
    List,
    ListOf(Box<Type>),
    ListExact(Vec<Type>),
    Map,
    Range,
    String,
}

impl AbstractTree for Type {
    fn expected_type(
        &self,
        context: &crate::context::Context,
    ) -> Result<Type, crate::error::ValidationError> {
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
    ) -> Result<crate::Value, crate::error::RuntimeError> {
        todo!()
    }
}
