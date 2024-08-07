use std::collections::HashMap;

use crate::{
    parse, AbstractSyntaxTree, Analyzer, AnalyzerError, BuiltInFunctionError, Identifier, Node,
    ParseError, Span, Statement, Value, ValueError,
};

pub fn run(
    input: &str,
    variables: &mut HashMap<Identifier, Value>,
) -> Result<Option<Value>, VmError> {
    let abstract_syntax_tree = parse(input)?;
    let analyzer = Analyzer::new(&abstract_syntax_tree, variables);

    analyzer.analyze()?;

    let mut vm = Vm::new(abstract_syntax_tree);

    vm.run(variables)
}

pub struct Vm {
    abstract_tree: AbstractSyntaxTree,
}

impl Vm {
    pub fn new(abstract_tree: AbstractSyntaxTree) -> Self {
        Self { abstract_tree }
    }

    pub fn run(
        &mut self,
        variables: &mut HashMap<Identifier, Value>,
    ) -> Result<Option<Value>, VmError> {
        let mut previous_value = None;

        while let Some(node) = self.abstract_tree.nodes.pop_front() {
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
            Statement::Add(left, right) => {
                let left_span = left.position;
                let left = if let Some(value) = self.run_node(*left, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_span,
                    });
                };
                let right_span = right.position;
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
                        position: left.position,
                    });
                };
                let right_span = right.position;
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
            Statement::BuiltInFunctionCall {
                function,
                type_arguments: _,
                value_arguments: value_nodes,
            } => {
                let values = if let Some(nodes) = value_nodes {
                    let mut values = Vec::new();

                    for node in nodes {
                        let position = node.position;
                        let value = if let Some(value) = self.run_node(node, variables)? {
                            value
                        } else {
                            return Err(VmError::ExpectedValue { position });
                        };

                        values.push(value);
                    }

                    Some(values)
                } else {
                    None
                };
                let function_call_return = function.call(None, values)?;

                Ok(Some(function_call_return))
            }
            Statement::Constant(value) => Ok(Some(value.clone())),
            Statement::FunctionCall {
                function: function_node,
                type_arguments: _,
                value_arguments: value_parameter_nodes,
            } => {
                let function_position = function_node.position;
                let function_value =
                    if let Some(value) = self.run_node(*function_node, variables)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: function_position,
                        });
                    };
                let function = if let Some(function) = function_value.as_function() {
                    function
                } else {
                    return Err(VmError::AnaylyzerError(AnalyzerError::ExpectedFunction {
                        position: function_position,
                    }));
                };

                let value_parameters = if let Some(value_nodes) = value_parameter_nodes {
                    let mut value_parameters = Vec::new();

                    for node in value_nodes {
                        let position = node.position;
                        let value = if let Some(value) = self.run_node(node, variables)? {
                            value
                        } else {
                            return Err(VmError::ExpectedValue { position });
                        };

                        value_parameters.push(value);
                    }

                    Some(value_parameters)
                } else {
                    None
                };

                Ok(function.clone().call(None, value_parameters, variables)?)
            }
            Statement::Identifier(_) => Ok(None),
            Statement::List(nodes) => {
                let values = nodes
                    .into_iter()
                    .map(|node| {
                        let span = node.position;
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
                let left_span = left.position;
                let left_value = if let Some(value) = self.run_node(*left, variables)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_span,
                    });
                };
                let right_span = right.position;

                if let (Some(list), Statement::Constant(value)) =
                    (left_value.as_list(), &right.statement)
                {
                    if let Some(index) = value.as_integer() {
                        let value = list.get(index as usize).cloned();

                        return Ok(value);
                    }
                }

                if let (
                    value,
                    Statement::BuiltInFunctionCall {
                        function,
                        type_arguments: _,
                        value_arguments: value_argument_nodes,
                    },
                ) = (left_value, right.statement)
                {
                    let mut value_arguments = Vec::new();

                    value_arguments.push(value);

                    if let Some(value_nodes) = value_argument_nodes {
                        for node in value_nodes {
                            let position = node.position;
                            let value = if let Some(value) = self.run_node(node, variables)? {
                                value
                            } else {
                                return Err(VmError::ExpectedValue { position });
                            };

                            value_arguments.push(value);
                        }
                    }

                    let function_call_return = function.call(None, Some(value_arguments))?;

                    return Ok(Some(function_call_return));
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
    AnaylyzerError(AnalyzerError),
    ParseError(ParseError),
    ValueError(ValueError),

    // Anaylsis Failures
    // These should be prevented by running the analyzer before the VM
    BuiltInFunctionCallFailed(BuiltInFunctionError),
    ExpectedIdentifier { position: Span },
    ExpectedIdentifierOrInteger { position: Span },
    ExpectedInteger { position: Span },
    ExpectedList { position: Span },
    ExpectedValue { position: Span },
}

impl From<BuiltInFunctionError> for VmError {
    fn from(v: BuiltInFunctionError) -> Self {
        Self::BuiltInFunctionCallFailed(v)
    }
}

impl From<AnalyzerError> for VmError {
    fn from(error: AnalyzerError) -> Self {
        Self::AnaylyzerError(error)
    }
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
    fn boolean() {
        let input = "true";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn is_even() {
        let input = "42.is_even()";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn is_odd() {
        let input = "42.is_odd()";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(false)))
        );
    }

    #[test]
    fn length() {
        let input = "[1, 2, 3].length()";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn list_access() {
        let input = "[1, 2, 3].1";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(2))));
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
