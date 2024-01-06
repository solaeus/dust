use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{Type, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Structure(Arc<BTreeMap<String, (Option<Value>, Type)>>);

impl Structure {
    pub fn new(map: BTreeMap<String, (Option<Value>, Type)>) -> Self {
        Structure(Arc::new(map))
    }

    pub fn inner(&self) -> &BTreeMap<String, (Option<Value>, Type)> {
        &self.0
    }
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{{")?;

        for (key, (value_option, r#type)) in self.0.as_ref() {
            if let Some(value) = value_option {
                writeln!(f, "  {key} <{}> = {value}", r#type)?;
            } else {
                writeln!(f, "  {key} <{}>", r#type)?;
            }
        }
        write!(f, "}}")
    }
}
