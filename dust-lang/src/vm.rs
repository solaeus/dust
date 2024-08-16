//! Virtual machine for running the abstract syntax tree.
//!
//! This module provides three running option:
//! - `run` convenience function that takes a source code string and runs it
//! - `run_with_context` convenience function that takes a source code string and a context
//! - `Vm` struct that can be used to run an abstract syntax tree
use std::{
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex},
};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    abstract_tree::{
        AbstractSyntaxTree, BlockExpression, CallExpression, ElseExpression, FieldAccessExpression,
        IfExpression, ListExpression, ListIndexExpression, LiteralExpression, LoopExpression, Node,
        OperatorExpression, Statement,
    },
    parse, Analyzer, BuiltInFunctionError, Context, DustError, Expression, Identifier, ParseError,
    Span, Type, Value, ValueError,
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
        let position = statement.position();
        let result = match statement {
            Statement::Expression(expression) => self
                .run_expression(expression)
                .map(|evaluation| evaluation.value()),
            Statement::ExpressionNullified(expression) => {
                self.run_expression(expression.inner)?;

                Ok(None)
            }
            Statement::Let(_) => todo!(),
            Statement::StructDefinition(_) => todo!(),
        };

        result.map_err(|error| VmError::Trace {
            error: Box::new(error),
            position,
        })
    }

    fn run_expression(&self, expression: Expression) -> Result<Evaluation, VmError> {
        let position = expression.position();
        let evaluation_result = match expression {
            Expression::Block(Node { inner, .. }) => self.run_block(*inner),
            Expression::Call(call) => self.run_call(*call.inner),
            Expression::FieldAccess(field_access) => self.run_field_access(*field_access.inner),
            Expression::Grouped(expression) => self.run_expression(*expression.inner),
            Expression::Identifier(identifier) => self.run_identifier(identifier.inner),
            Expression::If(if_expression) => self.run_if(*if_expression.inner),
            Expression::List(list_expression) => self.run_list(*list_expression.inner),
            Expression::ListIndex(list_index) => self.run_list_index(*list_index.inner),
            Expression::Literal(literal) => self.run_literal(*literal.inner),
            Expression::Loop(loop_expression) => self.run_loop(*loop_expression.inner),
            Expression::Operator(_) => todo!(),
            Expression::Range(_) => todo!(),
            Expression::Struct(_) => todo!(),
            Expression::TupleAccess(_) => todo!(),
        };

        evaluation_result.map_err(|error| VmError::Trace {
            error: Box::new(error),
            position,
        })
    }

    fn run_operator(&self, operator: OperatorExpression) -> Result<Evaluation, VmError> {
        match operator {
            OperatorExpression::Assignment { assignee, value } => todo!(),
            OperatorExpression::Comparison {
                left,
                operator,
                right,
            } => todo!(),
            OperatorExpression::CompoundAssignment {
                assignee,
                operator,
                modifier,
            } => todo!(),
            OperatorExpression::ErrorPropagation(_) => todo!(),
            OperatorExpression::Negation(_) => todo!(),
            OperatorExpression::Not(_) => todo!(),
            OperatorExpression::Math {
                left,
                operator,
                right,
            } => todo!(),
            OperatorExpression::Logic {
                left,
                operator,
                right,
            } => todo!(),
        }
    }

    fn run_loop(&self, loop_expression: LoopExpression) -> Result<Evaluation, VmError> {
        match loop_expression {
            LoopExpression::Infinite { block } => loop {
                self.run_expression(Expression::block(block.inner.clone(), block.position))?;
            },
            LoopExpression::While { condition, block } => todo!(),
            LoopExpression::For {
                identifier,
                iterator,
                block,
            } => todo!(),
        }
    }

    fn run_literal(&self, literal: LiteralExpression) -> Result<Evaluation, VmError> {
        let value = match literal {
            LiteralExpression::Boolean(boolean) => Value::boolean(boolean),
            LiteralExpression::Float(float) => Value::float(float),
            LiteralExpression::Integer(integer) => Value::integer(integer),
            LiteralExpression::String(string) => Value::string(string),
            LiteralExpression::Value(value) => value,
        };

        Ok(Evaluation::Return(Some(value)))
    }

    fn run_list_index(&self, list_index: ListIndexExpression) -> Result<Evaluation, VmError> {
        let ListIndexExpression { list, index } = list_index;

        let list_position = list.position();
        let list_value = self.run_expression(list)?.expect_value(list_position)?;

        let index_position = index.position();
        let index_value = self.run_expression(index)?.expect_value(index_position)?;

        let index = if let Some(index) = index_value.as_integer() {
            index as usize
        } else {
            return Err(VmError::ExpectedInteger {
                position: index_position,
            });
        };

        let value_option = list_value.get_index(index);

        Ok(Evaluation::Return(value_option))
    }

    fn run_call(&self, call_expression: CallExpression) -> Result<Evaluation, VmError> {
        let CallExpression { invoker, arguments } = call_expression;

        let invoker_position = invoker.position();
        let invoker_value = if let Some(value) = self.run_expression(invoker)?.value() {
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

            if let Some(value) = self.run_expression(argument)?.value() {
                value_arguments.push(value);
            } else {
                return Err(VmError::ExpectedValue { position });
            }
        }

        let context = Context::new();

        function
            .call(None, Some(value_arguments), &context)
            .map(|value_option| Evaluation::Return(value_option))
    }

    fn run_field_access(&self, field_access: FieldAccessExpression) -> Result<Evaluation, VmError> {
        let FieldAccessExpression { container, field } = field_access;

        let container_position = container.position();
        let container_value = if let Some(value) = self.run_expression(container)?.value() {
            value
        } else {
            return Err(VmError::ExpectedValue {
                position: container_position,
            });
        };

        Ok(Evaluation::Return(container_value.get_field(&field.inner)))
    }

    fn run_identifier(&self, identifier: Identifier) -> Result<Evaluation, VmError> {
        let value_option = self.context.get_value(&identifier);

        if let Some(value) = value_option {
            Ok(Evaluation::Return(Some(value)))
        } else {
            Err(VmError::UndefinedVariable { identifier })
        }
    }

    fn run_list(&self, list_expression: ListExpression) -> Result<Evaluation, VmError> {
        match list_expression {
            ListExpression::AutoFill {
                repeat_operand,
                length_operand,
            } => {
                let position = length_operand.position();
                let length = self
                    .run_expression(length_operand)?
                    .expect_value(position)?
                    .as_integer()
                    .ok_or(VmError::ExpectedInteger { position })?;

                let position = repeat_operand.position();
                let value = self
                    .run_expression(repeat_operand)?
                    .expect_value(position)?;

                Ok(Evaluation::Return(Some(Value::list(vec![
                    value;
                    length as usize
                ]))))
            }
            ListExpression::Ordered(expressions) => {
                let mut values = Vec::new();

                for expression in expressions {
                    let position = expression.position();
                    let value = self.run_expression(expression)?.expect_value(position)?;

                    values.push(value);
                }

                Ok(Evaluation::Return(Some(Value::list(values))))
            }
        }
    }

    fn run_block(&self, block: BlockExpression) -> Result<Evaluation, VmError> {
        match block {
            BlockExpression::Async(statements) => {
                let expected_return = statements.last().unwrap().expected_type();

                let final_result = Arc::new(Mutex::new(None));

                let error_option = statements
                    .into_par_iter()
                    .enumerate()
                    .find_map_any(|statement| self.run_statement(statement).err());

                if let Some(error) = error_option {
                    Err(error)
                } else {
                    Ok(Evaluation::Return(None))
                }
            }
            BlockExpression::Sync(statements) => {
                let mut previous_value = None;

                for statement in statements {
                    let position = statement.position();

                    previous_value = self.run_statement(statement)?;

                    self.context.collect_garbage(position.1);
                }

                Ok(Evaluation::Return(previous_value))
            }
        }
    }

    fn run_if(&self, if_expression: IfExpression) -> Result<Evaluation, VmError> {
        match if_expression {
            IfExpression::If {
                condition,
                if_block,
            } => {
                let position = condition.position();
                let boolean = self
                    .run_expression(condition)?
                    .expect_value(position)?
                    .as_boolean()
                    .ok_or(VmError::ExpectedBoolean { position })?;

                if boolean {
                    self.run_expression(Expression::block(if_block.inner, if_block.position))?;
                }

                Ok(Evaluation::Return(None))
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                let position = condition.position();
                let boolean = self
                    .run_expression(condition)?
                    .expect_value(position)?
                    .as_boolean()
                    .ok_or(VmError::ExpectedBoolean { position })?;

                if boolean {
                    self.run_expression(Expression::block(if_block.inner, if_block.position))?;
                }

                match r#else {
                    ElseExpression::If(if_expression) => {
                        self.run_expression(Expression::If(if_expression))
                    }
                    ElseExpression::Block(block) => {
                        self.run_expression(Expression::block(block.inner, block.position))
                    }
                }
            }
        }
    }
}

enum Evaluation {
    Break,
    Return(Option<Value>),
}

impl Evaluation {
    pub fn value(self) -> Option<Value> {
        match self {
            Evaluation::Break => None,
            Evaluation::Return(value_option) => value_option,
        }
    }

    pub fn expect_value(self, position: Span) -> Result<Value, VmError> {
        if let Evaluation::Return(Some(value)) = self {
            Ok(value)
        } else {
            Err(VmError::ExpectedValue { position })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    ParseError(ParseError),
    Trace {
        error: Box<VmError>,
        position: Span,
    },
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
        identifier: Identifier,
    },
    UndefinedProperty {
        value: Value,
        value_position: Span,
        property: Identifier,
        property_position: Span,
    },
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
            Self::Trace { error, position } => {
                write!(
                    f,
                    "Error during execution at position: {:?}\n{}",
                    position, error
                )
            }
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
