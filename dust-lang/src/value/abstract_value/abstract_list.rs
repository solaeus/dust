use serde::{Deserialize, Serialize};

use crate::instruction::AddressKind;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AbstractList {
    pub pointer_kind: AddressKind,
    pub indices: Vec<u16>,
}
