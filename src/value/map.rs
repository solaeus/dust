use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
use stanza::{
    renderer::{console::Console, Renderer},
    style::{HAlign, Styles},
    table::{Row, Table},
};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{error::rw_lock_error::RwLockError, value::Value, Structure, Type};

/// A collection dust variables comprised of key-value pairs.
///
/// The inner value is a BTreeMap in order to allow VariableMap instances to be sorted and compared
/// to one another.
#[derive(Clone, Debug)]
pub struct Map {
    variables: Arc<RwLock<BTreeMap<String, (Value, Type)>>>,
    structure: Option<Structure>,
}

impl Map {
    /// Creates a new instace.
    pub fn new() -> Self {
        Map {
            variables: Arc::new(RwLock::new(BTreeMap::new())),
            structure: None,
        }
    }

    pub fn from_structure(structure: Structure) -> Self {
        let mut variables = BTreeMap::new();

        for (key, (value_option, r#type)) in structure.inner() {
            variables.insert(
                key.clone(),
                (
                    value_option.clone().unwrap_or(Value::none()),
                    r#type.clone(),
                ),
            );
        }

        Map {
            variables: Arc::new(RwLock::new(variables)),
            structure: Some(structure),
        }
    }

    pub fn with_variables(variables: BTreeMap<String, (Value, Type)>) -> Self {
        Map {
            variables: Arc::new(RwLock::new(variables)),
            structure: None,
        }
    }

    pub fn clone_from(other: &Self) -> Result<Self, RwLockError> {
        let mut new_map = BTreeMap::new();

        for (key, (value, r#type)) in other.variables()?.iter() {
            new_map.insert(key.clone(), (value.clone(), r#type.clone()));
        }

        Ok(Map {
            variables: Arc::new(RwLock::new(new_map)),
            structure: other.structure.clone(),
        })
    }

    pub fn variables(
        &self,
    ) -> Result<RwLockReadGuard<BTreeMap<String, (Value, Type)>>, RwLockError> {
        self.variables.read().map_err(|_| RwLockError)
    }

    pub fn set(&self, key: String, value: Value) -> Result<Option<(Value, Type)>, RwLockError> {
        log::info!("Setting variable {key} = {value}");

        let value_type = value.r#type();
        let previous = self
            .variables
            .write()?
            .insert(key, (value, value_type.clone()));

        Ok(previous)
    }

    pub fn set_type(
        &self,
        key: String,
        r#type: Type,
    ) -> Result<Option<(Value, Type)>, RwLockError> {
        log::info!("Setting type {key} = {}", r#type);

        let previous = self
            .variables
            .write()
            .map_err(|_| RwLockError)?
            .insert(key, (Value::none(), r#type));

        Ok(previous)
    }

    pub fn as_text_table(&self) -> Table {
        let variables = self.variables.read().unwrap().clone().into_iter();
        let mut table = Table::with_styles(Styles::default().with(HAlign::Centred));

        for (key, (value, r#type)) in variables {
            if let Value::Map(map) = value {
                table.push_row(Row::new(
                    Styles::default(),
                    vec![key.into(), map.as_text_table().into(), "".into()],
                ));
            } else if let Value::List(list) = value {
                table.push_row(Row::new(
                    Styles::default(),
                    vec![
                        key.into(),
                        list.as_text_table().into(),
                        r#type.to_string().into(),
                    ],
                ));
            } else {
                table.push_row([key, value.to_string(), r#type.to_string()]);
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

impl Eq for Map {}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        let left = self.variables.read().unwrap().clone().into_iter();
        let right = other.variables.read().unwrap().clone().into_iter();

        left.eq(right)
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.variables.read().unwrap().clone().into_iter();
        let right = other.variables.read().unwrap().clone().into_iter();

        left.cmp(right)
    }
}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let renderer = Console::default();

        f.write_str(&renderer.render(&self.as_text_table()))
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variables = self.variables.read().unwrap();
        let mut map = serializer.serialize_map(Some(variables.len()))?;

        for (key, (value, _type)) in variables.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

struct MapVisitor {
    marker: PhantomData<fn() -> Map>,
}

impl MapVisitor {
    fn new() -> Self {
        MapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for MapVisitor {
    type Value = Map;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("key-value pairs")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Map, M::Error>
    where
        M: MapAccess<'de>,
    {
        let map = Map::new();

        {
            while let Some((key, value)) = access.next_entry::<String, Value>()? {
                map.set(key, value).unwrap();
            }
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MapVisitor::new())
    }
}
