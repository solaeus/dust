use crate::{AbstractSyntaxTree, Node, Statement};

pub fn analyze(abstract_tree: &AbstractSyntaxTree) -> Result<(), AnalyzerError> {
    let analyzer = Analyzer::new(abstract_tree);

    analyzer.analyze()
}

pub struct Analyzer<'a> {
    abstract_tree: &'a AbstractSyntaxTree,
}

impl<'a> Analyzer<'a> {
    pub fn new(abstract_tree: &'a AbstractSyntaxTree) -> Self {
        Analyzer { abstract_tree }
    }

    pub fn analyze(&self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree.nodes {
            self.analyze_node(node)?;
        }

        Ok(())
    }

    fn analyze_node(&self, node: &Node) -> Result<(), AnalyzerError> {
        match &node.statement {
            Statement::Add(left, right) => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::Assign(left, right) => {
                if let Statement::Identifier(_) = &left.statement {
                    // Identifier is in the correct position
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: left.as_ref().clone(),
                    });
                }

                self.analyze_node(right)?;
            }
            Statement::BuiltInValue(node) => {
                self.analyze_node(node)?;
            }
            Statement::Constant(_) => {}
            Statement::Identifier(_) => {
                return Err(AnalyzerError::UnexpectedIdentifier {
                    identifier: node.clone(),
                });
            }
            Statement::List(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
            }
            Statement::Multiply(left, right) => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::PropertyAccess(left, right) => {
                if let Statement::Identifier(_) = &left.statement {
                    // Identifier is in the correct position
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: left.as_ref().clone(),
                    });
                }

                self.analyze_node(right)?;
            }
            Statement::ReservedIdentifier(_) => {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedIdentifier { actual: Node },
    UnexpectedIdentifier { identifier: Node },
}

#[cfg(test)]
mod tests {
    use crate::{Identifier, Value};

    use super::*;

    #[test]
    fn assignment_expect_identifier() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Assign(
                    Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    Box::new(Node::new(Statement::Constant(Value::integer(2)), (1, 2))),
                ),
                (0, 2),
            )]
            .into(),
        };

        let analyzer = Analyzer::new(&abstract_tree);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIdentifier {
                actual: Node::new(Statement::Constant(Value::integer(1)), (0, 1))
            })
        )
    }

    #[test]
    fn unexpected_identifier_simple() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Identifier(Identifier::new("x")),
                (0, 1),
            )]
            .into(),
        };

        let analyzer = Analyzer::new(&abstract_tree);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::UnexpectedIdentifier {
                identifier: Node::new(Statement::Identifier(Identifier::new("x")), (0, 1))
            })
        )
    }

    #[test]
    fn unexpected_identifier_nested() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Add(
                    Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    Box::new(Node::new(
                        Statement::Identifier(Identifier::new("x")),
                        (1, 2),
                    )),
                ),
                (0, 1),
            )]
            .into(),
        };

        let analyzer = Analyzer::new(&abstract_tree);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::UnexpectedIdentifier {
                identifier: Node::new(Statement::Identifier(Identifier::new("x")), (1, 2))
            })
        )
    }
}
