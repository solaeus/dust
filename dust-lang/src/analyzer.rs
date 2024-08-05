use crate::{Node, Span, Statement};

pub struct Analyzer {
    abstract_tree: Vec<Node>,
}

impl Analyzer {
    pub fn new(abstract_tree: Vec<Node>) -> Self {
        Analyzer { abstract_tree }
    }

    pub fn analyze(&self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree {
            self.analyze_node(node)?;
        }

        Ok(())
    }

    fn analyze_node(&self, node: &Node) -> Result<(), AnalyzerError> {
        match &node.operation {
            Statement::Add(instructions) => {
                self.analyze_node(&instructions.0)?;
                self.analyze_node(&instructions.1)?;
            }
            Statement::Assign(instructions) => {
                if let Statement::Identifier(_) = &instructions.0.operation {
                    // Identifier
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: instructions.0.clone(),
                    });
                }

                self.analyze_node(&instructions.0)?;
                self.analyze_node(&instructions.1)?;
            }
            Statement::Constant(_) => {}
            Statement::Identifier(_) => {}
            Statement::List(instructions) => {
                for instruction in instructions {
                    self.analyze_node(instruction)?;
                }
            }
            Statement::Multiply(instructions) => {
                self.analyze_node(&instructions.0)?;
                self.analyze_node(&instructions.1)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedIdentifier { actual: Node },
}

#[cfg(test)]
mod tests {
    use crate::Value;

    use super::*;

    #[test]
    fn analyze() {
        let abstract_tree = vec![Node::new(
            Statement::Assign(Box::new((
                Node::new(Statement::Constant(Value::integer(1)), (0, 1)),
                Node::new(Statement::Constant(Value::integer(2)), (1, 2)),
            ))),
            (0, 1),
        )];

        let analyzer = Analyzer::new(abstract_tree);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIdentifier {
                actual: Node::new(Statement::Constant(Value::integer(1)), (0, 1))
            })
        )
    }
}
