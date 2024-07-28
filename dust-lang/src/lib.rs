/**
The Dust programming language.

Dust is a statically typed, interpreted programming language.

The top-level module contains the `Interpreter` struct, which is used to lex, parse and/or
interpret Dust code. The `interpret` function is a convenience function that creates a new
`Interpreter` and runs the given source code.
*/
pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod identifier;
pub mod lexer;
pub mod parser;
pub mod standard_library;
pub mod value;

use std::{
    collections::{hash_map, HashMap},
    ops::Range,
    sync::{Arc, RwLock},
};

pub use abstract_tree::Type;
pub use value::Value;

use abstract_tree::AbstractTree;
use ariadne::{Color, Fmt, Label, Report, ReportKind};
use context::Context;
use error::{DustError, RuntimeError, TypeConflict, ValidationError};
use lexer::{lex, Token};
use parser::{parse, parser};
use standard_library::core_context;

pub fn interpret(source_id: &str, source: &str) -> Result<Option<Value>, InterpreterError> {
    let interpreter = Interpreter::new();

    interpreter.run(Arc::from(source_id), Arc::from(source))
}

/// Interpreter, lexer and parser for the Dust programming language.
///
/// You must provide the interpreter with an ID for each piece of code you pass to it. These are
/// used to identify the source of errors and to provide more detailed error messages.
#[derive(Clone, Debug)]
pub struct Interpreter {
    contexts: Arc<RwLock<HashMap<Arc<str>, Context>>>,
    sources: Arc<RwLock<HashMap<Arc<str>, Arc<str>>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Lexes the source code and returns a list of tokens.
    pub fn lex<'id>(
        &self,
        source_id: Arc<str>,
        source: &'id Arc<str>,
    ) -> Result<Vec<Token<'id>>, InterpreterError> {
        self.sources
            .write()
            .unwrap()
            .insert(source_id.clone(), source.clone());

        lex(source.as_ref())
            .map(|tokens| tokens.into_iter().map(|(token, _)| token).collect())
            .map_err(|errors| InterpreterError {
                source_id: source_id.clone(),
                errors,
            })
    }

    /// Parses the source code and returns an abstract syntax tree.
    pub fn parse<'id>(
        &self,
        source_id: Arc<str>,
        source: &'id Arc<str>,
    ) -> Result<AbstractTree, InterpreterError> {
        self.sources
            .write()
            .unwrap()
            .insert(source_id.clone(), source.clone());

        parse(&lex(source).map_err(|errors| InterpreterError {
            source_id: source_id.clone(),
            errors,
        })?)
        .map_err(|errors| InterpreterError { source_id, errors })
    }

    /// Runs the source code and returns the result.
    pub fn run(
        &self,
        source_id: Arc<str>,
        source: Arc<str>,
    ) -> Result<Option<Value>, InterpreterError> {
        let mut sources = self.sources.write().unwrap();

        sources.insert(source_id.clone(), source.clone());

        let tokens = lex(source.as_ref()).map_err(|errors| InterpreterError {
            source_id: source_id.clone(),
            errors,
        })?;
        let abstract_tree = parse(&tokens).map_err(|errors| InterpreterError {
            source_id: source_id.clone(),
            errors,
        })?;
        let context = self.get_or_create_context(&source_id);
        let value_option = abstract_tree
            .run(&context, true)
            .map_err(|errors| InterpreterError { source_id, errors })?;

        Ok(value_option)
    }

    pub fn sources(&self) -> hash_map::IntoIter<Arc<str>, Arc<str>> {
        self.sources.read().unwrap().clone().into_iter()
    }

    fn get_or_create_context(&self, source_id: &Arc<str>) -> Context {
        let mut contexts = self.contexts.write().unwrap();

        if let Some(context) = contexts.get(source_id) {
            context.clone()
        } else {
            let context = core_context().clone();

            contexts.insert(source_id.clone(), context.clone());

            context
        }
    }
}

#[derive(Debug, PartialEq)]
/// An error that occurred during the interpretation of a piece of code.
///
/// Each error has a source ID that identifies the piece of code that caused the error, and a list
/// of errors that occurred during the interpretation of that code.
pub struct InterpreterError {
    source_id: Arc<str>,
    errors: Vec<DustError>,
}

impl InterpreterError {
    pub fn new(source_id: Arc<str>, errors: Vec<DustError>) -> Self {
        InterpreterError { source_id, errors }
    }

    pub fn errors(&self) -> &Vec<DustError> {
        &self.errors
    }
}

