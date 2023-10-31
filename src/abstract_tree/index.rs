use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, List, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    collection: Expression,
    index: Expression,
    index_end: Option<Expression>,
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

    fn run(&self, source: &str, context: &mut crate::Map) -> crate::Result<crate::Value> {
        let value = self.collection.run(source, context)?;

        match value {
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
            Value::Map(mut map) => {
                let value = self.index.run(source, &mut map)?;

                Ok(value)
            }
            Value::String(string) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = string.chars().nth(index).unwrap_or_default();

                Ok(Value::String(item.to_string()))
            }
            _ => Err(Error::ExpectedCollection { actual: value }),
        }
    }
}