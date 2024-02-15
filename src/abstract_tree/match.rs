//! Pattern matching.
//!
//! Note that this module is called "match" but is escaped as "r#match" because
//! "match" is a keyword in Rust.
use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Expression, Format, MatchPattern, Statement, Type, Value,
};

/// Abstract representation of a match statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Match {
    matcher: Expression,
    options: Vec<(MatchPattern, Statement)>,
    fallback: Option<Box<Statement>>,
}

impl AbstractTree for Match {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "match", node)?;

        let matcher_node = node.child(1).unwrap();
        let matcher = Expression::from_syntax(matcher_node, source, context)?;

        let mut options = Vec::new();
        let mut previous_pattern = None;
        let mut next_statement_is_fallback = false;
        let mut fallback = None;

        for index in 2..node.child_count() {
            let child = node.child(index).unwrap();

            if child.kind() == "match_pattern" {
                previous_pattern = Some(MatchPattern::from_syntax(child, source, context)?);
            }

            if child.kind() == "statement" {
                let statement = Statement::from_syntax(child, source, context)?;

                if next_statement_is_fallback {
                    fallback = Some(Box::new(statement));
                    next_statement_is_fallback = false;
                } else if let Some(expression) = &previous_pattern {
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

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        let (_, first_statement) = self.options.first().unwrap();

        first_statement.expected_type(context)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        self.matcher.validate(_source, _context)?;

        for (expression, statement) in &self.options {
            expression.validate(_source, _context)?;
            statement.validate(_source, _context)?;
        }

        if let Some(statement) = &self.fallback {
            statement.validate(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let matcher_value = self.matcher.run(source, context)?;

        for (pattern, statement) in &self.options {
            if let (Value::Enum(enum_instance), MatchPattern::EnumPattern(enum_pattern)) =
                (&matcher_value, pattern)
            {
                if enum_instance.name() == enum_pattern.name().inner()
                    && enum_instance.variant_name() == enum_pattern.variant().inner()
                {
                    let statement_context = Context::with_variables_from(context)?;

                    if let Some(identifier) = enum_pattern.inner_identifier() {
                        statement_context
                            .set_value(identifier.inner().clone(), enum_instance.value().clone())?;
                    }

                    return statement.run(source, &statement_context);
                }
            }
            let pattern_value = pattern.run(source, context)?;

            if matcher_value == pattern_value {
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
