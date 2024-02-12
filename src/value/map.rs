use stanza::{
    renderer::{console::Console, Renderer},
    style::{HAlign, Styles},
    table::{Row, Table},
};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{error::rw_lock_error::RwLockError, value::Value};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    inner: Arc<RwLock<BTreeMap<String, Value>>>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_values(variables: BTreeMap<String, Value>) -> Self {
        Map {
            inner: Arc::new(RwLock::new(variables)),
        }
    }

    pub fn inner(&self) -> Result<RwLockReadGuard<BTreeMap<String, Value>>, RwLockError> {
        Ok(self.inner.read()?)
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, RwLockError> {
        Ok(self.inner()?.get(key).cloned())
    }

    pub fn set(&self, key: String, value: Value) -> Result<(), RwLockError> {
        self.inner.write()?.insert(key, value);

        Ok(())
    }

    pub fn as_text_table(&self) -> Table {
        let mut table = Table::with_styles(Styles::default().with(HAlign::Centred));

        for (key, value) in self.inner().unwrap().iter() {
            if let Value::Map(map) = value {
                table.push_row(Row::new(
                    Styles::default(),
                    vec![
                        key.into(),
                        map.as_text_table().into(),
                        "".to_string().into(),
                    ],
                ));
            } else if let Value::List(list) = value {
                table.push_row(Row::new(
                    Styles::default(),
                    vec![key.into(), list.as_text_table().into()],
                ));
            } else {
                table.push_row([key, &value.to_string()]);
            };
        }

        if table.is_empty() {
            table.push_row(vec!["", "empty map", ""])
        }

        table
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let renderer = Console::default();

        f.write_str(&renderer.render(&self.as_text_table()))
    }
}

impl Eq for Map {}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        let left = self.inner().unwrap();
        let right = other.inner().unwrap();

        if left.len() != right.len() {
            return false;
        }

        left.iter()
            .zip(right.iter())
            .all(|((left_key, left_value), (right_key, right_value))| {
                left_key == right_key && left_value == right_value
            })
    }
}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner().unwrap().cmp(&other.inner().unwrap())
    }
}
