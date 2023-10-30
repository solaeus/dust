use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Expression,
    then_statement: Statement,
    else_if_expressions: Vec<Expression>,
    else_if_statements: Vec<Statement>,
    else_statement: Option<Statement>,
}

impl AbstractTree for IfElse {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let if_node = node.child(0).unwrap().child(1).unwrap();
        let if_expression = Expression::from_syntax_node(source, if_node)?;

        let then_node = node.child(0).unwrap().child(3).unwrap();
        let then_statement = Statement::from_syntax_node(source, then_node)?;

        let child_count = node.child_count();
        let mut else_if_expressions = Vec::new();
        let mut else_if_statements = Vec::new();
        let mut else_statement = None;

        for index in 1..child_count {
            let child = node.child(index);

            if let Some(node) = child {
                if node.kind() == "else_if" {
                    let expression_node = node.child(1).unwrap();
                    let expression = Expression::from_syntax_node(source, expression_node)?;

                    else_if_expressions.push(expression);

                    let statement_node = node.child(3).unwrap();
                    let statement = Statement::from_syntax_node(source, statement_node)?;

                    else_if_statements.push(statement);
                }

                if node.kind() == "else" {
                    let else_node = node.child(2).unwrap();
                    else_statement = Some(Statement::from_syntax_node(source, else_node)?);
                }
            }
        }

        Ok(IfElse {
            if_expression,
            then_statement,
            else_if_expressions,
            else_if_statements,
            else_statement,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let if_boolean = self.if_expression.run(source, context)?.as_boolean()?;

        if if_boolean {
            self.then_statement.run(source, context)
        } else {
            let expressions = &self.else_if_expressions;

            for (index, expression) in expressions.iter().enumerate() {
                let if_boolean = expression.run(source, context)?.as_boolean()?;

                if if_boolean {
                    let statement = self.else_if_statements.get(index).unwrap();

                    return statement.run(source, context);
                }
            }

            if let Some(statement) = &self.else_statement {
                statement.run(source, context)
            } else {
                Ok(Value::Empty)
            }
        }
    }
}
