use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, EnumPattern, Format, Type, Value, ValueNode,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MatchPattern {
    EnumPattern(EnumPattern),
    Value(ValueNode),
    Wildcard,
}

impl AbstractTree for MatchPattern {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("match_pattern", node)?;

        let child = node.child(0).unwrap();
        let pattern = match child.kind() {
            "enum_pattern" => {
                MatchPattern::EnumPattern(EnumPattern::from_syntax(child, source, context)?)
            }
            "value" => MatchPattern::Value(ValueNode::from_syntax(child, source, context)?),
            "*" => MatchPattern::Wildcard,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "enum pattern or value".to_string(),
                    actual: child.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(pattern)
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            MatchPattern::EnumPattern(enum_pattern) => enum_pattern.expected_type(_context),
            MatchPattern::Value(value_node) => value_node.expected_type(_context),
            MatchPattern::Wildcard => todo!(),
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            MatchPattern::EnumPattern(enum_pattern) => enum_pattern.run(_source, _context),
            MatchPattern::Value(value_node) => value_node.run(_source, _context),
            MatchPattern::Wildcard => todo!(),
        }
    }
}

impl Format for MatchPattern {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
