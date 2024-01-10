use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Block, Expression, Format, Map, Result, SyntaxNode, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    if_block: Block,
    else_if_expressions: Vec<Expression>,
    else_if_blocks: Vec<Block>,
    else_block: Option<Block>,
}

impl AbstractTree for IfElse {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        let if_expression_node = node.child(0).unwrap().child(1).unwrap();
        let if_expression = Expression::from_syntax(if_expression_node, source, context)?;

        let if_block_node = node.child(0).unwrap().child(2).unwrap();
        let if_block = Block::from_syntax(if_block_node, source, context)?;

        let child_count = node.child_count();
        let mut else_if_expressions = Vec::new();
        let mut else_if_blocks = Vec::new();
        let mut else_block = None;

        for index in 1..child_count {
            let child = node.child(index).unwrap();

            if child.kind() == "else_if" {
                let expression_node = child.child(1).unwrap();
                let expression = Expression::from_syntax(expression_node, source, context)?;

                else_if_expressions.push(expression);

                let block_node = child.child(2).unwrap();
                let block = Block::from_syntax(block_node, source, context)?;

                else_if_blocks.push(block);
            }

            if child.kind() == "else" {
                let else_node = child.child(1).unwrap();
                else_block = Some(Block::from_syntax(else_node, source, context)?);
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

impl Format for IfElse {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push_str("if ");
        self.if_expression.format(output, indent_level);
        output.push(' ');
        self.if_block.format(output, indent_level);

        let else_ifs = self
            .else_if_expressions
            .iter()
            .zip(self.else_if_blocks.iter());

        for (expression, block) in else_ifs {
            output.push_str("else if ");
            expression.format(output, indent_level);
            output.push(' ');
            block.format(output, indent_level);
        }

        if let Some(block) = &self.else_block {
            output.push_str("else ");
            block.format(output, indent_level);
        }
    }
}
