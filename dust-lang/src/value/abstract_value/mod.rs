mod abstract_function;
mod abstract_list;

pub use abstract_function::AbstractFunction;
pub use abstract_list::AbstractList;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AbstractValue {
    Function(AbstractFunction),
    List(AbstractList),
}
