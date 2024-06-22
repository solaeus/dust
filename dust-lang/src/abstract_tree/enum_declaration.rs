use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Evaluation, Type, TypeConstructor, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumDeclaration {
    name: WithPosition<Identifier>,
    type_parameters: Option<Vec<WithPosition<Identifier>>>,
    variants: Vec<EnumVariant>,
}

impl EnumDeclaration {
    pub fn new(
        name: WithPosition<Identifier>,
        type_parameters: Option<Vec<WithPosition<Identifier>>>,
        variants: Vec<EnumVariant>,
    ) -> Self {
        Self {
            name,
            type_parameters,
            variants,
        }
    }
}

impl AbstractNode for EnumDeclaration {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
        } = self;

        let type_parameters = type_parameters.as_ref().map(|parameters| {
            parameters
                .iter()
                .map(|identifier| Type::Generic {
                    identifier: identifier.node.clone(),
                    concrete_type: None,
                })
                .collect()
        });
        let mut type_variants = Vec::with_capacity(variants.len());

        for EnumVariant { name, content } in variants {
            let types = if let Some(content) = content {
                let mut types = Vec::with_capacity(content.len());

                for constructor in content {
                    let r#type = constructor.construct(&context)?;

                    types.push(r#type);
                }

                Some(types)
            } else {
                None
            };

            type_variants.push((name.node, types));
        }

        let r#type = Type::Enum {
            name: name.node.clone(),
            type_parameters,
            variants: type_variants,
        };

        context.set_type(name.node, r#type)?;

        Ok(())
    }

    fn validate(
        &self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<(), crate::error::ValidationError> {
        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        Ok(None)
    }

    fn expected_type(
        &self,
        context: &Context,
    ) -> Result<Option<Type>, crate::error::ValidationError> {
        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: WithPosition<Identifier>,
    pub content: Option<Vec<TypeConstructor>>,
}
