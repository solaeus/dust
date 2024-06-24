use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Block, Evaluation, Expression, Statement, Type};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct For {
    identifier: Identifier,
    expression: Expression,
    block: Block,

    #[serde(skip)]
    context: Context,
}

impl For {
    pub fn new(identifier: Identifier, expression: Expression, block: Block) -> Self {
        Self {
            identifier,
            expression,
            block,
            context: Context::new(None),
        }
    }
}

impl AbstractNode for For {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        self.context.set_parent(context.clone())?;
        self.expression.define_types(context)?;

        let collection_type =
            self.expression
                .expected_type(context)?
                .ok_or(ValidationError::ExpectedExpression(
                    self.expression.position(),
                ))?;

        let item_type = if let Type::Range = collection_type {
            Type::Integer
        } else {
            todo!("Create an error for this occurence");
        };

        self.context.set_type(self.identifier.clone(), item_type)?;

        for statement in self.block.statements() {
            statement.define_types(&self.context)?;
        }

        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        todo!()
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        todo!()
    }

    fn expected_type(
        &self,
        context: &crate::context::Context,
    ) -> Result<Option<super::Type>, ValidationError> {
        todo!()
    }
}

impl Eq for For {}

impl PartialEq for For {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            && self.expression == other.expression
            && self.block == other.block
    }
}

impl PartialOrd for For {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl Ord for For {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}
