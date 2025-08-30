use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

/// Nesting context for declarations.
///
/// The `index` is a unique identifier for a scope within a module of function. It is used to
/// differentiate between scopes that are not nested together but have the same depth, i.e. sibling
/// scopes.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Scope {
    /// Level of block nesting.
    pub depth: u8,

    /// Index of the scope in its parent scope.
    pub index: u8,
}

impl Scope {
    pub fn contains(&self, other: &Self) -> bool {
        match self.depth.cmp(&other.depth) {
            Ordering::Less => false,
            Ordering::Greater => self.index >= other.index,
            Ordering::Equal => self.index == other.index,
        }
    }

    pub fn begin(&mut self, index: u8) {
        self.depth += 1;
        self.index = index;
    }

    pub fn end(&mut self, index: u8) {
        self.depth -= 1;
        self.index = index;
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.depth, self.index)
    }
}
