//! Virtual machine for running the abstract syntax tree.
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::BinaryOperator, parse, value::ValueInner, AbstractSyntaxTree, Analyzer,
    BuiltInFunctionError, Context, DustError, Node, ParseError, Span, Statement, Value, ValueError,
};

pub fn run<'src>(
    source: &'src str,
    context: &mut Context,
) -> Result<Option<Value>, DustError<'src>> {
    let abstract_syntax_tree = parse(source)?;
    let mut analyzer = Analyzer::new(&abstract_syntax_tree, context);

    analyzer
        .analyze()
        .map_err(|analyzer_error| DustError::AnalyzerError {
            analyzer_error,
            source,
        })?;

    let mut vm = Vm::new(abstract_syntax_tree);

    vm.run(context)
        .map_err(|vm_error| DustError::VmError { vm_error, source })
}

pub struct Vm {
    abstract_tree: AbstractSyntaxTree,
}

impl Vm {
    pub fn new(abstract_tree: AbstractSyntaxTree) -> Self {
        Self { abstract_tree }
    }

    pub fn run(&mut self, context: &mut Context) -> Result<Option<Value>, VmError> {
        let mut previous_value = None;

        while let Some(node) = self.abstract_tree.nodes.pop_front() {
            previous_value = self.run_node(node, context)?;
        }

        Ok(previous_value)
    }

    fn run_node(
        &self,
        node: Node<Statement>,
        context: &mut Context,
    ) -> Result<Option<Value>, VmError> {
        match node.inner {
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let right_position = right.position;

                if let BinaryOperator::Assign = operator.inner {
                    let identifier = if let Statement::Identifier(identifier) = left.inner {
                        identifier
                    } else {
                        return Err(VmError::ExpectedIdentifier {
                            position: left.position,
                        });
                    };

                    let value = if let Some(value) = self.run_node(*right, context)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: right_position,
                        });
                    };

                    context.set_value(identifier, value);

