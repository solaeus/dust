use serde::{Deserialize, Serialize};
use stanza::{
    renderer::{console::Console, Renderer},
    style::{HAlign, Styles},
    table::{Row, Table},
};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::value::Value;

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Map {
    inner: BTreeMap<String, Value>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            inner: BTreeMap::new(),
        }
    }

    pub fn with_values(variables: BTreeMap<String, Value>) -> Self {
        Map { inner: variables }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.inner.iter()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.inner.insert(key, value);
    }

    pub fn as_text_table(&self) -> Table {
        let mut table = Table::with_styles(Styles::default().with(HAlign::Centred));

        for (key, value) in &self.inner {
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
