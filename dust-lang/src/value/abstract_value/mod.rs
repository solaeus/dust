mod abstract_function;
mod abstract_list;

pub use abstract_function::AbstractFunction;
pub use abstract_list::AbstractList;
use serde::{Deserialize, Serialize};

use crate::Address;

use super::DustRange;

pub type AbstractRange = DustRange<Address>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AbstractValue {
    Function(AbstractFunction),
    List(AbstractList),
}
