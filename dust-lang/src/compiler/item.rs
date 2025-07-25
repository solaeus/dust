use serde::{Deserialize, Serialize};

use crate::{Type, Value};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item<C> {
    Constant { value: Value, r#type: Type },
    Function(C),
    Module(Module<C>),
}
