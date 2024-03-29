use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Block, Context, Expression, Format, Identifier, SourcePosition, SyntaxNode, Type,
    Value,
};

/// Abstract representation of a for loop statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct For {
    is_async: bool,
    item_id: Identifier,
    collection: Expression,
    block: Block,
    source_position: SourcePosition,

    #[serde(skip)]
    context: Context,
}

impl AbstractTree for For {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("for", node)?;

        let for_node = node.child(0).unwrap();
        let is_async = match for_node.kind() {
            "for" => false,
            "async for" => true,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "for or async for".to_string(),
                    actual: for_node.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax(expression_node, source, context)?;

        let loop_context = Context::with_variables_from(context)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax(item_node, source, &loop_context)?;

        Ok(For {
            is_async,
            item_id: identifier,
            collection: expression,
            block: item,
            source_position: SourcePosition::from(node.range()),
            context: loop_context,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        self.collection.validate(_source, context)?;

        let collection_type = self.collection.expected_type(context)?;
        let item_type = match collection_type {
            Type::Any => Type::Any,
            Type::Collection => Type::Any,
            Type::List => Type::Any,
            Type::ListOf(_) => todo!(),
            Type::ListExact(_) => todo!(),
            Type::Map(_) => todo!(),
            Type::String => todo!(),
            Type::Range => todo!(),
            _ => {
                return Err(ValidationError::TypeCheck {
                    expected: Type::Collection,
                    actual: collection_type,
                    position: self.source_position,
                });
            }
        };
        let key = self.item_id.clone();

        self.context.inherit_all_from(context)?;
        self.context.set_type(key, item_type)?;
        self.item_id.validate(_source, &self.context)?;
        self.block.validate(_source, &self.context)
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        self.context.inherit_all_from(context)?;

        let expression_run = self.collection.run(source, context)?;
        let key = &self.item_id;

        if let Value::Range(range) = expression_run {
            if self.is_async {
                range.into_par_iter().try_for_each(|integer| {
                    self.context.add_allowance(key)?;
                    self.context
                        .set_value(key.clone(), Value::Integer(integer))?;
                    self.block.run(source, &self.context).map(|_value| ())
                })?;
            } else {
                for i in range {
                    self.context.add_allowance(key)?;
                    self.context.set_value(key.clone(), Value::Integer(i))?;
                    self.block.run(source, &self.context)?;
                }
            }

            return Ok(Value::none());
        }

        if let Value::List(list) = &expression_run {
            if self.is_async {
                list.items()?.par_iter().try_for_each(|value| {
                    self.context.add_allowance(key)?;
                    self.context.set_value(key.clone(), value.clone())?;
                    self.block.run(source, &self.context).map(|_value| ())
                })?;
            } else {
                for value in list.items()?.iter() {
                    self.context.add_allowance(key)?;
                    self.context.set_value(key.clone(), value.clone())?;
                    self.block.run(source, &self.context)?;
                }
            }
        }

        Ok(Value::none())
    }
}

impl Format for For {
    fn format(&self, output: &mut String, indent_level: u8) {
        if self.is_async {
            output.push_str("async for ");
        } else {
            output.push_str("for ");
        }

        self.item_id.format(output, indent_level);
        output.push_str(" in ");
        self.collection.format(output, indent_level);
        output.push(' ');
        self.block.format(output, indent_level);
    }
}
