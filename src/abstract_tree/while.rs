use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    statement: Statement,
}

impl AbstractTree for While {
    fn from_syntax_node(source: &str, node: Node) -> crate::Result<Self> {
        debug_assert_eq!("while", node.kind());

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let statement_node = node.child(3).unwrap();
        let statement = Statement::from_syntax_node(source, statement_node)?;

        Ok(While {
            expression,
            statement,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        while self.expression.run(source, context)?.as_boolean()? {
            self.statement.run(source, context)?;
        }

        Ok(Value::Empty)
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate;

    #[test]
    fn evalualate_while_loop() {
        assert_eq!(evaluate("while false { 'foo' }"), Ok(crate::Value::Empty))
    }
}
