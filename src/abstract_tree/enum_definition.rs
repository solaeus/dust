use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Identifier, Type, WithPosition};

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
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        context.set_enum_definition(self.name.clone(), self)?;

        Ok(Action::None)
    }
}
