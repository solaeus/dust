use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Block, Error, Expression, Format, Identifier, Map, SyntaxNode, Type, Value,
};

/// Abstract representation of a for loop statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct For {
    is_async: bool,
    item_id: Identifier,
    collection: Expression,
    block: Block,
}

impl AbstractTree for For {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "for", node)?;

        let for_node = node.child(0).unwrap();
        let is_async = match for_node.kind() {
            "for" => false,
            "async for" => true,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "for or async for".to_string(),
                    actual: for_node.kind().to_string(),
                    location: for_node.start_position(),
                    relevant_source: source[for_node.byte_range()].to_string(),
                })
            }
        };

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax(expression_node, source, context)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax(item_node, source, context)?;

        Ok(For {
            is_async,
            item_id: identifier,
            collection: expression,
            block: item,
        })
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        self.block.check_type(_source, _context)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        let expression_run = self.collection.run(source, context)?;
        let key = self.item_id.inner();

        if let Value::Range(range) = expression_run {
            if self.is_async {
                range.into_par_iter().try_for_each(|integer| {
                    let iter_context = Map::clone_from(context)?;

                    iter_context.set(key.clone(), Value::Integer(integer))?;

                    self.block.run(source, &iter_context).map(|_value| ())
                })?;
            } else {
                let loop_context = Map::clone_from(context)?;

                for i in range {
                    loop_context.set(key.clone(), Value::Integer(i))?;

                    self.block.run(source, &loop_context)?;
                }
            }

            return Ok(Value::none());
        }

        let values = expression_run.as_list()?.items();

        if self.is_async {
            values.par_iter().try_for_each(|value| {
                let iter_context = Map::clone_from(context)?;

                iter_context.set(key.clone(), value.clone())?;

                self.block.run(source, &iter_context).map(|_value| ())
            })?;
        } else {
            let loop_context = Map::clone_from(context)?;

            for value in values.iter() {
                loop_context.set(key.clone(), value.clone())?;

                self.block.run(source, &loop_context)?;
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
