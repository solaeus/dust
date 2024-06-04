use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Action, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StructureDefinition {
    name: Identifier,
    fields: Vec<(Identifier, WithPosition<Type>)>,
}

impl StructureDefinition {
    pub fn new(name: Identifier, fields: Vec<(Identifier, WithPosition<Type>)>) -> Self {
        Self { name, fields }
    }
}

impl AbstractNode for StructureDefinition {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let struct_type = Type::Structure {
            name: self.name.clone(),
            fields: self.fields,
        };

        context.set_type(self.name, struct_type)?;

        Ok(Action::None)
    }
}
