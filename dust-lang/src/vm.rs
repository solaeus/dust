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
    parse, value::ValueInner, AbstractSyntaxTree, Analyzer, AssignmentOperator, BinaryOperator,
    BuiltInFunctionError, Context, DustError, Identifier, Node, ParseError, Span, Statement,
    Struct, StructDefinition, StructInstantiation, StructType, Type, UnaryOperator, Value,
    ValueError,
};

/// Run the source code and return the result.
///
/// # Example
/// ```
/// # use dust_lang::vm::run;
/// # use dust_lang::value::Value;
/// let result = run("40 + 2");
///
/// assert_eq!(result, Ok(Some(Value::integer(42))));
/// ```
pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let context = Context::new();

    run_with_context(source, context)
}

/// Run the source code with a context and return the result.
///
/// # Example
/// ```
/// # use dust_lang::{Context, Identifier, Value, run_with_context};
/// let context = Context::new();
///
/// context.set_value(Identifier::new("foo"), Value::integer(40));
/// context.update_last_position(&Identifier::new("foo"), (100, 100));
///
/// let result = run_with_context("foo + 2", context);
///
/// assert_eq!(result, Ok(Some(Value::integer(42))));
/// ```
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

/// Dust virtual machine.
///
/// **Warning**: Do not run an AbstractSyntaxTree that has not been analyzed. Use the `run` or
/// `run_with_context` functions to make sure the program is analyzed before running it.
///
/// See the `run_with_context` function for an example of how to use the Analyzer and the VM.
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
            Statement::Assignment {
                identifier,
                operator,
                value,
            } => match operator.inner {
                AssignmentOperator::Assign => {
                    let position = value.position;
                    let value = if let Some(value) = self.run_statement(*value)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue { position });
                    };

                    self.context.set_value(identifier.inner, value);

                    Ok(None)
                }
                AssignmentOperator::AddAssign => {
                    let left_value = if let Some(value) = self.context.get_value(&identifier.inner)
                    {
                        value
                    } else {
                        return Err(VmError::UndefinedVariable { identifier });
                    };
                    let value_position = value.position;
                    let right_value = if let Some(value) = self.run_statement(*value)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: value_position,
                        });
                    };
                    let new_value = left_value.add(&right_value).map_err(|value_error| {
                        VmError::ValueError {
                            error: value_error,
                            position: (identifier.position.0, value_position.1),
                        }
                    })?;

                    self.context.set_value(identifier.inner, new_value);

                    Ok(None)
                }
                AssignmentOperator::SubtractAssign => {
                    todo!()
                }
            },
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let right_position = right.position;

                if let BinaryOperator::FieldAccess = operator.inner {
                    let left_span = left.position;
                    let left_value = if let Some(value) = self.run_statement(*left)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: left_span,
                        });
                    };
                    let right_span = right.position;

                    if let Some(map) = left_value.as_map() {
                        if let Statement::Identifier(identifier) = right.inner {
                            let value = map.get(&identifier).cloned();

                            return Ok(value);
                        }

                        if let Some(value) = self.run_statement(*right)? {
                            if let Some(string) = value.as_string() {
                                let identifier = Identifier::new(string);

                                let value = map.get(&identifier).cloned();

                                return Ok(value);
                            }
                        }

                        return Err(VmError::ExpectedIdentifierOrString {
                            position: right_span,
                        });
                    } else {
                        return Err(VmError::ExpectedMap {
                            position: left_span,
                        });
                    }
                }

                if let BinaryOperator::ListIndex = operator.inner {
                    let list_position = left.position;
                    let list_value = if let Some(value) = self.run_statement(*left)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: list_position,
                        });
                    };
                    let list = if let Some(list) = list_value.as_list() {
                        list
                    } else {
                        return Err(VmError::ExpectedList {
                            position: list_position,
                        });
                    };
                    let index_position = right.position;
                    let index_value = if let Some(value) = self.run_statement(*right)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue {
                            position: index_position,
                        });
                    };

                    if let Some(index) = index_value.as_integer() {
                        return if let Some(value) = list.get(index as usize) {
                            Ok(Some(value.clone()))
                        } else {
                            Ok(None)
                        };
                    }

                    if let Some(range) = index_value.as_range() {
                        let range = range.start as usize..range.end as usize;

                        return if let Some(list) = list.get(range) {
                            Ok(Some(Value::list(list.to_vec())))
                        } else {
                            Ok(None)
                        };
                    }

                    return Err(VmError::ExpectedIntegerOrRange {
                        position: index_position,
                    });
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
            Statement::Invokation {
                invokee,
                type_arguments: _,
                value_arguments: value_parameter_nodes,
            } => {
                let invokee_position = invokee.position;
                let invokee_type = invokee.inner.expected_type(&self.context);

                if let Some(Type::Struct(StructType::Tuple { name, .. })) = invokee_type {
                    let mut fields = Vec::new();

                    if let Some(value_parameter_nodes) = value_parameter_nodes {
                        for statement in value_parameter_nodes {
                            let position = statement.position;
                            let value = if let Some(value) = self.run_statement(statement)? {
                                value
                            } else {
                                return Err(VmError::ExpectedValue { position });
                            };

                            fields.push(value);
                        }
                    }

                    let struct_value = Value::r#struct(Struct::Tuple {
                        name: name.clone(),
                        fields,
                    });

                    return Ok(Some(struct_value));
                }

                let function_value = if let Some(value) = self.run_statement(*invokee)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: invokee_position,
                    });
                };
                let function = if let Some(function) = function_value.as_function() {
                    function
                } else {
                    return Err(VmError::ExpectedFunction {
                        actual: function_value,
                        position: invokee_position,
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
                    return Ok(Some(value.clone()));
                }

                let type_option = self.context.get_type(&identifier);

                println!("{type_option:?}");

                if let Some(Type::Struct(StructType::Unit { name })) = type_option {
                    return Ok(Some(Value::r#struct(Struct::Unit { name })));
                }

                Err(VmError::UndefinedVariable {
                    identifier: Node::new(identifier, node.position),
                })
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
                    let position = value_node.position;
                    let value = if let Some(value) = self.run_statement(value_node)? {
                        value
                    } else {
                        return Err(VmError::ExpectedValue { position });
                    };

                    values.insert(identifier.inner, value);
                }

                Ok(Some(Value::map(values)))
            }
            Statement::Nil(node) => {
                let _return = self.run_statement(*node)?;

                Ok(None)
            }
            Statement::StructDefinition(_) => Ok(None),
            Statement::StructInstantiation(struct_instantiation) => match struct_instantiation {
                StructInstantiation::Tuple { name, fields } => {
                    Ok(Some(Value::r#struct(Struct::Tuple {
                        name: name.inner,
                        fields: fields
                            .into_iter()
                            .map(|node| {
                                let position = node.position;
                                if let Some(value) = self.run_statement(node)? {
                                    Ok(value)
                                } else {
                                    Err(VmError::ExpectedValue { position })
                                }
                            })
                            .collect::<Result<Vec<Value>, VmError>>()?,
                    })))
                }
            },
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
    ExpectedIntegerOrRange {
        position: Span,
    },
    ExpectedIdentifierOrString {
        position: Span,
    },
    ExpectedInteger {
        position: Span,
    },
    ExpectedNumber {
        position: Span,
    },
    ExpectedMap {
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
        identifier: Node<Identifier>,
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
            Self::ExpectedIdentifierOrString { position } => *position,
            Self::ExpectedIntegerOrRange { position } => *position,
            Self::ExpectedInteger { position } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedList { position } => *position,
            Self::ExpectedMap { position } => *position,
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
            Self::ExpectedIdentifierOrString { position } => {
                write!(
                    f,
                    "Expected an identifier or string at position: {:?}",
                    position
                )
            }
            Self::ExpectedIntegerOrRange { position } => {
                write!(
                    f,
                    "Expected an identifier, integer, or range at position: {:?}",
                    position
                )
            }
            Self::ExpectedInteger { position } => {
                write!(f, "Expected an integer at position: {:?}", position)
            }
            Self::ExpectedList { position } => {
                write!(f, "Expected a list at position: {:?}", position)
            }
            Self::ExpectedMap { position } => {
                write!(f, "Expected a map at position: {:?}", position)
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
    use crate::Struct;

    use super::*;

    #[test]
    fn assign_tuple_struct_variable() {
        let input = "
            struct Foo(int)
            x = Foo(42)
            x
        ";

        assert_eq!(
            run(input),
            Ok(Some(Value::r#struct(Struct::Tuple {
                name: Identifier::new("Foo"),
                fields: vec![Value::integer(42)]
            })))
        )
    }

    #[test]
    fn define_and_instantiate_tuple_struct() {
        let input = "struct Foo(int) Foo(42)";

        assert_eq!(
            run(input),
            Ok(Some(Value::r#struct(Struct::Tuple {
                name: Identifier::new("Foo"),
                fields: vec![Value::integer(42)]
            })))
        );
    }

    #[test]
    fn assign_unit_struct_variable() {
        let input = "
            struct Foo
            x = Foo
            x
        ";

        assert_eq!(
            run(input),
            Ok(Some(Value::r#struct(Struct::Unit {
                name: Identifier::new("Foo")
            })))
        )
    }

    #[test]
    fn define_and_instantiate_unit_struct() {
        let input = "struct Foo Foo";

        assert_eq!(
            run(input),
            Ok(Some(Value::r#struct(Struct::Unit {
                name: Identifier::new("Foo")
            })))
        );
    }

    #[test]
    fn list_index_nested() {
        let input = "[[1, 2], [42, 4], [5, 6]][1][0]";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn map_property() {
        let input = "{ x = 42 }.x";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn map_property_nested() {
        let input = "{ x = { y = 42 } }.x.y";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn map_property_access_expression() {
        let input = "{ foobar = 42 }.('foo' + 'bar')";

        assert_eq!(run(input), Ok(Some(Value::integer(42))));
    }

    #[test]
    fn list_index_range() {
        let input = "[1, 2, 3, 4, 5][1..3]";

        assert_eq!(
            run(input),
            Ok(Some(Value::list(vec![
                Value::integer(2),
                Value::integer(3)
            ])))
        );
    }

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
        let input = "[1, 42, 3][1]";

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
