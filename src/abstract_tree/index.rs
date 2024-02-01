use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Format, IndexExpression, List, Map, SourcePosition, SyntaxNode, Type, Value,
};

/// Abstract representation of an index expression.
///
/// An index is a means of accessing values stored in list, maps and strings.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub collection: IndexExpression,
    pub index: IndexExpression,
    pub index_end: Option<IndexExpression>,
    source_position: SourcePosition,
}

impl AbstractTree for Index {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "index", node)?;

        let collection_node = node.child(0).unwrap();
        let collection = IndexExpression::from_syntax(collection_node, source, context)?;

        let index_node = node.child(2).unwrap();
        let index = IndexExpression::from_syntax(index_node, source, context)?;

        let index_end_node = node.child(4);
        let index_end = if let Some(index_end_node) = index_end_node {
            Some(IndexExpression::from_syntax(
                index_end_node,
                source,
                context,
            )?)
        } else {
            None
        };

        Ok(Index {
            collection,
            index,
            index_end,
            source_position: SourcePosition::from(node.range()),
        })
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        match self.collection.expected_type(context)? {
            Type::List(item_type) => Ok(*item_type.clone()),
            Type::Map(_) => Ok(Type::Any),
            Type::None => Ok(Type::None),
            r#type => Ok(r#type),
        }
    }

    fn validate(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        self.collection.validate(_source, _context)?;
        self.index.validate(_source, _context)?;

        if let Some(index_end) = &self.index_end {
            index_end.validate(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
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
            Value::Map(map) => {
                let (key, value) = if let IndexExpression::Identifier(identifier) = &self.index {
                    let key = identifier.inner();
                    let value = map
                        .variables()?
                        .get(key)
                        .map(|(value, _)| value.clone())
                        .unwrap_or_default();

                    (key.clone(), value)
                } else {
                    let index_value = self.index.run(source, context)?;
                    let key = index_value.as_string()?;
                    let value = map
                        .variables()?
                        .get(key.as_str())
                        .map(|(value, _)| value.clone())
                        .unwrap_or_default();

                    (key.clone(), value)
                };

                if value.is_none() {
                    Err(RuntimeError::VariableIdentifierNotFound(key))
                } else {
                    Ok(value)
                }
            }
            Value::String(string) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = string.chars().nth(index).unwrap_or_default();

                Ok(Value::string(item.to_string()))
            }
            _ => Err(RuntimeError::ExpectedCollection { actual: value }),
        }
    }
}

impl Format for Index {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.collection.format(output, indent_level);
        output.push(':');
        self.index.format(output, indent_level);

        if let Some(expression) = &self.index_end {
            output.push_str("..");
            expression.format(output, indent_level);
        }
    }
}
