use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Expression, Identifier, Map, Result, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Select {
    identifiers: Vec<Identifier>,
    expression: Expression,
    predicate: Option<Block>,
}

impl AbstractTree for Select {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let child_count = node.child_count();
        let mut identifiers = Vec::new();

        let identifier_list = node.child(1).unwrap();

        for index in 1..identifier_list.child_count() - 1 {
            let node = identifier_list.child(index).unwrap();

            if node.is_named() {
                let identifier = Identifier::from_syntax_node(source, node)?;
                identifiers.push(identifier);
            }
        }

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let final_node = node.child(child_count - 1).unwrap();

        let predicate = if final_node.kind() == "block" {
            Some(Block::from_syntax_node(source, final_node)?)
        } else {
            None
        };

        Ok(Select {
            identifiers,
            expression,
            predicate,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let old_table = value.as_table()?;
        let column_names = if !self.identifiers.is_empty() {
            self.identifiers
                .iter()
                .cloned()
                .map(|identifier| identifier.take_inner())
                .collect()
        } else {
            old_table.headers().clone()
        };
        let mut new_table = Table::new(column_names.to_vec());

        for row in old_table.rows() {
            let mut new_row = Vec::new();
            let row_context = Map::new();
            let mut row_variables = row_context.variables_mut()?;

            for (i, value) in row.iter().enumerate() {
                let column_name = old_table.headers().get(i).unwrap();

                row_variables.insert(column_name.clone(), value.clone());

                let new_table_column_index =
                    new_table
                        .headers()
                        .iter()
                        .enumerate()
                        .find_map(|(index, new_column_name)| {
                            if new_column_name == column_name {
                                Some(index)
                            } else {
                                None
                            }
                        });

                if let Some(index) = new_table_column_index {
                    while new_row.len() < index + 1 {
                        new_row.push(Value::Empty);
                    }
                    new_row[index] = value.clone();
                }
            }

            if let Some(where_clause) = &self.predicate {
                let should_include = where_clause
                    .run(source, &mut row_context.clone())?
                    .as_boolean()?;

                if should_include {
                    new_table.insert(new_row)?;
                }
            } else {
                new_table.insert(new_row)?;
            }
        }

        Ok(Value::Table(new_table))
    }
}
