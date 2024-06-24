use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, OnceLock, RwLock},
};

use serde::{de::Visitor, Deserialize, Serialize};

static IDENTIFIER_CACHE: OnceLock<RwLock<HashMap<String, Identifier>>> = OnceLock::new();

fn identifier_cache<'a>() -> &'a RwLock<HashMap<String, Identifier>> {
    IDENTIFIER_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(Arc<String>);

impl Identifier {
    pub fn new(string: &str) -> Self {
        let cache = identifier_cache();

        if let Some(identifier) = cache.read().unwrap().get(string) {
            return identifier.clone();
        }

        let identifier = Identifier(Arc::new(string.to_string()));

        cache
            .write()
            .unwrap()
            .insert(string.to_string(), identifier.clone());

        identifier
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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
