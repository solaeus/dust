//! Pattern matching.
//!
//! Note that this module is called "match" but is escaped as "r#match" because
//! "match" is a keyword in Rust.

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Map, Result, Statement, Type, Value};

/// Abstract representation of a match statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Match {
    matcher: Expression,
    options: Vec<(Expression, Statement)>,
    fallback: Option<Box<Statement>>,
}

impl AbstractTree for Match {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "match", node)?;

        let matcher_node = node.child(1).unwrap();
        let matcher = Expression::from_syntax_node(source, matcher_node, context)?;

        let mut options = Vec::new();
        let mut previous_expression = None;
        let mut next_statement_is_fallback = false;
        let mut fallback = None;

        for index in 2..node.child_count() {
            let child = node.child(index).unwrap();

            if child.kind() == "*" {
                next_statement_is_fallback = true;
            }

            if child.kind() == "expression" {
                previous_expression = Some(Expression::from_syntax_node(source, child, context)?);
            }

            if child.kind() == "statement" {
                let statement = Statement::from_syntax_node(source, child, context)?;

                if next_statement_is_fallback {
                    fallback = Some(Box::new(statement));
                    next_statement_is_fallback = false;
                } else if let Some(expression) = &previous_expression {
                    options.push((expression.clone(), statement));
                }
            }
        }

        Ok(Match {
            matcher,
            options,
            fallback,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let matcher_value = self.matcher.run(source, context)?;

        for (expression, statement) in &self.options {
            let option_value = expression.run(source, context)?;

            if matcher_value == option_value {
                return statement.run(source, context);
            }
        }

        if let Some(fallback) = &self.fallback {
            fallback.run(source, context)
        } else {
            Ok(Value::Empty)
        }
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, Value};

    #[test]
    fn evaluate_match() {
        let test = evaluate(
            "
                match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
            ",
        )
        .unwrap();

        assert_eq!(Value::Boolean(true), test);
    }

    #[test]
    fn evaluate_match_assignment() {
        let test = evaluate(
            "
                x = match 1 {
                    3 => false
                    2 => { false }
                    1 => true
                }
                x
            ",
        )
        .unwrap();

        assert_eq!(Value::Boolean(true), test);
    }
}
