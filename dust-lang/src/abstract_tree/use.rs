use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    standard_library::{std_fs_compiled, std_io_compiled, std_json_compiled, std_thread_compiled},
    Type,
};

use super::{AbstractNode, AbstractTree, Evaluation, SourcePosition};

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
    fn define_and_validate(
        &self,
        context: &Context,
        manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
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

        *self.abstract_tree.write()? = Some(abstract_tree);

        if let Some(abstract_tree) = self.abstract_tree.read()?.as_ref() {
            abstract_tree.define_and_validate(context, manage_memory, scope)
        } else {
            Err(ValidationError::Uninitialized)
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        if let Some(abstract_tree) = self.abstract_tree.read()?.as_ref() {
            abstract_tree
                .clone()
                .evaluate(context, manage_memory, scope)
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "use {}", self.path)
    }
}

impl Eq for Use {}

impl PartialEq for Use {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl PartialOrd for Use {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Use {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}
