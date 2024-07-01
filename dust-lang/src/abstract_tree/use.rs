use std::{fs::read_to_string, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    lexer::{self, lex},
    Type,
};

use super::{AbstractNode, Evaluation};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Use {
    path: String,
}

impl AbstractNode for Use {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        if Path::new(&self.path).exists() {
            Ok(())
        } else {
            todo!()
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let file_contents = read_to_string(self.path)?;

        let tokens = lex(&file_contents).map_err(|errors| RuntimeError::Use(errors))?;
        let abstract_tree = 

        Ok(None)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        todo!()
    }
}
