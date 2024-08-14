//! Tools for analyzing an abstract syntax tree and catching errors before running the virtual
//! machine.
//!
//! This module provides two anlysis options:
//! - `analyze` convenience function, which takes a string input
//! - `Analyzer` struct, which borrows an abstract syntax tree and a context
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::{BinaryOperator, UnaryOperator},
    parse, AbstractSyntaxTree, Context, DustError, Identifier, Node, Span, Statement,
    StructDefinition, StructType, Type,
};

/// Analyzes the abstract syntax tree for errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let result = analyze(input);
///
/// assert!(result.is_err());
/// ```
pub fn analyze(source: &str) -> Result<(), DustError> {
    let abstract_tree = parse(source)?;
    let context = Context::new();
    let mut analyzer = Analyzer::new(&abstract_tree, &context);

    analyzer
        .analyze()
        .map_err(|analyzer_error| DustError::AnalyzerError {
            analyzer_error,
            source,
        })
}

/// Static analyzer that checks for potential runtime errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let abstract_tree = parse(input).unwrap();
/// let mut context = Context::new();
/// let mut analyzer = Analyzer::new(&abstract_tree, &mut context);
/// let result = analyzer.analyze();
///
/// assert!(result.is_err());
pub struct Analyzer<'a> {
    abstract_tree: &'a AbstractSyntaxTree,
    context: &'a Context,
}

impl<'a> Analyzer<'a> {
    pub fn new(abstract_tree: &'a AbstractSyntaxTree, context: &'a Context) -> Self {
        Self {
            abstract_tree,
            context,
        }
    }