                    return Ok(None);
                }

                if let BinaryOperator::AddAssign = operator.inner {
                    let identifier = if let Statement::Identifier(identifier) = left.inner {
                        identifier
                    } else {
                        return Err(VmError::ExpectedIdentifier {
                            position: left.position,
                        });
                    };
                    let right_value = if let Some(value) = self.run_node(*right, context)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: right_position,
                        });
                    };
                    let left_value = context.use_value(&identifier).ok_or_else(|| {
                        VmError::UndefinedVariable {
                            identifier: Node::new(
                                Statement::Identifier(identifier.clone()),
                                left.position,
                            ),
                        }
                    })?;
                    let new_value = left_value.add(&right_value).map_err(|value_error| {
                        VmError::ValueError {
                            error: value_error,
                            position: right_position,
                        }
                    })?;

                    context.set_value(identifier, new_value);

                    return Ok(None);
                }

                let left_position = left.position;
                let left_value = if let Some(value) = self.run_node(*left, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_position,
                    });
                };
                let right_value = if let Some(value) = self.run_node(*right, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: right_position,
                    });
                };

                match operator.inner {
                    BinaryOperator::Add => left_value.add(&right_value),
                    BinaryOperator::And => left_value.and(&right_value),
                    BinaryOperator::Divide => left_value.divide(&right_value),
                    BinaryOperator::Equal => Ok(Value::boolean(left_value == right_value)),
                    BinaryOperator::Greater => left_value.greater_than(&right_value),
                    BinaryOperator::GreaterOrEqual => {
                        left_value.greater_than_or_equal(&right_value)
                    }
                    BinaryOperator::Less => left_value.less_than(&right_value),
                    BinaryOperator::LessOrEqual => left_value.less_than_or_equal(&right_value),
                    BinaryOperator::Modulo => left_value.modulo(&right_value),
                    BinaryOperator::Multiply => left_value.multiply(&right_value),
                    BinaryOperator::Or => left_value.or(&right_value),
                    BinaryOperator::Subtract => left_value.subtract(&right_value),
                    _ => unreachable!(),
                }
                .map(Some)
                .map_err(|value_error| VmError::ValueError {
                    error: value_error,
                    position: node.position,
                })
            }
            Statement::Block(statements) => {
                let mut previous_value = None;

                for statement in statements {
                    previous_value = self.run_node(statement, context)?;
                }

                Ok(previous_value)
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
                        let value = if let Some(value) = self.run_node(node, context)? {
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
            Statement::Constant(value) => Ok(Some(value.clone())),
            Statement::FunctionCall {
                function: function_node,
                type_arguments: _,
                value_arguments: value_parameter_nodes,
            } => {
                let function_position = function_node.position;
                let function_value = if let Some(value) = self.run_node(*function_node, context)? {
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
                        let value = if let Some(value) = self.run_node(node, context)? {
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

                Ok(function.clone().call(None, value_parameters, context)?)
            }
            Statement::Identifier(identifier) => {
                if let Some(value) = context.use_value(&identifier) {
                    Ok(Some(value.clone()))
                } else {
                    Err(VmError::UndefinedVariable {
                        identifier: Node::new(Statement::Identifier(identifier), node.position),
                    })
                }
            }
            Statement::If { condition, body } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_node(*condition, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };
                let condition = if let Some(condition) = condition_value.as_boolean() {
                    condition
                } else {
                    return Err(VmError::ExpectedBoolean {
                        position: condition_position,
                    });
                };

                if condition {
                    self.run_node(*body, context)?;
                }

                Ok(None)
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_node(*condition, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_node(*if_body, context)
                    } else {
                        self.run_node(*else_body, context)
                    }
                } else {
                    Err(VmError::ExpectedBoolean {
                        position: condition_position,
                    })
                }
            }
            Statement::IfElseIf {
                condition,
                if_body,
                else_ifs,
            } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_node(*condition, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_node(*if_body, context)
                    } else {
                        for (condition, body) in else_ifs {
                            let condition_position = condition.position;
                            let condition_value =
                                if let Some(value) = self.run_node(condition, context)? {
                                    value
                                } else {
                                    return Err(VmError::ExpectedValue {
                                        position: condition_position,
                                    });
                                };
                            let condition = if let Some(condition) = condition_value.as_boolean() {
                                condition
                            } else {
                                return Err(VmError::ExpectedBoolean {
                                    position: condition_position,
                                });
                            };

                            if condition {
                                self.run_node(body, context)?;
                            }
                        }

                        Ok(None)
                    }
                } else {
                    Err(VmError::ExpectedBoolean {
                        position: condition_position,
                    })
                }
            }
            Statement::IfElseIfElse {
                condition,
                if_body,
                else_ifs,
                else_body,
            } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_node(*condition, context)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_node(*if_body, context)
                    } else {
                        for (condition, body) in else_ifs {
                            let condition_position = condition.position;
                            let condition_value =
                                if let Some(value) = self.run_node(condition, context)? {
                                    value
                                } else {
                                    return Err(VmError::ExpectedValue {
                                        position: condition_position,
                                    });
                                };
                            let condition = if let Some(condition) = condition_value.as_boolean() {
                                condition
                            } else {
                                return Err(VmError::ExpectedBoolean {
                                    position: condition_position,
                                });
                            };

                            if condition {
                                return self.run_node(body, context);
                            }
                        }

                        self.run_node(*else_body, context)
                    }
                } else {
                    Err(VmError::ExpectedBoolean {
                        position: condition_position,
                    })
                }
            }
            Statement::List(nodes) => {
                let values = nodes
                    .into_iter()
                    .map(|node| {
                        let span = node.position;
                        if let Some(value) = self.run_node(node, context)? {
                            Ok(value)
                        } else {
                            Err(VmError::ExpectedValue { position: span })
                        }
                    })
                    .collect::<Result<Vec<Value>, VmError>>()?;

                Ok(Some(Value::list(values)))
            }
            Statement::Map(nodes) => {
                let mut values = BTreeMap::new();

                for (identifier, value_node) in nodes {
                    let identifier = if let Statement::Identifier(identifier) = identifier.inner {
                        identifier
                    } else {
                        return Err(VmError::ExpectedIdentifier {
                            position: identifier.position,
                        });
                    };
                    let position = value_node.position;
                    let value = if let Some(value) = self.run_node(value_node, context)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue { position });
                    };

                    values.insert(identifier, value);
                }

                Ok(Some(Value::map(values)))
            }
            Statement::Nil(node) => {
                let _return = self.run_node(*node, context)?;

                Ok(None)
            }
            Statement::PropertyAccess(left, right) => {
                let left_span = left.position;
                let left_value = if let Some(value) = self.run_node(*left, context)? {
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
                            let value = if let Some(value) = self.run_node(node, context)? {
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
            Statement::While { condition, body } => {
                let mut return_value = None;

                let condition_position = condition.position;

                while let Some(condition_value) = self.run_node(*condition.clone(), context)? {
                    if let ValueInner::Boolean(condition_value) = condition_value.inner().as_ref() {
                        if !condition_value {
                            break;
                        }
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            position: condition_position,
                        });
                    }

                    return_value = self.run_node(*body.clone(), context)?;

                    if return_value.is_some() {
                        break;
                    }
                }

                Ok(return_value)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
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
    ExpectedBoolean {
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
    UndefinedVariable {
        identifier: Node<Statement>,
    },
}

impl VmError {
    pub fn position(&self) -> Span {
        match self {
            Self::ParseError(parse_error) => parse_error.position(),
            Self::ValueError { position, .. } => *position,
            Self::BuiltInFunctionError { position, .. } => *position,
            Self::ExpectedBoolean { position } => *position,
            Self::ExpectedIdentifier { position } => *position,
            Self::ExpectedIdentifierOrInteger { position } => *position,
            Self::ExpectedInteger { position } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedList { position } => *position,
            Self::ExpectedValue { position } => *position,
            Self::UndefinedVariable { identifier } => identifier.position,
        }
    }
}

impl From<ParseError> for VmError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ParseError(parse_error) => write!(f, "{}", parse_error),
            Self::ValueError { error, .. } => write!(f, "{}", error),
            Self::BuiltInFunctionError { error, .. } => {
                write!(f, "{}", error)
            }
            Self::ExpectedBoolean { position } => {
                write!(f, "Expected a boolean at position: {:?}", position)
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
            Self::UndefinedVariable { identifier } => {
                write!(f, "Undefined identifier: {}", identifier)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let input = "42.to_string()";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::string("42".to_string())))
        );
    }

    #[test]
    fn r#if() {
        let input = "if true { 1 }";

        assert_eq!(run(input, &mut Context::new()), Ok(None));
    }

    #[test]
    fn if_else() {
        let input = "if false { 1 } else { 2 }";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn if_else_if() {
        let input = "if false { 1 } else if true { 2 }";

        assert_eq!(run(input, &mut Context::new()), Ok(None));
    }

    #[test]
    fn if_else_if_else() {
        let input = "if false { 1 } else if false { 2 } else { 3 }";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn while_loop() {
        let input = "x = 0; while x < 5 { x += 1; } x";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(5))));
    }

    #[test]
    fn add_assign() {
        let input = "x = 1; x += 1; x";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn map_equal() {
        let input = "{ y = 'foo' } == { y = 'foo' }";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn integer_equal() {
        let input = "42 == 42";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(0))));
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::integer(21)))
        );
    }

    #[test]
    fn less_than() {
        let input = "2 < 3";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn less_than_or_equal() {
        let input = "42 <= 42";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn greater_than() {
        let input = "2 > 3";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(false)))
        );
    }

    #[test]
    fn greater_than_or_equal() {
        let input = "42 >= 42";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn integer_saturating_add() {
        let input = "9223372036854775807 + 1";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::integer(i64::MAX)))
        );
    }

    #[test]
    fn integer_saturating_sub() {
        let input = "-9223372036854775808 - 1";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::integer(i64::MIN)))
        );
    }

    #[test]
    fn multiply() {
        let input = "2 * 3";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(6))));
    }

    #[test]
    fn boolean() {
        let input = "true";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn is_even() {
        let input = "42.is_even()";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(true)))
        );
    }

    #[test]
    fn is_odd() {
        let input = "42.is_odd()";

        assert_eq!(
            run(input, &mut Context::new()),
            Ok(Some(Value::boolean(false)))
        );
    }

    #[test]
    fn length() {
        let input = "[1, 2, 3].length()";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn list_access() {
        let input = "[1, 2, 3].1";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn add_multiple() {
        let input = "1 + 2 + 3";

        assert_eq!(run(input, &mut Context::new()), Ok(Some(Value::integer(6))));
    }
}
