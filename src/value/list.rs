use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use crate::Value;

#[derive(Debug, Clone)]
pub struct List(Vec<Value>);

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl List {
    pub fn new() -> Self {
        List(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        List(Vec::with_capacity(capacity))
    }

    pub fn with_items(items: Vec<Value>) -> Self {
        List(items)
    }

    pub fn items(&self) -> &Vec<Value> {
        &self.0
    }

    pub fn items_mut(&mut self) -> &mut Vec<Value> {
        &mut self.0
    }
}

impl Eq for List {}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let items = self.items();

        write!(f, "[")?;

        for (index, value) in items.iter().enumerate() {
            write!(f, "{value}")?;

            if index != items.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}