impl InterpreterError {
    /// Converts the error into a list of user-friendly reports that can be printed to the console.
    pub fn build_reports<'a>(self) -> Vec<Report<'a, (Arc<str>, Range<usize>)>> {
        let token_color = Color::Yellow;
        let type_color = Color::Green;
        let identifier_color = Color::Blue;

        let mut reports = Vec::new();

        for error in self.errors {
            let (mut builder, validation_error) = match error {
                DustError::Lex {
                    expected,
                    span,
                    reason,
                } => {
                    let description = if expected.is_empty() {
                        "Invalid character.".to_string()
                    } else {
                        format!("Expected {expected}.")
                    };

                    (
                        Report::build(
                            ReportKind::Custom("Lexing Error", Color::Yellow),
                            self.source_id.clone(),
                            span.1,
                        )
                        .with_message(description)
                        .with_label(
                            Label::new((self.source_id.clone(), span.0..span.1))
                                .with_message(reason)
                                .with_color(Color::Red),
                        ),
                        None,
                    )
                }
                DustError::Parse {
                    expected,
                    span,
                    found,
                } => {
                    let description = if expected.is_empty() {
                        "Invalid token.".to_string()
                    } else {
                        format!("Expected {expected}.")
                    };
                    let found = found
                        .unwrap_or_else(|| "End of input".to_string())
                        .fg(token_color);

                    (
                        Report::build(
                            ReportKind::Custom("Parsing Error", Color::Yellow),
                            self.source_id.clone(),
                            span.1,
                        )
                        .with_message(description)
                        .with_label(
                            Label::new((self.source_id.clone(), span.0..span.1))
                                .with_message(format!("{found} is not valid in this position."))
                                .with_color(Color::Red),
                        ),
                        None,
                    )
                }
                DustError::Validation { error, position } => (
                    Report::build(
                        ReportKind::Custom("Validation Error", Color::Magenta),
                        self.source_id.clone(),
                        position.1,
                    )
                    .with_message("The syntax is valid but this code would cause an error.")
                    .with_note(
                        "This error was detected by the interpreter before running the code.",
                    ),
                    Some(error),
                ),
                DustError::Runtime { error, position } => {
                    let note = match &error {
                        RuntimeError::Io(io_error) => &io_error.to_string(),
                        RuntimeError::RwLockPoison(_) => todo!(),
                        RuntimeError::ValidationFailure(_) => {
                            "This is the interpreter's fault. Please submit a bug with this error message."
                        }
                        RuntimeError::SerdeJson(serde_json_error) => &serde_json_error.to_string(),
                        RuntimeError::Use(_) => todo!(),
                    };

                    (
                            Report::build(
                                ReportKind::Custom("Runtime Error", Color::Red),
                                self.source_id.clone(),
                                position.1,
                            )
                            .with_message("An error occured that forced the program to exit. There may be unexpected side-effects because the program could not finish.")
                            .with_note(note)
                            .with_label(
                                Label::new((self.source_id.clone(), position.0..position.1)).with_message("Error occured here.")
                            ),
                        if let RuntimeError::ValidationFailure(validation_error) = error {
                            Some(validation_error)
                        } else {
                            None
                        },
                    )
                }
            };

            if let Some(validation_error) = validation_error {
                match validation_error {
                    ValidationError::CannotAssignToNone(postion) => {
                        builder.add_label(
                            Label::new((self.source_id.clone(), postion.0..postion.1))
                                .with_message(
                                    "This statement does not yield a value, you cannot assign a variable to it."
                                ),
                        );
                    }
                    ValidationError::ExpectedBoolean { actual, position } => {
                        builder.add_label(
                            Label::new((self.source_id.clone(), position.0..position.1))
                                .with_message(format!(
                                    "Expected {} but got {}.",
                                    "boolean".fg(type_color),
                                    actual.fg(type_color)
                                )),
                        );
                    }
                    ValidationError::ExpectedIntegerOrFloat(position) => {
                        builder.add_label(
                            Label::new((self.source_id.clone(), position.0..position.1))
                                .with_message(format!(
                                    "Expected {} or {}.",
                                    "integer".fg(type_color),
                                    "float".fg(type_color)
                                )),
                        );
                    }
                    ValidationError::FullTypeNotKnown {
                        identifier,
                        position,
                    } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!(
                                "The full type for {} must be known.",
                                identifier.fg(identifier_color)
                            ),
                        ),
                    ),
                    ValidationError::RwLockPoison(_) => todo!(),
                    ValidationError::TypeCheck {
                        conflict: TypeConflict { actual, expected },
                        actual_position,
                        expected_position,
                    } => {
                        if let Type::Generic {
                            concrete_type: None,
                            ..
                        } = actual
                        {
                            builder = builder.with_help("Try specifying the type using turbofish.");
                        }

                        let actual_type_message = if let Some(position) = expected_position {
                            builder.add_label(
                                Label::new((self.source_id.clone(), position.0..position.1))
                                    .with_message(format!(
                                        "Type {} established here.",
                                        expected.fg(type_color)
                                    )),
                            );

                            format!("Got type {} here.", actual.fg(type_color))
                        } else {
                            format!(
                                "Got type {} but expected {}.",
                                actual.fg(type_color),
                                expected.fg(type_color)
                            )
                        };

                        builder.add_label(
                            Label::new((
                                self.source_id.clone(),
                                actual_position.0..actual_position.1,
                            ))
                            .with_message(actual_type_message),
                        )
                    }
                    ValidationError::VariableNotFound {
                        identifier,
                        position,
                    } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!(
                                "Variable {} does not exist in this context.",
                                identifier.fg(identifier_color)
                            ),
                        ),
                    ),
                    ValidationError::CannotIndex { r#type, position } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!("Cannot index into a {}.", r#type.fg(type_color)),
                        ),
                    ),
                    ValidationError::CannotIndexWith {
                        collection_type,
                        collection_position,
                        index_type,
                        index_position,
                    } => {
                        builder = builder.with_message(format!(
                            "Cannot index into {} with {}.",
                            collection_type.clone().fg(type_color),
                            index_type.clone().fg(type_color)
                        ));

                        builder.add_labels([
                            Label::new((
                                self.source_id.clone(),
                                collection_position.0..collection_position.1,
                            ))
                            .with_message(format!(
                                "This has type {}.",
                                collection_type.fg(type_color),
                            )),
                            Label::new((
                                self.source_id.clone(),
                                index_position.0..index_position.1,
                            ))
                            .with_message(format!("This has type {}.", index_type.fg(type_color),)),
                        ])
                    }
                    ValidationError::ExpectedValueStatement(position) => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1))
                            .with_message("Expected a statement that yields a value."),
                    ),
                    ValidationError::ExpectedNonValueStatement(position) => {
                        builder.add_label(
                            Label::new((self.source_id.clone(), position.0..position.1))
                                .with_message("Expected a statement that does not yield a value."),
                        );
                        builder.add_label(
                            Label::new((self.source_id.clone(), position.0..position.1))
                                .with_message("Try adding a semicolon here."),
                        );
                    }
                    ValidationError::ExpectedFunction { actual, position } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!(
                                "Expected a function value but got {}.",
                                actual.fg(type_color)
                            ),
                        ),
                    ),
                    ValidationError::FieldNotFound {
                        identifier,
                        position,
                    } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!(
                                "This map has no field named {}.",
                                identifier.fg(identifier_color)
                            ),
                        ),
                    ),
                    ValidationError::WrongTypeArguments {
                        parameters,
                        arguments,
                    } => {
                        builder = builder.with_message(format!(
                            "Expected {parameters:?} arguments but got {arguments:?}."
                        ));
                    }
                    ValidationError::WrongValueArguments {
                        parameters,
                        arguments,
                    } => {
                        builder = builder.with_message(format!(
                            "Expected {parameters:?} arguments but got {arguments:?}."
                        ));
                    }
                    ValidationError::ExpectedIntegerFloatOrString { actual, position } => {
                        builder = builder.with_message(format!(
                            "Expected an {}, {} or {}.",
                            Type::Integer.fg(type_color),
                            Type::Float.fg(type_color),
                            Type::String.fg(type_color)
                        ));

                        builder.add_label(
                            Label::new((self.source_id.clone(), position.0..position.1))
                                .with_message(format!("This has type {}.", actual.fg(type_color),)),
                        )
                    }
                    ValidationError::ExpectedString { .. } => todo!(),
                    ValidationError::EnumDefinitionNotFound {
                        identifier,
                        position,
                    } => {
                        let message = format!(
                            "The enum {} does not exist in this context.",
                            identifier.fg(identifier_color),
                        );

                        if let Some(position) = position {
                            builder.add_label(
                                Label::new((self.source_id.clone(), position.0..position.1))
                                    .with_message(message),
                            )
                        } else {
                            builder = builder.with_message(message);
                        }
                    }
                    ValidationError::EnumVariantNotFound { .. } => todo!(),
                    ValidationError::ExpectedList { .. } => todo!(),
                    ValidationError::BuiltInFunctionFailure(reason) => builder
                        .add_label(Label::new((self.source_id.clone(), 0..0)).with_message(reason)),
                    ValidationError::CannotUsePath(_) => todo!(),
                    ValidationError::Uninitialized => todo!(),
                    ValidationError::WrongTypeArgumentsCount {
                        expected,
                        actual,
                        position,
                    } => builder.add_label(
                        Label::new((self.source_id.clone(), position.0..position.1)).with_message(
                            format!(
                                "Expected {} type arguments but got {}.",
                                expected.fg(type_color),
                                actual.fg(type_color)
                            ),
                        ),
                    ),
                    ValidationError::StructDefinitionNotFound {
                        identifier,
                        position,
                    } => todo!(),
                }
            }

            let report = builder.finish();

            reports.push(report);
        }

        reports
    }
}

#[cfg(test)]
mod tests {
    use abstract_tree::{AbstractNode, SourcePosition};

    use self::standard_library::std_full_compiled;

    use super::*;

    #[test]
    fn load_standard_library() {
        let context = Context::new();

        for abstract_tree in std_full_compiled() {
            abstract_tree
                .define_and_validate(&context, true, SourcePosition(0, usize::MAX))
                .unwrap();
            abstract_tree.run(&context, true).unwrap();
        }
    }
}
