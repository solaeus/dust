//! Key used to identify a value or type.
//!
//! Identifiers are used to uniquely identify values and types in Dust programs. They are
//! cached to avoid duplication. This means that two identifiers with the same text are the same
//! object in memory.
//!
//! # Examples
//! ```
//! # use dust_lang::Identifier;
//! let foo = Identifier::new("foo");
//! let also_foo = Identifier::new("foo");
//! let another_foo = Identifier::new("foo");
//!
//! assert_eq!(foo.strong_count(), 4); // One for each of the above and one for the cache.
//! ```
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    hash::Hash,
    sync::{Arc, OnceLock, RwLock},
};

use serde::{de::Visitor, Deserialize, Serialize};

/// In-use identifiers.
static IDENTIFIER_CACHE: OnceLock<RwLock<HashMap<String, Identifier>>> = OnceLock::new();

/// Returns the identifier cache.
fn identifier_cache<'a>() -> &'a RwLock<HashMap<String, Identifier>> {
    IDENTIFIER_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Key used to identify a value or type.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Identifier(Arc<String>);

impl Identifier {
    /// Creates a new identifier or returns a clone of an existing one from a cache.
    pub fn new<T: ToString>(text: T) -> Self {
        let string = text.to_string();
        let mut cache = identifier_cache().write().unwrap();

        if let Some(old) = cache.get(&string) {
            old.clone()
        } else {
            let new = Identifier(Arc::new(string.clone()));

            cache.insert(string, new.clone());

            new
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl From<&str> for Identifier {
    fn from(text: &str) -> Self {
        Identifier::new(text)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(IdentifierVisitor)
    }
}

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a UTF-8 string")
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Identifier::new(v))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}
