use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Item, Map, Result, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Select {
    identifiers: Vec<Identifier>,
    expression: Expression,
    item: Option<Item>,
}

impl AbstractTree for Select {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let child_count = node.child_count();
        let mut identifiers = Vec::new();

        for index in 2..child_count - 4 {
            let node = node.child(index).unwrap();

            if node.kind() == "identifier" {
                let identifier = Identifier::from_syntax_node(source, node)?;
                identifiers.push(identifier);
            }

            if node.kind() == ">" {
                break;
            }
        }

        let final_node = node.child(child_count - 1).unwrap();

        let item = if final_node.kind() == "}" {
            let item_node = node.child(child_count - 2).unwrap();

            Some(Item::from_syntax_node(source, item_node)?)
        } else {
            None
        };

        let expression_node = if item.is_some() {
            node.child(child_count - 4).unwrap()
        } else {
            node.child(child_count - 1).unwrap()
        };

        let expression = Expression::from_syntax_node(source, expression_node)?;

        Ok(Select {
            identifiers,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let old_table = value.as_table()?;
        let column_names = if !self.identifiers.is_empty() {
            self.identifiers
                .iter()
                .cloned()
                .map(|identifierier| identifierier.take_inner())
                .collect()
        } else {
            old_table.headers().clone()
        };
        let mut new_table = Table::new(column_names.to_vec());

        for row in old_table.rows() {
            let mut new_row = Vec::new();
            let mut row_context = Map::new();

            for (i, value) in row.iter().enumerate() {
                let column_name = old_table.headers().get(i).unwrap();

                row_context
                    .variables_mut()
                    .insert(column_name.clone(), value.clone());

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

            if let Some(where_clause) = &self.item {
                let should_include = where_clause.run(source, &mut row_context)?.as_boolean()?;

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
