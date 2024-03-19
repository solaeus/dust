use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Identifier, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct StructureDefinition {
    name: Identifier,
    fields: Vec<(Identifier, WithPosition<Type>)>,
}

impl StructureDefinition {
    pub fn new(name: Identifier, fields: Vec<(Identifier, WithPosition<Type>)>) -> Self {
        Self { name, fields }
    }
}

impl AbstractTree for StructureDefinition {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let struct_type = Type::Structure {
            name: self.name.clone(),
            fields: self.fields,
        };

        context.set_type(self.name, struct_type)?;

        Ok(Action::None)
    }
}
