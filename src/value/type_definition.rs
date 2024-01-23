use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Structure;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TypeDefintion {
    Structure(Structure),
}

impl Display for TypeDefintion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeDefintion::Structure(structure) => write!(f, "{structure}"),
        }
    }
}
