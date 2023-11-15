use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, List, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub collection: Expression,
    pub index: Expression,
    pub index_end: Option<Expression>,
}

impl AbstractTree for Index {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let collection_node = node.child(0).unwrap();
        let collection = Expression::from_syntax_node(source, collection_node)?;

        let index_node = node.child(2).unwrap();
        let index = Expression::from_syntax_node(source, index_node)?;

        let index_end_node = node.child(4);
        let index_end = if let Some(index_end_node) = index_end_node {
            Some(Expression::from_syntax_node(source, index_end_node)?)
        } else {
            None
        };

        Ok(Index {
            collection,
            index,
            index_end,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let collection = self.collection.run(source, context)?;

        match collection {
            Value::List(list) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;

                let item = if let Some(index_end) = &self.index_end {
                    let index_end = index_end.run(source, context)?.as_integer()? as usize;
                    let sublist = list.items()[index..=index_end].to_vec();

                    Value::List(List::with_items(sublist))
                } else {
                    list.items().get(index).cloned().unwrap_or_default()
                };

                Ok(item)
            }
            Value::Map(map) => {
                let value = if let Expression::Identifier(identifier) = &self.index {
                    let key = identifier.inner();

                    map.variables()?.get(key).cloned().unwrap_or(Value::Empty)
                } else {
                    let value = self.index.run(source, context)?;
                    let key = value.as_string()?;

                    map.variables()?.get(key).cloned().unwrap_or(Value::Empty)
                };

                Ok(value)
            }
            Value::String(string) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = string.chars().nth(index).unwrap_or_default();

                Ok(Value::String(item.to_string()))
            }
            _ => Err(Error::ExpectedCollection { actual: collection }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluate;

    #[test]
    fn evaluate_list_index() {
        let test = evaluate("x = [1 [2] 3] x:1:0").unwrap();

        assert_eq!(Value::Integer(2), test);
    }

    #[test]
    fn evaluate_map_index() {
        let test = evaluate("x = {y = {z = 2}} x:y:z").unwrap();

        assert_eq!(Value::Integer(2), test);
    }

    #[test]
    fn evaluate_complex_index() {
        let test = evaluate("x = [1 2 3]; y = || => {0}; x:((y));").unwrap();

        assert_eq!(Value::Integer(1), test);
    }
}
