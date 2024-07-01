use std::{
    fmt::Display,
    fs::read_to_string,
    path::Path,
    sync::{Arc, RwLock},
};

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

use super::{AbstractNode, AbstractTree, Evaluation};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Use {
    path: String,

    #[serde(skip)]
    abstract_tree: Arc<RwLock<Option<AbstractTree>>>,
}

impl Use {
    pub fn new(path: String) -> Self {
        Self {
            path,
            abstract_tree: Arc::new(RwLock::new(None)),
        }
    }
}

impl AbstractNode for Use {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        let abstract_tree = match self.path.as_str() {
            "std.fs" => std_fs_compiled().clone(),
            "std.json" => std_json_compiled().clone(),
            "std.io" => std_io_compiled().clone(),
            "std.thread" => std_thread_compiled().clone(),
            _ => {
                if Path::new(&self.path).exists() {
                    todo!()
                } else {
                    return Err(ValidationError::CannotUsePath(self.path.clone()));
                }
            }
        };

        abstract_tree.define_types(_context)?;

        *self.abstract_tree.write()? = Some(abstract_tree);

        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        if let Some(abstract_tree) = self.abstract_tree.read()?.as_ref() {
            abstract_tree.validate(context, manage_memory)
        } else {
            Err(ValidationError::Uninitialized)
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        if let Some(abstract_tree) = self.abstract_tree.read()?.as_ref() {
            abstract_tree.clone().evaluate(context, manage_memory)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::Uninitialized,
            ))
        }
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        if let Some(abstract_tree) = self.abstract_tree.read()?.as_ref() {
            abstract_tree.expected_type(context)
        } else {
            Err(ValidationError::Uninitialized)
        }
    }
}

impl Display for Use {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Eq for Use {}

impl PartialEq for Use {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for Use {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl Ord for Use {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}
