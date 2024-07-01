use std::{fmt::Display, fs::read_to_string, path::Path};

use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{DustError, RuntimeError, ValidationError},
    lexer::lex,
    parser,
    standard_library::{std_fs_compiled, std_io_compiled, std_json_compiled, std_thread_compiled},
    Type,
};

use super::{AbstractNode, Evaluation};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Use {
    path: String,
}

impl Use {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl AbstractNode for Use {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        match self.path.as_str() {
            "std.io" => std_io_compiled().define_types(context),
            _ => {
                if Path::new(&self.path).exists() {
                    Ok(())
                } else {
                    Err(ValidationError::CannotUsePath(self.path.clone()))
                }
            }
        }
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        match self.path.as_str() {
            "std.io" => std_io_compiled().validate(context, manage_memory),
            _ => {
                if Path::new(&self.path).exists() {
                    Ok(())
                } else {
                    Err(ValidationError::CannotUsePath(self.path.clone()))
                }
            }
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let abstact_tree = match self.path.as_str() {
            "std.fs" => std_fs_compiled().clone(),
            "std.io" => std_io_compiled().clone(),
            "std.json" => std_json_compiled().clone(),
            "std.thread" => std_thread_compiled().clone(),
            path => {
                let file_contents = read_to_string(path)?;
                let tokens = lex(&file_contents).map_err(|errors| RuntimeError::Use(errors))?;
                let abstract_tree = parser(false)
                    .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
                    .into_result()
                    .map_err(|errors| {
                        RuntimeError::Use(
                            errors
                                .into_iter()
                                .map(|error| DustError::from(error))
                                .collect::<Vec<DustError>>(),
                        )
                    })?;

                abstract_tree
            }
        };

        abstact_tree
            .run(context, manage_memory)
            .map_err(|errors| RuntimeError::Use(errors))?;

        Ok(None)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        todo!()
    }
}

impl Display for Use {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
