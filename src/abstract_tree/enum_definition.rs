use super::{AbstractTree, Identifier, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnumDefinition {
    name: Identifier,
    type_parameters: Vec<Identifier>,
    variants: Vec<(Identifier, Vec<WithPosition<Type>>)>,
}

impl EnumDefinition {
    pub fn new(
        name: Identifier,
        type_parameters: Vec<Identifier>,
        variants: Vec<(Identifier, Vec<WithPosition<Type>>)>,
    ) -> Self {
        Self {
            name,
            type_parameters,
            variants,
        }
    }
}

impl AbstractTree for EnumDefinition {
    fn expected_type(
        &self,
        _context: &crate::context::Context,
    ) -> Result<Type, crate::error::ValidationError> {
        todo!()
    }

    fn validate(
        &self,
        _context: &crate::context::Context,
    ) -> Result<(), crate::error::ValidationError> {
        todo!()
    }

    fn run(
        self,
        _context: &crate::context::Context,
    ) -> Result<super::Action, crate::error::RuntimeError> {
        todo!()
    }
}
