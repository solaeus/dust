use std::fmt::{self, Display, Formatter};

use crate::{
    Type,
    vm::{Pointer, ThreadData},
};

use super::DustString;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: Type,
    pub item_pointers: Vec<Pointer>,
}

impl AbstractList {
    pub fn display(&self, data: &ThreadData) -> DustString {
        todo!()
    }
}

impl Display for AbstractList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for pointer in &self.item_pointers {
            write!(f, "{}", pointer)?;
        }

        write!(f, "]")
    }
}
