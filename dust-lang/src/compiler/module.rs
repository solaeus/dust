use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::DustString;

use super::Item;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub items: HashMap<DustString, Item>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            items: HashMap::new(),
        }
    }
}
