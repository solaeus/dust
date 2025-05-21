use serde::{Deserialize, Serialize};

use crate::Address;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AbstractList {
    pub item_pointers: Vec<Address>,
}
