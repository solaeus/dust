use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, IndexExpression, List, Result, Structure, Type, Value};

/// Abstract representation of an index expression.
///
/// An index is a means of accessing values stored in list, maps and strings.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub collection: IndexExpression,
    pub index: IndexExpression,
    pub index_end: Option<IndexExpression>,
}

impl AbstractTree for Index {
    fn from_syntax_node(source: &str, node: Node, context: &Structure) -> Result<Self> {
        Error::expect_syntax_node(source, "index", node)?;

        let collection_node = node.child(0).unwrap();
        let collection = IndexExpression::from_syntax_node(source, collection_node, context)?;

        let index_node = node.child(2).unwrap();
        let index = IndexExpression::from_syntax_node(source, index_node, context)?;

        let index_end_node = node.child(4);
        let index_end = if let Some(index_end_node) = index_end_node {
            Some(IndexExpression::from_syntax_node(
                source,
                index_end_node,
                context,
            )?)
        } else {
            None
        };

        Ok(Index {
            collection,
            index,
            index_end,
        })
    }

    fn run(&self, source: &str, context: &Structure) -> Result<Value> {
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
            Value::Structure(structure) => {
                let value = if let IndexExpression::Identifier(identifier) = &self.index {
                    let key = identifier.inner();

                    structure
                        .variables()?
                        .get(key)
                        .map(|(value, _)| value.clone())
                        .unwrap_or_default()
                } else {
                    let value = self.index.run(source, context)?;
                    let key = value.as_string()?;

                    structure
                        .variables()?
                        .get(key.as_str())
                        .map(|(value, _)| value.clone())
                        .unwrap_or_default()
                };

                Ok(value)
            }
            Value::String(string) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = string.read()?.chars().nth(index).unwrap_or_default();

                Ok(Value::string(item.to_string()))
            }
            _ => Err(Error::ExpectedCollection { actual: collection }),
        }
    }

    fn expected_type(&self, context: &Structure) -> Result<Type> {
        match self.collection.expected_type(context)? {
            Type::List(item_type) => Ok(*item_type.clone()),
            Type::StructureDefinition(instantiator) => {
                if let IndexExpression::Identifier(identifier) = &self.index {
                    if let Some((statement_option, type_option)) =
                        instantiator.get(identifier.inner())
                    {
                        if let Some(type_definition) = type_option {
                            Ok(type_definition.inner().clone())
                        } else if let Some(statement) = statement_option {
                            statement.expected_type(context)
                        } else {
                            Ok(Type::None)
                        }
                    } else {
                        todo!()
                    }
                } else {
                    todo!()
                }
            }
            Type::Structure(definition_identifier) => {
                let (r#type, key) = if let IndexExpression::Identifier(identifier) = &self.index {
                    let get_type = context
                        .variables()?
                        .get(definition_identifier.inner())
                        .map(|(_, r#type)| r#type.clone());

                    if let Some(r#type) = get_type {
                        (r#type, identifier.inner())
                    } else {
                        return Err(Error::VariableIdentifierNotFound(
                            definition_identifier.inner().clone(),
                        ));
                    }
                } else {
                    return Err(Error::ExpectedIndexIdentifier {
                        actual: self.index.clone(),
                    });
                };

                if let Type::Structure(identifier) = r#type {
                    let variables = context.variables()?;
                    let get_type = variables.get(identifier.inner());

                    if let Some((_, r#type)) = get_type {
                        Ok(r#type.clone())
                    } else {
                        Ok(Type::None)
                    }
                } else if let Type::StructureDefinition(instantiator) = &r#type {
                    let get_type = instantiator.get(key);

                    if let Some((statement_option, type_option)) = get_type {
                        let r#type = if let Some(r#type) = type_option {
                            r#type.inner().clone()
                        } else {
                            if let Some(statement) = statement_option {
                                statement.expected_type(context)?
                            } else {
                                Type::None
                            }
                        };

                        Ok(r#type)
                    } else {
                        Err(Error::VariableIdentifierNotFound(key.clone()))
                    }
                } else {
                    Err(Error::ExpectedStructureDefinition { actual: r#type })
                }
            }
            Type::None => Ok(Type::None),
            r#type => Ok(r#type),
        }
    }
}
