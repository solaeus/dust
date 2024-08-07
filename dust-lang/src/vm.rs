use std::collections::{HashMap, VecDeque};

use crate::{
    parse, Identifier, Node, ParseError, ReservedIdentifier, Span, Statement, Value, ValueError,
};

pub fn run(
    input: &str,
    variables: &mut HashMap<Identifier, Value>,
) -> Result<Option<Value>, VmError> {
    let abstract_syntax_tree = parse(input)?;
    let mut vm = Vm::new(abstract_syntax_tree);

    vm.run(variables)
}

pub struct Vm {
    statement_nodes: VecDeque<Node>,
}

impl Vm {
    pub fn new(statement_nodes: VecDeque<Node>) -> Self {
        Vm { statement_nodes }
    }

    pub fn run(
        &mut self,
        variables: &mut HashMap<Identifier, Value>,
    ) -> Result<Option<Value>, VmError> {
        let mut previous_value = None;

        while let Some(node) = self.statement_nodes.pop_front() {
            previous_value = self.run_node(node, variables)?;
        }

        Ok(previous_value)
    }

    fn run_node(
        &self,
        node: Node,
        variables: &mut HashMap<Identifier, Value>,
    ) -> Result<Option<Value>, VmError> {
        match node.statement {
            Statement::Constant(value) => Ok(Some(value.clone())),
            Statement::Identifier(_) => Ok(None),
            Statement::ReservedIdentifier(_) => Ok(None),
            Statement::Add(left, right) => {
                let left_span = left.span;
                let left = if let Some(value) = self.run_node(*left, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_span,
                    });
                };
                let right_span = right.span;
                let right = if let Some(value) = self.run_node(*right, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: right_span,
                    });
                };
                let sum = left.add(&right)?;

                Ok(Some(sum))
            }
            Statement::Assign(left, right) => {
                let identifier = if let Statement::Identifier(identifier) = &left.statement {
                    identifier
                } else {
                    return Err(VmError::ExpectedIdentifier {
                        position: left.span,
                    });
                };
                let right_span = right.span;
                let value = if let Some(value) = self.run_node(*right, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: right_span,
                    });
                };

                variables.insert(identifier.clone(), value);

                Ok(None)
            }
            Statement::List(nodes) => {
                let values = nodes
                    .into_iter()
                    .map(|node| {
                        let span = node.span;
                        if let Some(value) = self.run_node(node, variables)? {
                            Ok(value)
                        } else {
                            Err(VmError::ExpectedValue { position: span })
                        }
                    })
                    .collect::<Result<Vec<Value>, VmError>>()?;

                Ok(Some(Value::list(values)))
            }
            Statement::Multiply(_, _) => todo!(),
            Statement::PropertyAccess(left, right) => {
                let left_span = left.span;
                let left = if let Some(value) = self.run_node(*left, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_span,
                    });
                };
                let right_span = right.span;

                if let Statement::ReservedIdentifier(reserved) = &right.statement {
                    match reserved {
                        ReservedIdentifier::IsEven => {
                            if let Some(integer) = left.as_integer() {
                                return Ok(Some(Value::boolean(integer % 2 == 0)));
                            } else {
                                return Err(VmError::ExpectedInteger {
                                    position: right_span,
                                });
                            }
                        }
                        ReservedIdentifier::IsOdd => {
                            if let Some(integer) = left.as_integer() {
                                return Ok(Some(Value::boolean(integer % 2 != 0)));
                            } else {
                                return Err(VmError::ExpectedInteger {
                                    position: right_span,
                                });
                            }
                        }
                        ReservedIdentifier::Length => {
                            if let Some(list) = left.as_list() {
                                return Ok(Some(Value::integer(list.len() as i64)));
                            } else {
                                return Err(VmError::ExpectedList {
                                    position: right_span,
                                });
                            }
                        }
                    }
                }

                if let (Some(list), Statement::Constant(value)) = (left.as_list(), &right.statement)
                {
                    if let Some(index) = value.as_integer() {
                        let value = list.get(index as usize).cloned();

                        return Ok(value);
                    }
                }

                Err(VmError::ExpectedIdentifierOrInteger {
                    position: right_span,
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    ParseError(ParseError),
    ValueError(ValueError),

    // Anaylsis Failures
    // These should be prevented by running the analyzer before the VM
    ExpectedIdentifier { position: Span },
    ExpectedIdentifierOrInteger { position: Span },
    ExpectedInteger { position: Span },
    ExpectedList { position: Span },
    ExpectedValue { position: Span },
}

impl From<ParseError> for VmError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl From<ValueError> for VmError {
    fn from(error: ValueError) -> Self {
        Self::ValueError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_even() {
        let input = "42.is_even";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn is_odd() {
        let input = "42.is_odd";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(false)))
        );
    }

    #[test]
    fn list_access() {
        let input = "[1, 2, 3].1";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn property_access() {
        let input = "[1, 2, 3].length";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn add_multiple() {
        let input = "1 + 2 + 3";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(6))));
    }
}
