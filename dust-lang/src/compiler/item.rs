use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Type, Value};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item<C> {
    Constant { value: Value<C>, r#type: Type },
    Function(Arc<C>),
    Module(Module<C>),
}
