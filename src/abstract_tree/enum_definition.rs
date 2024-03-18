use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Identifier, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnumDefinition {
    name: Identifier,
    type_parameters: Option<Vec<Identifier>>,
    variants: Vec<(Identifier, Option<WithPosition<Type>>)>,
}

impl EnumDefinition {
    pub fn new(
        name: Identifier,
        type_parameters: Option<Vec<Identifier>>,
        variants: Vec<(Identifier, Option<WithPosition<Type>>)>,
    ) -> Self {
        Self {
            name,
            type_parameters,
            variants,
        }
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn type_parameters(&self) -> &Option<Vec<Identifier>> {
        &self.type_parameters
    }

    pub fn variants(&self) -> &Vec<(Identifier, Option<WithPosition<Type>>)> {
        &self.variants
    }
}

impl AbstractTree for EnumDefinition {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        context.set_enum_definition(self.name.clone(), self.clone())?;

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        Ok(Action::None)
    }
}
