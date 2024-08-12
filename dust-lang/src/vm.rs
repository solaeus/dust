//! Virtual machine for running the abstract syntax tree.
//!
//! This module provides three running option:
//! - `run` convenience function that takes a source code string and runs it
//! - `run_with_context` convenience function that takes a source code string and a context
//! - `Vm` struct that can be used to run an abstract syntax tree
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::{
    parse, value::ValueInner, AbstractSyntaxTree, Analyzer, BinaryOperator, BuiltInFunctionError,
    Context, DustError, Identifier, Node, ParseError, Span, Statement, UnaryOperator, Value,
    ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let context = Context::new();

    run_with_context(source, context)
}

pub fn run_with_context(source: &str, context: Context) -> Result<Option<Value>, DustError> {
    let abstract_syntax_tree = parse(source)?;
    let mut analyzer = Analyzer::new(&abstract_syntax_tree, &context);

    analyzer
        .analyze()
        .map_err(|analyzer_error| DustError::AnalyzerError {
            analyzer_error,
            source,
        })?;

    let mut vm = Vm::new(abstract_syntax_tree, context);

    vm.run()
        .map_err(|vm_error| DustError::VmError { vm_error, source })
}

pub struct Vm {
    abstract_tree: AbstractSyntaxTree,
    context: Context,
}

