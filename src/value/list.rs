use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use stanza::{
    renderer::{console::Console, Renderer},
    style::Styles,
    table::{Cell, Content, Row, Table},
};

use crate::{error::rw_lock_error::RwLockError, Value};

#[derive(Debug, Clone)]
pub struct List(Arc<RwLock<Vec<Value>>>);

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl List {
    pub fn new() -> Self {
        List(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        List(Arc::new(RwLock::new(Vec::with_capacity(capacity))))
    }

    pub fn with_items(items: Vec<Value>) -> Self {
        List(Arc::new(RwLock::new(items)))
    }

    pub fn items(&self) -> Result<RwLockReadGuard<Vec<Value>>, RwLockError> {
        Ok(self.0.read()?)
    }

    pub fn items_mut(&self) -> Result<RwLockWriteGuard<Vec<Value>>, RwLockError> {
        Ok(self.0.write()?)
    }

    pub fn as_text_table(&self) -> Table {
        let cells: Vec<Cell> = self
            .items()
            .unwrap()
            .iter()
            .map(|value| {
                if let Value::List(list) = value {
                    Cell::new(Styles::default(), Content::Nested(list.as_text_table()))
                } else if let Value::Map(map) = value {
                    Cell::new(Styles::default(), Content::Nested(map.as_text_table()))
                } else {
                    Cell::new(Styles::default(), Content::Label(value.to_string()))
                }
            })
            .collect();

        let row = if cells.is_empty() {
            Row::new(
                Styles::default(),
                vec![Cell::new(
                    Styles::default(),
                    Content::Label("empty list".to_string()),
                )],
            )
        } else {
            Row::new(Styles::default(), cells)
        };

        Table::default().with_row(row)
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let renderer = Console::default();

        f.write_str(&renderer.render(&self.as_text_table()))
    }
}

impl Eq for List {}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(left) = self.items() {
            if let Ok(right) = other.items() {
                if left.len() != right.len() {
                    return false;
                } else {
                    for (i, j) in left.iter().zip(right.iter()) {
                        if i != j {
                            return false;
                        }
                    }
                }
            }
        }

        false
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Ok(left) = self.items() {
            if let Ok(right) = other.items() {
                return left.cmp(&right);
            }
        }

        Ordering::Equal
    }
}
