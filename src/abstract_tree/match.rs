//! Pattern matching.
//!
//! Note that this module is called "match" but is escaped as "r#match" because
//! "match" is a keyword in Rust.
use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Error, Expression, Format, Map, Statement, SyntaxNode, Type, Value,
};

/// Abstract representation of a match statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Match {
    matcher: Expression,
    options: Vec<(Expression, Statement)>,
    fallback: Option<Box<Statement>>,
}

impl AbstractTree for Match {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "match", node)?;

        let matcher_node = node.child(1).unwrap();
        let matcher = Expression::from_syntax(matcher_node, source, context)?;

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
                previous_expression = Some(Expression::from_syntax(child, source, context)?);
            }

            if child.kind() == "statement" {
                let statement = Statement::from_syntax(child, source, context)?;

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

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        let (_, first_statement) = self.options.first().unwrap();

        first_statement.expected_type(context)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        self.matcher.check_type(_source, _context)?;

        for (expression, statement) in &self.options {
            expression.check_type(_source, _context)?;
            statement.check_type(_source, _context)?;
        }

        if let Some(statement) = &self.fallback {
            statement.check_type(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
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
            Ok(Value::none())
        }
    }
}

impl Format for Match {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push_str("match ");
        self.matcher.format(output, indent_level);
        output.push_str(" {");

        for (expression, statement) in &self.options {
            expression.format(output, indent_level);
            output.push_str(" => ");
            statement.format(output, indent_level);
        }

        if let Some(statement) = &self.fallback {
            output.push_str("* => ");
            statement.format(output, indent_level);
        }

        output.push('}');
    }
}
