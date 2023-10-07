use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, Statement};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    statement: Statement,
}

impl AbstractTree for While {
    fn from_syntax_node(node: tree_sitter::Node, source: &str) -> crate::Result<Self> {
        debug_assert_eq!("while", node.kind());

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax_node(expression_node, source)?;

        let statement_node = node.child(3).unwrap();
        let statement = Statement::from_syntax_node(statement_node, source)?;

        Ok(While {
            expression,
            statement,
        })
    }

    fn run(&self, context: &mut crate::VariableMap) -> crate::Result<crate::Value> {
        while self.expression.run(context)?.as_boolean()? {
            self.statement.run(context)?;
        }

        Ok(crate::Value::Empty)
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate;

    #[test]
    fn evalualate_while_loop() {
        assert_eq!(
            evaluate("while false { 'foo' }"),
            vec![Ok(crate::Value::Empty)]
        )
    }
}