    pub fn analyze(&mut self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree.nodes {
            self.analyze_statement(node)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, node: &Node<Statement>) -> Result<(), AnalyzerError> {
        match &node.inner {
            Statement::Assignment {
                identifier, value, ..
            } => {
                self.analyze_statement(value)?;

                let value_type = value.inner.expected_type(self.context);

                if let Some(r#type) = value_type {
                    self.context
                        .set_type(identifier.inner.clone(), r#type, identifier.position);
                } else {
                    return Err(AnalyzerError::ExpectedValue {
                        actual: value.as_ref().clone(),
                    });
                }
            }
            Statement::AsyncBlock(statements) => {
                for statement in statements {
                    self.analyze_statement(statement)?;
                }
            }
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                if let BinaryOperator::FieldAccess = operator.inner {
                    self.analyze_statement(left)?;

                    if let Statement::Identifier(_) = right.inner {
                        // Do not expect a value for property accessors
                    } else {
                        self.analyze_statement(right)?;
                    }

                    let left_type = left.inner.expected_type(self.context);
                    let right_type = right.inner.expected_type(self.context);

                    if let Some(Type::Map { .. }) = left_type {
                        if let Some(Type::String) = right_type {
                            // Allow indexing maps with strings
                        } else if let Statement::Identifier(_) = right.inner {
                            // Allow indexing maps with identifiers
                        } else {
                            return Err(AnalyzerError::ExpectedIdentifierOrString {
                                actual: right.as_ref().clone(),
                            });
                        }
                    } else {
                        return Err(AnalyzerError::ExpectedMap {
                            actual: left.as_ref().clone(),
                        });
                    }

                    // If the accessor is an identifier, check if it is a valid field
                    if let Statement::Identifier(identifier) = &right.inner {
                        if let Some(Type::Map(fields)) = &left_type {
                            if !fields.contains_key(identifier) {
                                return Err(AnalyzerError::UndefinedField {
                                    identifier: right.as_ref().clone(),
                                    statement: left.as_ref().clone(),
                                });
                            }
                        }
                    }
                    // If the accessor is a constant, check if it is a valid field
                    if let Statement::Constant(value) = &right.inner {
                        if let Some(field_name) = value.as_string() {
                            if let Some(Type::Map(fields)) = left_type {
                                if !fields.contains_key(&Identifier::new(field_name)) {
                                    return Err(AnalyzerError::UndefinedField {
                                        identifier: right.as_ref().clone(),
                                        statement: left.as_ref().clone(),
                                    });
                                }
                            }
                        }
                    }

                    return Ok(());
                }

                if let BinaryOperator::ListIndex = operator.inner {
                    self.analyze_statement(left)?;
                    self.analyze_statement(right)?;

                    if let Some(Type::List { length, .. }) = left.inner.expected_type(self.context)
                    {
                        let index_type = right.inner.expected_type(self.context);

                        if let Some(Type::Integer | Type::Range) = index_type {
                            // List and index are valid
                        } else {
                            return Err(AnalyzerError::ExpectedIntegerOrRange {
                                actual: right.as_ref().clone(),
                            });
                        }

                        // If the index is a constant, check if it is out of bounds
                        if let Statement::Constant(value) = &right.inner {
                            if let Some(index_value) = value.as_integer() {
                                let index_value = index_value as usize;

                                if index_value >= length {
                                    return Err(AnalyzerError::IndexOutOfBounds {
                                        list: left.as_ref().clone(),
                                        index: right.as_ref().clone(),
                                        index_value,
                                        length,
                                    });
                                }
                            }
                        }
                    } else {
                        return Err(AnalyzerError::ExpectedList {
                            actual: left.as_ref().clone(),
                        });
                    }

                    return Ok(());
                }

                self.analyze_statement(left)?;
                self.analyze_statement(right)?;

                let left_type = left.inner.expected_type(self.context);
                let right_type = right.inner.expected_type(self.context);

                if let BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Greater
                | BinaryOperator::GreaterOrEqual
                | BinaryOperator::Less
                | BinaryOperator::LessOrEqual = operator.inner
                {
                    if let Some(expected_type) = left_type {
                        if let Some(actual_type) = right_type {
                            expected_type.check(&actual_type).map_err(|conflict| {
                                AnalyzerError::TypeConflict {
                                    actual_statement: right.as_ref().clone(),
                                    actual_type: conflict.actual,
                                    expected: conflict.expected,
                                }
                            })?;
                        } else {
                            return Err(AnalyzerError::ExpectedValue {
                                actual: right.as_ref().clone(),
                            });
                        }
                    } else {
                        return Err(AnalyzerError::ExpectedValue {
                            actual: left.as_ref().clone(),
                        });
                    }
                }
            }
            Statement::Block(statements) => {
                for statement in statements {
                    self.analyze_statement(statement)?;
                }
            }
            Statement::BuiltInFunctionCall {
                function,
                value_arguments,
                ..
            } => {
                let value_parameters = function.value_parameters();

                if let Some(arguments) = value_arguments {
                    for argument in arguments {
                        self.analyze_statement(argument)?;
                    }

                    if arguments.len() != value_parameters.len() {
                        return Err(AnalyzerError::ExpectedValueArgumentCount {
                            expected: value_parameters.len(),
                            actual: arguments.len(),
                            position: node.position,
                        });
                    }

                    for ((_identifier, parameter_type), argument) in
                        value_parameters.iter().zip(arguments)
                    {
                        let argument_type_option = argument.inner.expected_type(self.context);

                        if let Some(argument_type) = argument_type_option {
                            parameter_type.check(&argument_type).map_err(|conflict| {
                                AnalyzerError::TypeConflict {
                                    actual_statement: argument.clone(),
                                    actual_type: conflict.actual,
                                    expected: parameter_type.clone(),
                                }
                            })?;
                        } else {
                            return Err(AnalyzerError::ExpectedValue {
                                actual: argument.clone(),
                            });
                        }
                    }

                    if arguments.is_empty() && !value_parameters.is_empty() {
                        return Err(AnalyzerError::ExpectedValueArgumentCount {
                            expected: value_parameters.len(),
                            actual: 0,
                            position: node.position,
                        });
                    }
                } else if !value_parameters.is_empty() {
                    return Err(AnalyzerError::ExpectedValueArgumentCount {
                        expected: value_parameters.len(),
                        actual: 0,
                        position: node.position,
                    });
                }
            }
            Statement::Constant(_) => {}
            Statement::FieldsStructInstantiation {
                name,
                fields: field_arguments,
            } => {
                let expected_type = self.context.get_type(&name.inner);

                if let Some(Type::Struct(StructType::Fields { fields, .. })) = expected_type {
                    for ((_, expected_type), (_, argument)) in
                        fields.iter().zip(field_arguments.iter())
                    {
                        let actual_type = argument.inner.expected_type(self.context);

                        if let Some(actual_type) = actual_type {
                            expected_type.check(&actual_type).map_err(|conflict| {
                                AnalyzerError::TypeConflict {
                                    actual_statement: argument.clone(),
                                    actual_type: conflict.actual,
                                    expected: conflict.expected,
                                }
                            })?;
                        } else {
                            return Err(AnalyzerError::ExpectedValue {
                                actual: argument.clone(),
                            });
                        }
                    }
                }
            }
            Statement::Invokation {
                invokee,
                value_arguments,
                ..
            } => {
                self.analyze_statement(invokee)?;

                let invokee_type = invokee.inner.expected_type(self.context);

                if let Some(arguments) = value_arguments {
                    for argument in arguments {
                        self.analyze_statement(argument)?;
                    }

                    if let Some(Type::Struct(struct_type)) = invokee_type {
                        match struct_type {
                            StructType::Unit { .. } => todo!(),
                            StructType::Tuple { fields, .. } => {
                                for (expected_type, argument) in fields.iter().zip(arguments.iter())
                                {
                                    let actual_type = argument.inner.expected_type(self.context);

                                    if let Some(actual_type) = actual_type {
                                        expected_type.check(&actual_type).map_err(|conflict| {
                                            AnalyzerError::TypeConflict {
                                                actual_statement: argument.clone(),
                                                actual_type: conflict.actual,
                                                expected: expected_type.clone(),
                                            }
                                        })?;
                                    } else {
                                        return Err(AnalyzerError::ExpectedValue {
                                            actual: argument.clone(),
                                        });
                                    }
                                }
                            }
                            StructType::Fields { .. } => todo!(),
                        }
                    }
                }
            }
            Statement::Identifier(identifier) => {
                let exists = self.context.update_last_position(identifier, node.position);

                if !exists {
                    return Err(AnalyzerError::UndefinedVariable {
                        identifier: node.clone(),
                    });
                }
            }
            Statement::If { condition, body } => {
                self.analyze_statement(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                    });
                }

                self.analyze_statement(body)?;
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                self.analyze_statement(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                    });
                }

