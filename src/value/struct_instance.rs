use std::fmt::{self, Display, Formatter};

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::{Identifier, Map};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct StructInstance {
    name: Identifier,
    map: Map,
}

impl StructInstance {
    pub fn new(name: Identifier, map: Map) -> Self {
        StructInstance { name, map }
    }
}

impl Display for StructInstance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{{")?;

        for (key, value) in self.map.inner() {
            writeln!(f, "  {key} <{}> = {value}", value.r#type().unwrap())?;
        }

        write!(f, "}}")
    }
}

impl Serialize for StructInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = self.map.inner();
        let mut serde_map = serializer.serialize_map(Some(map.len()))?;

        for (key, value) in map.iter() {
            serde_map.serialize_entry(key, value)?;
        }

        serde_map.end()
    }
}
