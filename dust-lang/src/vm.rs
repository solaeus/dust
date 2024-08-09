//! Virtual machine for running the abstract syntax tree.
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::ComparisonOperator, parse, AbstractSyntaxTree, Analyzer, AnalyzerError,
    BuiltInFunctionError, Identifier, Node, ParseError, Span, Statement, Value, ValueError,
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
        node: Node<Statement>,
        variables: &mut HashMap<Identifier, Value>,
    ) -> Result<Option<Value>, VmError> {
        match node.inner {
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
                let sum = left
                    .add(&right)
                    .map_err(|value_error| VmError::ValueError {
                        error: value_error,
                        position: (left_span.0, right_span.1),
                    })?;

                Ok(Some(sum))
            }
            Statement::Assign(left, right) => {
                let identifier = if let Statement::Identifier(identifier) = &left.inner {
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
                let function_call_return =
                    function
                        .call(None, values)
                        .map_err(|built_in_function_error| VmError::BuiltInFunctionError {
                            error: built_in_function_error,
                            position: node.position,
                        })?;

                Ok(function_call_return)
            }
            Statement::Comparison(left, operator, right) => {
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
                let comparison = match operator.inner {
                    ComparisonOperator::GreaterThan => {
                        left.greater_than(&right)
                            .map_err(|value_error| VmError::ValueError {
                                error: value_error,
                                position: (left_span.0, right_span.1),
                            })?
                    }
                    ComparisonOperator::GreaterThanOrEqual => left
                        .greater_than_or_equal(&right)
                        .map_err(|value_error| VmError::ValueError {
                            error: value_error,
                            position: (left_span.0, right_span.1),
                        })?,
                    ComparisonOperator::LessThan => {
                        left.less_than(&right)
                            .map_err(|value_error| VmError::ValueError {
                                error: value_error,
                                position: (left_span.0, right_span.1),
                            })?
                    }
                    ComparisonOperator::LessThanOrEqual => left
                        .less_than_or_equal(&right)
                        .map_err(|value_error| VmError::ValueError {
                            error: value_error,
                            position: (left_span.0, right_span.1),
                        })?,
                };

                Ok(Some(comparison))
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
                    return Err(VmError::ExpectedFunction {
                        actual: function_value,
                        position: function_position,
                    });
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
            Statement::Identifier(identifier) => {
                if let Some(value) = variables.get(&identifier) {
                    Ok(Some(value.clone()))
                } else {
                    Err(VmError::UndefinedIdentifier {
                        identifier,
                        position: node.position,
                    })
                }
            }
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
            Statement::Multiply(left, right) => {
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
                let product = left
                    .multiply(&right)
                    .map_err(|value_error| VmError::ValueError {
                        error: value_error,
                        position: (left_span.0, right_span.1),
                    })?;

                Ok(Some(product))
            }
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
                    (left_value.as_list(), &right.inner)
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
                ) = (left_value, right.inner)
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

                    let function_call_return = function.call(None, Some(value_arguments)).map_err(
                        |built_in_function_error| VmError::BuiltInFunctionError {
                            error: built_in_function_error,
                            position: right_span,
                        },
                    )?;

                    return Ok(function_call_return);
                }

                Err(VmError::ExpectedIdentifierOrInteger {
                    position: right_span,
                })
            }
            Statement::Subtract(left, right) => {
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
                let difference =
                    left.subtract(&right)
                        .map_err(|value_error| VmError::ValueError {
                            error: value_error,
                            position: (left_span.0, right_span.1),
                        })?;

                Ok(Some(difference))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    AnaylyzerError(AnalyzerError),
    ParseError(ParseError),
    ValueError {
        error: ValueError,
        position: Span,
    },

    // Anaylsis Failures
    // These should be prevented by running the analyzer before the VM
    BuiltInFunctionError {
        error: BuiltInFunctionError,
        position: Span,
    },
    ExpectedIdentifier {
        position: Span,
    },
    ExpectedIdentifierOrInteger {
        position: Span,
    },
    ExpectedInteger {
        position: Span,
    },
    ExpectedFunction {
        actual: Value,
        position: Span,
    },
    ExpectedList {
        position: Span,
    },
    ExpectedValue {
        position: Span,
    },
    UndefinedIdentifier {
        identifier: Identifier,
        position: Span,
    },
}

impl VmError {
    pub fn position(&self) -> Span {
        match self {
            Self::AnaylyzerError(analyzer_error) => analyzer_error.position(),
            Self::ParseError(parse_error) => parse_error.position(),
            Self::ValueError { position, .. } => *position,
            Self::BuiltInFunctionError { position, .. } => *position,
            Self::ExpectedIdentifier { position } => *position,
            Self::ExpectedIdentifierOrInteger { position } => *position,
            Self::ExpectedInteger { position } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedList { position } => *position,
            Self::ExpectedValue { position } => *position,
            Self::UndefinedIdentifier { position, .. } => *position,
        }
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

impl Error for VmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AnaylyzerError(analyzer_error) => Some(analyzer_error),
            Self::ParseError(parse_error) => Some(parse_error),
            Self::ValueError { error, .. } => Some(error),
            Self::BuiltInFunctionError { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::AnaylyzerError(analyzer_error) => write!(f, "{}", analyzer_error),
            Self::ParseError(parse_error) => write!(f, "{}", parse_error),
            Self::ValueError { error, .. } => write!(f, "{}", error),
            Self::BuiltInFunctionError { error, .. } => {
                write!(f, "{}", error)
            }
            Self::ExpectedFunction { actual, position } => {
                write!(
                    f,
                    "Expected a function, but got: {} at position: {:?}",
                    actual, position
                )
            }
            Self::ExpectedIdentifier { position } => {
                write!(f, "Expected an identifier at position: {:?}", position)
            }
            Self::ExpectedIdentifierOrInteger { position } => {
                write!(
                    f,
                    "Expected an identifier or integer at position: {:?}",
                    position
                )
            }
            Self::ExpectedInteger { position } => {
                write!(f, "Expected an integer at position: {:?}", position)
            }
            Self::ExpectedList { position } => {
                write!(f, "Expected a list at position: {:?}", position)
            }
            Self::ExpectedValue { position } => {
                write!(f, "Expected a value at position: {:?}", position)
            }
            Self::UndefinedIdentifier {
                identifier,
                position,
            } => {
                write!(
                    f,
                    "Undefined identifier: {} at position: {:?}",
                    identifier, position
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn less_than() {
        let input = "2 < 3";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn less_than_or_equal() {
        let input = "42 <= 42";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn greater_than() {
        let input = "2 > 3";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(false)))
        );
    }

    #[test]
    fn greater_than_or_equal() {
        let input = "42 >= 42";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn integer_saturating_add() {
        let input = "9223372036854775807 + 1";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::integer(i64::MAX)))
        );
    }

    #[test]
    fn integer_saturating_sub() {
        let input = "-9223372036854775808 - 1";

        assert_eq!(
            run(input, &mut HashMap::new()),
            Ok(Some(Value::integer(i64::MIN)))
        );
    }

    #[test]
    fn multiply() {
        let input = "2 * 3";

        assert_eq!(run(input, &mut HashMap::new()), Ok(Some(Value::integer(6))));
    }

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
