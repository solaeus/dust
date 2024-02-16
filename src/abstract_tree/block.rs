use std::{
    fmt::{self, Formatter},
    sync::RwLock,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    error::{rw_lock_error::RwLockError, RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Statement, SyntaxNode, Type, Value,
};

/// Abstract representation of a block.
///
/// A block is almost identical to the root except that it must have curly
/// braces and can optionally be asynchronous. A block evaluates to the value of
/// its final statement but an async block will short-circuit if a statement
/// results in an error. Note that this will be the first statement to encounter
/// an error at runtime, not necessarilly the first statement as they are
/// written.
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    is_async: bool,
    statements: Vec<Statement>,
}

impl AbstractTree for Block {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "block", node)?;

        let first_child = node.child(0).unwrap();
        let is_async = first_child.kind() == "async";

        let statement_count = if is_async {
            node.child_count() - 3
        } else {
            node.child_count() - 2
        };
        let mut statements = Vec::with_capacity(statement_count);
        let block_context = Context::with_variables_from(context)?;

        for index in 1..node.child_count() - 1 {
            let child_node = node.child(index).unwrap();

            if child_node.kind() == "statement" {
                let statement = Statement::from_syntax(child_node, source, &block_context)?;

                statements.push(statement);
            }
        }

        Ok(Block {
            is_async,
            statements,
        })
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        if self.is_async {
            let statements = &self.statements;
            let final_result = RwLock::new(Ok(Value::none()));

            statements
                .into_par_iter()
                .enumerate()
                .find_map_first(|(index, statement)| {
                    let result = statement.run(_source, _context);
                    let is_last_statement = index == statements.len() - 1;

                    if is_last_statement {
                        let get_write_lock = final_result.write();

                        match get_write_lock {
                            Ok(mut final_result) => {
                                *final_result = result;
                                None
                            }
                            Err(_error) => Some(Err(RuntimeError::RwLock(RwLockError))),
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(final_result.into_inner().map_err(|_| RwLockError)?)
        } else {
            let mut prev_result = None;

            for statement in &self.statements {
                prev_result = Some(statement.run(_source, _context));
            }

            prev_result.unwrap_or(Ok(Value::none()))
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Some(statement) = self.statements.last() {
            statement.expected_type(_context)
        } else {
            Ok(Type::None)
        }
    }
}

impl Format for Block {
    fn format(&self, output: &mut String, indent_level: u8) {
        if self.is_async {
            output.push_str("async {\n");
        } else {
            output.push_str("{\n");
        }

        for (index, statement) in self.statements.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }

            statement.format(output, indent_level + 1);
        }

        output.push('\n');
        Block::indent(output, indent_level);
        output.push('}');
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Block")
            .field("is_async", &self.is_async)
            .field("statements", &self.statements)
            .finish()
    }
}
