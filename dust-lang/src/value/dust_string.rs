use std::{borrow::Borrow, fmt::Display, str::pattern::Pattern};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DustString(SmartString<LazyCompact>);

impl DustString {
    pub fn new() -> Self {
        DustString(SmartString::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn push(&mut self, character: char) {
        self.0.push(character);
    }

    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string);
    }

    pub fn split<P: Pattern>(&self, pattern: P) -> impl Iterator<Item = &str> {
        self.0.split(pattern)
    }
}

impl<T: Into<SmartString<LazyCompact>>> From<T> for DustString {
    fn from(value: T) -> Self {
        DustString(value.into())
    }
}

impl Display for DustString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Borrow<str> for DustString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
