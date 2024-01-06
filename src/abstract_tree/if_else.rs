use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Expression, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    if_block: Block,
    else_if_expressions: Vec<Expression>,
    else_if_blocks: Vec<Block>,
    else_block: Option<Block>,
}

impl AbstractTree for IfElse {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        let if_expression_node = node.child(0).unwrap().child(1).unwrap();
        let if_expression = Expression::from_syntax_node(source, if_expression_node, context)?;

        let if_block_node = node.child(0).unwrap().child(2).unwrap();
        let if_block = Block::from_syntax_node(source, if_block_node, context)?;

        let child_count = node.child_count();
        let mut else_if_expressions = Vec::new();
        let mut else_if_blocks = Vec::new();
        let mut else_block = None;

        for index in 1..child_count {
            let child = node.child(index).unwrap();

            if child.kind() == "else_if" {
                let expression_node = child.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node, context)?;

                else_if_expressions.push(expression);

                let block_node = child.child(2).unwrap();
                let block = Block::from_syntax_node(source, block_node, context)?;

                else_if_blocks.push(block);
            }

            if child.kind() == "else" {
                let else_node = child.child(1).unwrap();
                else_block = Some(Block::from_syntax_node(source, else_node, context)?);
            }
        }

        Ok(IfElse {
            if_expression,
            if_block,
            else_if_expressions,
            else_if_blocks,
            else_block,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let if_boolean = self.if_expression.run(source, context)?.as_boolean()?;

        if if_boolean {
            self.if_block.run(source, context)
        } else {
            let expressions = &self.else_if_expressions;

            for (index, expression) in expressions.iter().enumerate() {
                let if_boolean = expression.run(source, context)?.as_boolean()?;

                if if_boolean {
                    let block = self.else_if_blocks.get(index).unwrap();

                    return block.run(source, context);
                }
            }

            if let Some(block) = &self.else_block {
                block.run(source, context)
            } else {
                Ok(Value::none())
            }
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.if_block.expected_type(context)
    }
}

impl Display for IfElse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let IfElse {
            if_expression,
            if_block,
            else_if_expressions,
            else_if_blocks,
            else_block,
        } = self;

        write!(f, "if {if_expression} {{{if_block}}}")?;

        for (expression, block) in else_if_expressions.iter().zip(else_if_blocks.iter()) {
            write!(f, "else if {expression} {{{block}}}")?;
        }

        if let Some(block) = else_block {
            write!(f, "else {{{block}}}")?;
        }

        Ok(())
    }
}
