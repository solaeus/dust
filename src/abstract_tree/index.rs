use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, IndexExpression, SourcePosition, SyntaxNode, Type,
    Value,
};

/// Abstract representation of an index expression.
///
/// An index is a means of accessing values stored in list, maps and strings.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub collection: IndexExpression,
    pub index: IndexExpression,
    source_position: SourcePosition,
}

impl AbstractTree for Index {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("index", node)?;

        let collection_node = node.child(0).unwrap();
        let collection = IndexExpression::from_syntax(collection_node, source, context)?;

        let index_node = node.child(2).unwrap();
        let index = IndexExpression::from_syntax(index_node, source, context)?;

        Ok(Index {
            collection,
            index,
            source_position: SourcePosition::from(node.range()),
        })
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        match self.collection.expected_type(context)? {
            Type::ListOf(item_type) => Ok(*item_type.clone()),
            Type::Map(map_types_option) => {
                if let (Some(map_type), IndexExpression::Identifier(identifier)) =
                    (map_types_option, &self.index)
                {
                    if let Some(r#type) = map_type.get(&identifier) {
                        Ok(r#type.clone())
                    } else {
                        Ok(Type::Any)
                    }
                } else {
                    Ok(Type::Any)
                }
            }
            Type::None => Ok(Type::None),
            r#type => Ok(r#type),
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        self.collection.validate(_source, _context)?;

        let collection_type = self.collection.expected_type(_context)?;

        if let (Type::Map(type_map_option), IndexExpression::Identifier(identifier)) =
            (collection_type, &self.index)
        {
            if let Some(type_map) = type_map_option {
                if !type_map.contains_key(identifier) {
                    return Err(ValidationError::VariableIdentifierNotFound(
                        identifier.clone(),
                    ));
                }
            }
        } else {
            self.index.validate(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let value = self.collection.run(source, context)?;

        match value {
            Value::List(list) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = list.items()?.get(index).cloned().unwrap_or_default();

                Ok(item)
            }
            Value::Map(map) => {
                let map = map.inner();

                let value = if let IndexExpression::Identifier(identifier) = &self.index {
                    if let Some(value) = map.get(identifier) {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::VariableIdentifierNotFound(identifier.clone()),
                        ));
                    }
                } else {
                    let index_value = self.index.run(source, context)?;
                    let identifier = Identifier::new(index_value.as_string()?);

                    if let Some(value) = map.get(&identifier) {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::VariableIdentifierNotFound(identifier.clone()),
                        ));
                    }
                };

                Ok(value.clone())
            }
            Value::String(string) => {
                let index = self.index.run(source, context)?.as_integer()? as usize;
                let item = string.chars().nth(index).unwrap_or_default();

                Ok(Value::string(item.to_string()))
            }
            _ => Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedCollection { actual: value },
            )),
        }
    }
}

impl Format for Index {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.collection.format(output, indent_level);
        output.push(':');
        self.index.format(output, indent_level);
    }
}
