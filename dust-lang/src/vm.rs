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

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    abstract_tree::{AbstractSyntaxTree, Block, CallExpression, FieldAccess, Node, Statement},
    parse, Analyzer, BuiltInFunctionError, Context, DustError, Expression, Identifier, ParseError,
    Span, Struct, StructType, Type, Value, ValueError,
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
/// **Warning**: Do not run an AbstractSyntaxTree that has not been analyzed *with the same
/// context*. Use the `run` or `run_with_context` functions to make sure the program is analyzed
/// before running it.
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

        while let Some(statement) = self.abstract_tree.statements.pop_front() {
            let new_position = statement.position();

            previous_value = self.run_statement(statement)?;

            self.context.collect_garbage(previous_position.1);

            previous_position = new_position;
        }

        self.context.collect_garbage(previous_position.1);

        Ok(previous_value)
    }

    fn run_statement(&self, statement: Statement) -> Result<Option<Value>, VmError> {
        match statement {
            Statement::Expression(expression) => self.run_expression(expression),
            Statement::ExpressionNullified(expression) => {
                self.run_expression(expression.inner)?;

                Ok(None)
            }
            Statement::Let(_) => todo!(),
            Statement::StructDefinition(_) => todo!(),
        }
    }

    fn run_expression(&self, expression: Expression) -> Result<Option<Value>, VmError> {
        match expression {
            Expression::Block(Node { inner, position }) => match *inner {
                Block::Async(statements) => {
                    let error_option = statements
                        .into_par_iter()
                        .find_map_any(|statement| self.run_statement(statement).err());

                    if let Some(error) = error_option {
                        Err(error)
                    } else {
                        Ok(None)
                    }
                }
                Block::Sync(statements) => {
                    let mut previous_value = None;

                    for statement in statements {
                        let position = statement.position();

                        previous_value = self.run_statement(statement)?;

                        self.context.collect_garbage(position.1);
                    }

                    Ok(previous_value)
                }
            },
            Expression::Call(Node { inner, .. }) => {
                let CallExpression { invoker, arguments } = *inner;

                let invoker_position = invoker.position();
                let invoker_value = if let Some(value) = self.run_expression(invoker)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: invoker_position,
                    });
                };

                let function = if let Some(function) = invoker_value.as_function() {
                    function
                } else {
                    return Err(VmError::ExpectedFunction {
                        actual: invoker_value,
                        position: invoker_position,
                    });
                };

                let mut value_arguments = Vec::new();

                for argument in arguments {
                    let position = argument.position();

                    if let Some(value) = self.run_expression(argument)? {
                        value_arguments.push(value);
                    } else {
                        return Err(VmError::ExpectedValue { position });
                    }
                }

                let context = Context::new();

                function.call(None, Some(value_arguments), &context)
            }
            Expression::FieldAccess(Node { inner, .. }) => {
                let FieldAccess { container, field } = *inner;

                let container_position = container.position();
                let container_value = if let Some(value) = self.run_expression(container)? {
                    value
                } else {
                    return Err(VmError::ExpectedValue {
                        position: container_position,
                    });
                };

                Ok(container_value.get_field(&field.inner))
            }
            Expression::Grouped(_) => todo!(),
            Expression::Identifier(_) => todo!(),
            Expression::If(_) => todo!(),
            Expression::List(_) => todo!(),
            Expression::ListIndex(_) => todo!(),
            Expression::Literal(_) => todo!(),
            Expression::Loop(_) => todo!(),
            Expression::Operator(_) => todo!(),
            Expression::Range(_) => todo!(),
            Expression::Struct(_) => todo!(),
            Expression::TupleAccess(_) => todo!(),
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
    CannotMutate {
        value: Value,
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
            Self::CannotMutate { position, .. } => *position,
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
            Self::CannotMutate { value, .. } => {
                write!(f, "Cannot mutate immutable value {}", value)
            }
            Self::BuiltInFunctionError { error, .. } => {
                write!(f, "{}", error)
            }
            Self::ExpectedBoolean { position } => {
                write!(f, "Expected a boolean at position: {:?}", position)
            }
            Self::ExpectedFunction { actual, position } => {
                write!(
                    f,
                    "Expected a function, but got {} at position: {:?}",
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
    fn mutate_variable() {
        let input = "
            mut x = ''

            x += 'foo'
            x += 'bar'

            x
        ";

        assert_eq!(run(input), Ok(Some(Value::string_mut("foobar"))));
    }

    #[test]
    fn async_block() {
        let input = "mut x = 1; async { x += 1; x -= 1; } x";

        assert!(run(input).unwrap().unwrap().as_integer().is_some());
    }

    #[test]
    fn define_and_instantiate_fields_struct() {
        let input = "struct Foo { bar: int, baz: float } Foo { bar = 42, baz = 4.0 }";

        assert_eq!(
            run(input),
            Ok(Some(Value::r#struct(Struct::Fields {
                name: Identifier::new("Foo"),
                fields: vec![
                    (Identifier::new("bar"), Value::integer(42)),
                    (Identifier::new("baz"), Value::float(4.0))
                ]
            })))
        );
    }

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
        let input = "mut x = 0; while x < 5 { x += 1; } x";

        assert_eq!(run(input), Ok(Some(Value::integer(5))));
    }

    #[test]
    fn subtract_assign() {
        let input = "mut x = 1; x -= 1; x";

        assert_eq!(run(input), Ok(Some(Value::integer(0))));
    }

    #[test]
    fn add_assign() {
        let input = "mut x = 1; x += 1; x";

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
