use std::sync::OnceLock;

use enum_iterator::{all, Sequence};

use crate::{EnumDefinition, Identifier, Type, TypeDefinition, VariantContent};

static OPTION: OnceLock<TypeDefinition> = OnceLock::new();

pub fn all_built_in_type_definitions() -> impl Iterator<Item = BuiltInTypeDefinition> {
    all()
}

#[derive(Sequence)]
pub enum BuiltInTypeDefinition {
    Option,
}

impl BuiltInTypeDefinition {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInTypeDefinition::Option => "Option",
        }
    }

    pub fn get(&self) -> &TypeDefinition {
        match self {
            BuiltInTypeDefinition::Option => OPTION.get_or_init(|| {
                TypeDefinition::Enum(EnumDefinition::new(
                    Identifier::new("Option"),
                    vec![
                        (Identifier::new("Some"), VariantContent::Type(Type::Any)),
                        (Identifier::new("None"), VariantContent::None),
                    ],
                ))
            }),
        }
    }
}
