use serde::{Deserialize, Serialize};

use crate::instruction::OperandType;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AbstractList {
    pub indices: Vec<u16>,
    pub item_type: OperandType,
}
