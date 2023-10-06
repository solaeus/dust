use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Result, Statement, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    then_statement: Statement,
    else_statement: Option<Statement>,
}

impl AbstractTree for IfElse {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        let if_node = node.child(1).unwrap();
        let if_expression = Expression::from_syntax_node(if_node, source)?;

        let then_node = node.child(3).unwrap();
        let then_statement = Statement::from_syntax_node(then_node, source)?;

        let else_node = node.child(5);
        let else_statement = if let Some(node) = else_node {
            Some(Statement::from_syntax_node(node, source)?)
        } else {
            None
        };

        println!("{if_node:?} {then_node:?} {else_node:?}");

        Ok(IfElse {
            if_expression,
            then_statement,
            else_statement,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let if_boolean = self.if_expression.run(context)?.as_boolean()?;

        if if_boolean {
            self.then_statement.run(context)
        } else if let Some(statement) = &self.else_statement {
            statement.run(context)
        } else {
            Ok(Value::Empty)
        }
    }
}