impl Vm {
    pub fn new(abstract_tree: AbstractSyntaxTree, context: Context) -> Self {
        Self {
            abstract_tree,
            context,
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        let mut previous_position = (0, 0);
        let mut previous_value = None;

        while let Some(statement) = self.abstract_tree.nodes.pop_front() {
            let new_position = statement.position;

            previous_value = self.run_statement(statement)?;

            self.context.collect_garbage(previous_position.1);

            previous_position = new_position;
        }

        self.context.collect_garbage(previous_position.1);

        Ok(previous_value)
    }

    fn run_statement(&self, node: Node<Statement>) -> Result<Option<Value>, VmError> {
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
                    let value = if let Some(value) = self.run_statement(*right)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: right_position,
                        });
                    };

                    self.context.set_value(identifier, value);

                    return Ok(None);
                }

                if let BinaryOperator::AddAssign = operator.inner {
                    let (identifier, left_value) =
                        if let Statement::Identifier(identifier) = left.inner {
                            let value = self.context.get_value(&identifier).ok_or_else(|| {
                                VmError::UndefinedVariable {
                                    identifier: Node::new(
                                        Statement::Identifier(identifier.clone()),
                                        left.position,
                                    ),
                                }
                            })?;

                            (identifier, value)
                        } else {
                            return Err(VmError::ExpectedIdentifier {
                                position: left.position,
                            });
                        };
                    let right_value = if let Some(value) = self.run_statement(*right)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: right_position,
                        });
                    };
                    let new_value = left_value.add(&right_value).map_err(|value_error| {
                        VmError::ValueError {
                            error: value_error,
                            position: right_position,
                        }
                    })?;

                    self.context.set_value(identifier, new_value);

                    return Ok(None);
                }

                let left_position = left.position;
                let left_value = if let Some(value) = self.run_statement(*left)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: left_position,
                    });
                };
                let right_value = if let Some(value) = self.run_statement(*right)? {
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
                    previous_value = self.run_statement(statement)?;
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
                        let value = if let Some(value) = self.run_statement(node)? {
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
                let function_value = if let Some(value) = self.run_statement(*function_node)? {
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
                        let value = if let Some(value) = self.run_statement(node)? {
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

                Ok(function
                    .clone()
                    .call(None, value_parameters, &self.context)?)
            }
            Statement::Identifier(identifier) => {
                let value_option = self.context.get_value(&identifier);

                if let Some(value) = value_option {
                    Ok(Some(value.clone()))
                } else {
                    Err(VmError::UndefinedVariable {
                        identifier: Node::new(Statement::Identifier(identifier), node.position),
                    })
                }
            }
            Statement::If { condition, body } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_statement(*condition)? {
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
                    self.run_statement(*body)?;
                }

                Ok(None)
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                let condition_position = condition.position;
                let condition_value = if let Some(value) = self.run_statement(*condition)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_statement(*if_body)
                    } else {
                        self.run_statement(*else_body)
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
                let condition_value = if let Some(value) = self.run_statement(*condition)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_statement(*if_body)
                    } else {
                        for (condition, body) in else_ifs {
                            let condition_position = condition.position;
                            let condition_value =
                                if let Some(value) = self.run_statement(condition)? {
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
                                self.run_statement(body)?;
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
                let condition_value = if let Some(value) = self.run_statement(*condition)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: condition_position,
                    });
                };

                if let Some(condition) = condition_value.as_boolean() {
                    if condition {
                        self.run_statement(*if_body)
                    } else {
                        for (condition, body) in else_ifs {
                            let condition_position = condition.position;
                            let condition_value =
                                if let Some(value) = self.run_statement(condition)? {
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
                                return self.run_statement(body);
                            }
                        }

                        self.run_statement(*else_body)
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
                        if let Some(value) = self.run_statement(node)? {
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
                    let value = if let Some(value) = self.run_statement(value_node)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue { position });
                    };

                    values.insert(identifier, value);
                }

                Ok(Some(Value::map(values)))
            }
            Statement::Nil(node) => {
                let _return = self.run_statement(*node)?;

                Ok(None)
            }
            Statement::PropertyAccess(left, right) => {
                let left_span = left.position;
                let left_value = if let Some(value) = self.run_statement(*left)? {
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

                if let (Some(map), Statement::Identifier(identifier)) =
                    (left_value.as_map(), &right.inner)
                {
                    let value = map.get(identifier).cloned();

                    return Ok(value);
                }

                Err(VmError::ExpectedIdentifierOrInteger {
                    position: right_span,
                })
            }
            Statement::UnaryOperation { operator, operand } => {
                let position = operand.position;
                let value = if let Some(value) = self.run_statement(*operand)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue { position });
                };

                match operator.inner {
                    UnaryOperator::Negate => {
                        if let Some(value) = value.as_integer() {
                            Ok(Some(Value::integer(-value)))
                        } else if let Some(value) = value.as_float() {
                            Ok(Some(Value::float(-value)))
                        } else {
                            Err(VmError::ExpectedNumber { position })
                        }
                    }
                    UnaryOperator::Not => {
                        if let Some(value) = value.as_boolean() {
                            Ok(Some(Value::boolean(!value)))
                        } else {
                            Err(VmError::ExpectedBoolean { position })
                        }
                    }
                }
            }
            Statement::While { condition, body } => {
                let mut return_value = None;

                let condition_position = condition.position;

                while let Some(condition_value) = self.run_statement(*condition.clone())? {
                    if let ValueInner::Boolean(condition_value) = condition_value.inner().as_ref() {
                        if !condition_value {
                            break;
                        }
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            position: condition_position,
                        });
                    }

                    return_value = self.run_statement(*body.clone())?;

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
    ExpectedNumber {
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
    UndefinedProperty {
        value: Value,
        value_position: Span,
        property: Identifier,
        property_position: Span,
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
            Self::ExpectedNumber { position } => *position,
            Self::ExpectedValue { position } => *position,
            Self::UndefinedVariable { identifier } => identifier.position,
            Self::UndefinedProperty {
                property_position, ..
            } => *property_position,
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
            Self::ExpectedNumber { position } => {
                write!(
                    f,
                    "Expected an integer or float at position: {:?}",
                    position
                )
            }
            Self::ExpectedValue { position } => {
                write!(f, "Expected a value at position: {:?}", position)
            }
            Self::UndefinedVariable { identifier } => {
                write!(f, "Undefined identifier: {}", identifier)
            }
            Self::UndefinedProperty {
                value, property, ..
            } => {
                write!(f, "Value {} does not have the property {}", value, property)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range() {
        let input = "1..5";

        assert_eq!(run(input), Ok(Some(Value::range(1..5))));
    }

    #[test]
    fn negate_expression() {
        let input = "x = -42; -x";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn not_expression() {
        let input = "!(1 == 2 || 3 == 4 || 5 == 6)";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn list_index() {
        let input = "[1, 42, 3].1";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn map_property_access() {
        let input = "{ a = 42 }.a";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn built_in_function_dot_notation() {
        let input = "42.to_string()";

        assert_eq!(run(input), Ok(Some(Value::string("42"))));
    }

    #[test]
    fn to_string() {
        let input = "to_string(42)";

        assert_eq!(run(input), Ok(Some(Value::string("42".to_string()))));
    }

    #[test]
    fn r#if() {
        let input = "if true { 1 }";

        assert_eq!(run(input), Ok(None));
    }

    #[test]
    fn if_else() {
        let input = "if false { 1 } else { 2 }";

        assert_eq!(run(input), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn if_else_if() {
        let input = "if false { 1 } else if true { 2 }";

        assert_eq!(run(input), Ok(None));
    }

    #[test]
    fn if_else_if_else() {
        let input = "if false { 1 } else if false { 2 } else { 3 }";

        assert_eq!(run(input), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn while_loop() {
        let input = "x = 0; while x < 5 { x += 1; } x";

        assert_eq!(run(input), Ok(Some(Value::integer(5))));
    }

    #[test]
    fn add_assign() {
        let input = "x = 1; x += 1; x";

        assert_eq!(run(input), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn map_equal() {
        let input = "{ y = 'foo' } == { y = 'foo' }";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn integer_equal() {
        let input = "42 == 42";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(run(input), Ok(Some(Value::integer(0))));
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(run(input), Ok(Some(Value::integer(21))));
    }

    #[test]
    fn less_than() {
        let input = "2 < 3";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn less_than_or_equal() {
        let input = "42 <= 42";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn greater_than() {
        let input = "2 > 3";

        assert_eq!(run(input), Ok(Some(Value::boolean(false))));
    }

    #[test]
    fn greater_than_or_equal() {
        let input = "42 >= 42";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn integer_saturating_add() {
        let input = "9223372036854775807 + 1";

        assert_eq!(run(input), Ok(Some(Value::integer(i64::MAX))));
    }

    #[test]
    fn integer_saturating_sub() {
        let input = "-9223372036854775808 - 1";

        assert_eq!(run(input), Ok(Some(Value::integer(i64::MIN))));
    }

    #[test]
    fn multiply() {
        let input = "2 * 3";

        assert_eq!(run(input), Ok(Some(Value::integer(6))));
    }

    #[test]
    fn boolean() {
        let input = "true";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn is_even() {
        let input = "is_even(42)";

        assert_eq!(run(input), Ok(Some(Value::boolean(true))));
    }

    #[test]
    fn is_odd() {
        let input = "is_odd(42)";

        assert_eq!(run(input), Ok(Some(Value::boolean(false))));
    }

    #[test]
    fn length() {
        let input = "length([1, 2, 3])";

        assert_eq!(run(input), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn list_access() {
        let input = "[1, 2, 3].1";

        assert_eq!(run(input), Ok(Some(Value::integer(2))));
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(run(input), Ok(Some(Value::integer(3))));
    }

    #[test]
    fn add_multiple() {
        let input = "1 + 2 + 3";

        assert_eq!(run(input), Ok(Some(Value::integer(6))));
    }
}