                self.analyze_statement(if_body)?;
                self.analyze_statement(else_body)?;
            }
            Statement::IfElseIf {
                condition,
                if_body,
                else_ifs,
            } => {
                self.analyze_statement(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                    });
                }

                self.analyze_statement(if_body)?;

                for (condition, body) in else_ifs {
                    self.analyze_statement(condition)?;

                    if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                        // Condition is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: condition.clone(),
                        });
                    }

                    self.analyze_statement(body)?;
                }
            }
            Statement::IfElseIfElse {
                condition,
                if_body,
                else_ifs,
                else_body,
            } => {
                self.analyze_statement(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                    });
                }

                self.analyze_statement(if_body)?;

                for (condition, body) in else_ifs {
                    self.analyze_statement(condition)?;

                    if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                        // Condition is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: condition.clone(),
                        });
                    }

                    self.analyze_statement(body)?;
                }

                self.analyze_statement(else_body)?;
            }
            Statement::List(statements) => {
                for statement in statements {
                    self.analyze_statement(statement)?;
                }
            }
            Statement::Map(properties) => {
                for (_key, value_node) in properties {
                    self.analyze_statement(value_node)?;
                }
            }
            Statement::MutAssignment { identifier, value } => {
                self.analyze_statement(value)?;

                let value_type = value.inner.expected_type(self.context);

                if let Some(r#type) = value_type {
                    self.context
                        .set_type(identifier.inner.clone(), r#type, identifier.position);
                } else {
                    return Err(AnalyzerError::ExpectedValue {
                        actual: value.as_ref().clone(),
                    });
                }
            }
            Statement::Nil(node) => {
                self.analyze_statement(node)?;
            }
            Statement::StructDefinition(struct_definition) => {
                let (name, r#type) = match struct_definition {
                    StructDefinition::Unit { name } => (
                        name.inner.clone(),
                        Type::Struct(StructType::Unit {
                            name: name.inner.clone(),
                        }),
                    ),
                    StructDefinition::Tuple {
                        name,
                        items: fields,
                    } => (
                        name.inner.clone(),
                        Type::Struct(StructType::Tuple {
                            name: name.inner.clone(),
                            fields: fields
                                .iter()
                                .map(|type_node| type_node.inner.clone())
                                .collect(),
                        }),
                    ),
                    StructDefinition::Fields { name, fields } => (
                        name.inner.clone(),
                        Type::Struct(StructType::Fields {
                            name: name.inner.clone(),
                            fields: fields
                                .iter()
                                .map(|(identifier, r#type)| {
                                    (identifier.inner.clone(), r#type.inner.clone())
                                })
                                .collect(),
                        }),
                    ),
                };

                self.context.set_type(name, r#type, node.position);
            }
            Statement::UnaryOperation { operator, operand } => {
                self.analyze_statement(operand)?;

                if let UnaryOperator::Negate = operator.inner {
                    if let Some(Type::Integer | Type::Float | Type::Number) =
                        operand.inner.expected_type(self.context)
                    {
                        // Operand is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: operand.as_ref().clone(),
                        });
                    }
                }

                if let UnaryOperator::Not = operator.inner {
                    if let Some(Type::Boolean) = operand.inner.expected_type(self.context) {
                        // Operand is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: operand.as_ref().clone(),
                        });
                    }
                }
            }
            Statement::While { condition, body } => {
                self.analyze_statement(condition)?;
                self.analyze_statement(body)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedBoolean {
        actual: Node<Statement>,
    },
    ExpectedIdentifier {
        actual: Node<Statement>,
    },
    ExpectedIdentifierOrString {
        actual: Node<Statement>,
    },
    ExpectedIntegerOrRange {
        actual: Node<Statement>,
    },
    ExpectedList {
        actual: Node<Statement>,
    },
    ExpectedMap {
        actual: Node<Statement>,
    },
    ExpectedValue {
        actual: Node<Statement>,
    },
    ExpectedValueArgumentCount {
        expected: usize,
        actual: usize,
        position: Span,
    },
    IndexOutOfBounds {
        list: Node<Statement>,
        index: Node<Statement>,
        index_value: usize,
        length: usize,
    },
    TypeConflict {
        actual_statement: Node<Statement>,
        actual_type: Type,
        expected: Type,
    },
    UndefinedField {
        identifier: Node<Statement>,
        statement: Node<Statement>,
    },
    UndefinedType {
        identifier: Node<Identifier>,
    },
    UnexpectedIdentifier {
        identifier: Node<Statement>,
    },
    UnexectedString {
        actual: Node<Statement>,
    },
    UndefinedVariable {
        identifier: Node<Statement>,
    },
}

