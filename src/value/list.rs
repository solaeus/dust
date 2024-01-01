use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::Value;

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

    pub fn items(&self) -> RwLockReadGuard<'_, Vec<Value>> {
        self.0.read().unwrap()
    }

    pub fn items_mut(&self) -> RwLockWriteGuard<'_, Vec<Value>> {
        self.0.write().unwrap()
    }
}

impl Eq for List {}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        let left = self.0.read().unwrap().clone().into_iter();
        let right = other.0.read().unwrap().clone().into_iter();

        left.eq(right)
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.0.read().unwrap().clone().into_iter();
        let right = other.0.read().unwrap().clone().into_iter();

        left.cmp(right)
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
