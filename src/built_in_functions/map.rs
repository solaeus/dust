//! Functions related to [Map][crate::Map] values.

use crate::{BuiltInFunction, List, Map, Result, Type, Value};

pub struct Keys;

impl BuiltInFunction for Keys {
    fn name(&self) -> &'static str {
        "keys"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let map = arguments.first().unwrap_or_default().as_map()?;
        let variables = map.variables()?;
        let mut keys = Vec::with_capacity(variables.len());

        for (key, _) in variables.iter() {
            keys.push(Value::String(key.clone()))
        }

        Ok(Value::List(List::with_items(keys)))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Map],
            return_type: Box::new(Type::List(Box::new(Type::String))),
        }
    }
}