impl AnalyzerError {
    pub fn position(&self) -> Span {
        match self {
            AnalyzerError::ExpectedBoolean { actual, .. } => actual.position,
            AnalyzerError::ExpectedIdentifier { actual, .. } => actual.position,
            AnalyzerError::ExpectedIdentifierOrString { actual } => actual.position,
            AnalyzerError::ExpectedIntegerOrRange { actual, .. } => actual.position,
            AnalyzerError::ExpectedList { actual } => actual.position,
            AnalyzerError::ExpectedMap { actual } => actual.position,
            AnalyzerError::ExpectedValue { actual } => actual.position,
            AnalyzerError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalyzerError::IndexOutOfBounds { list, index, .. } => {
                (list.position.0, index.position.1)
            }
            AnalyzerError::TypeConflict {
                actual_statement, ..
            } => actual_statement.position,
            AnalyzerError::UndefinedField { identifier, .. } => identifier.position,
            AnalyzerError::UndefinedType { identifier } => identifier.position,
            AnalyzerError::UndefinedVariable { identifier } => identifier.position,
            AnalyzerError::UnexpectedIdentifier { identifier } => identifier.position,
            AnalyzerError::UnexectedString { actual } => actual.position,
        }
    }
}

impl Error for AnalyzerError {}

impl Display for AnalyzerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AnalyzerError::ExpectedBoolean { actual, .. } => {
                write!(f, "Expected boolean, found {}", actual)
            }
            AnalyzerError::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {}", actual)
            }
            AnalyzerError::ExpectedIdentifierOrString { actual } => {
                write!(f, "Expected identifier or string, found {}", actual)
            }
            AnalyzerError::ExpectedIntegerOrRange { actual, .. } => {
                write!(f, "Expected integer or range, found {}", actual)
            }
            AnalyzerError::ExpectedList { actual } => write!(f, "Expected list, found {}", actual),
            AnalyzerError::ExpectedMap { actual } => write!(f, "Expected map, found {}", actual),
            AnalyzerError::ExpectedValue { actual, .. } => {
                write!(f, "Expected value, found {}", actual)
            }
            AnalyzerError::ExpectedValueArgumentCount {
                expected, actual, ..
            } => write!(f, "Expected {} value arguments, found {}", expected, actual),
            AnalyzerError::IndexOutOfBounds {
                list,
                index_value,
                length,
                ..
            } => write!(
                f,
                "Index {} out of bounds for list {} with length {}",
                index_value, list, length
            ),
            AnalyzerError::TypeConflict {
                actual_statement,
                actual_type,
                expected,
            } => {
                write!(
                    f,
                    "Expected type {}, found {}, which has type {}",
                    expected, actual_statement, actual_type
                )
            }
            AnalyzerError::UndefinedField {
                identifier,
                statement: map,
            } => {
                write!(f, "Undefined field {} in map {}", identifier, map)
            }
            AnalyzerError::UndefinedType { identifier } => {
                write!(f, "Undefined type {}", identifier)
            }
            AnalyzerError::UndefinedVariable { identifier } => {
                write!(f, "Undefined variable {}", identifier)
            }
            AnalyzerError::UnexpectedIdentifier { identifier, .. } => {
                write!(f, "Unexpected identifier {}", identifier)
            }
            AnalyzerError::UnexectedString { actual, .. } => {
                write!(f, "Unexpected string {}", actual)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Identifier, Value};

    use super::*;

    #[test]
    fn tuple_struct_with_wrong_field_types() {
        let source = "
            struct Foo(int, float)
            Foo(1, 2)
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::integer(2)), (55, 56)),
                    actual_type: Type::Integer,
                    expected: Type::Float
                },
                source
            })
        );
    }

    #[test]
    fn constant_list_index_out_of_bounds() {
        let source = "[1, 2, 3][3]";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::IndexOutOfBounds {
                    list: Node::new(
                        Statement::List(vec![
                            Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                            Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                            Node::new(Statement::Constant(Value::integer(3)), (7, 8)),
                        ]),
                        (0, 9)
                    ),
                    index: Node::new(Statement::Constant(Value::integer(3)), (10, 11)),
                    index_value: 3,
                    length: 3
                },
                source
            })
        );
    }

    #[test]
    fn nonexistant_field_identifier() {
        let source = "{ x = 1 }.y";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedField {
                    identifier: Node::new(Statement::Identifier(Identifier::new("y")), (10, 11)),
                    statement: Node::new(
                        Statement::Map(vec![(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Node::new(Statement::Constant(Value::integer(1)), (6, 7))
                        )]),
                        (0, 9)
                    )
                },
                source
            })
        );
    }

    #[test]
    fn nonexistant_field_string() {
        let source = "{ x = 1 }.'y'";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedField {
                    identifier: Node::new(Statement::Constant(Value::string("y")), (10, 13)),
                    statement: Node::new(
                        Statement::Map(vec![(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Node::new(Statement::Constant(Value::integer(1)), (6, 7))
                        )]),
                        (0, 9)
                    )
                },
                source
            })
        );
    }

    #[test]
    fn malformed_list_index() {
        let source = "[1, 2, 3]['foo']";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedIntegerOrRange {
                    actual: Node::new(Statement::Constant(Value::string("foo")), (10, 15)),
                },
                source
            })
        );
    }

    #[test]
    fn malformed_field_access() {
        let source = "{ x = 1 }.0";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedIdentifierOrString {
                    actual: Node::new(Statement::Constant(Value::integer(0)), (10, 11)),
                },
                source
            })
        );
    }

    #[test]
    fn length_no_arguments() {
        let source = "length()";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedValueArgumentCount {
                    expected: 1,
                    actual: 0,
                    position: (0, 6)
                },
                source
            })
        );
    }

    #[test]
    fn float_plus_integer() {
        let source = "42.0 + 2";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::integer(2)), (7, 8)),
                    actual_type: Type::Integer,
                    expected: Type::Float,
                },
                source
            })
        )
    }

    #[test]
    fn integer_plus_boolean() {
        let source = "42 + true";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::boolean(true)), (5, 9)),
                    actual_type: Type::Boolean,
                    expected: Type::Integer,
                },
                source
            })
        )
    }

    #[test]
    fn is_even_expects_number() {
        let source = "is_even('hello')";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Constant(Value::string("hello")),
                        (8, 15)
                    ),
                    actual_type: Type::String,
                    expected: Type::Number,
                },
                source
            })
        );
    }

    #[test]
    fn is_odd_expects_number() {
        let source = "is_odd('hello')";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Constant(Value::string("hello")),
                        (7, 14)
                    ),
                    actual_type: Type::String,
                    expected: Type::Number,
                },
                source
            })
        );
    }

    #[test]
    fn undefined_variable() {
        let source = "foo";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedVariable {
                    identifier: Node::new(Statement::Identifier(Identifier::new("foo")), (0, 3)),
                },
                source
            })
        );
    }
}
