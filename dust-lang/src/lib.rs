pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod identifier;
pub mod lexer;
pub mod parser;
pub mod value;

use std::{
    ops::Range,
    sync::{Arc, RwLock},
    vec,
};

use abstract_tree::Type;
use ariadne::{Color, Fmt, Label, Report, ReportKind};
use context::Context;
use error::{Error, RuntimeError, TypeConflict, ValidationError};
use lexer::lex;
use parser::parse;
use rayon::prelude::*;
pub use value::Value;

pub fn interpret<'src>(source_id: &str, source: &str) -> Result<Option<Value>, InterpreterError> {
    let mut interpreter = Interpreter::new(Context::new());

    interpreter.load_std()?;
    interpreter.run(Arc::from(source_id), Arc::from(source))
}

pub fn interpret_without_std(
    source_id: &str,
    source: &str,
) -> Result<Option<Value>, InterpreterError> {
    let interpreter = Interpreter::new(Context::new());

    interpreter.run(Arc::from(source_id.to_string()), Arc::from(source))
}

pub struct Interpreter {
    context: Context,
    sources: Arc<RwLock<Vec<(Arc<str>, Arc<str>)>>>,
}

impl Interpreter {
    pub fn new(context: Context) -> Self {
        Interpreter {
            context,
            sources: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn run(
        &self,
        source_id: Arc<str>,
        source: Arc<str>,
    ) -> Result<Option<Value>, InterpreterError> {
        self.sources
            .write()
            .unwrap()
            .push((source_id.clone(), source.clone()));

        let tokens = lex(source.as_ref()).map_err(|errors| InterpreterError {
            source_id: source_id.clone(),
            errors,
        })?;
        let abstract_tree = parse(&tokens).map_err(|errors| InterpreterError {
            source_id: source_id.clone(),
            errors,
        })?;
        let value_option = abstract_tree
            .run(&self.context)
            .map_err(|errors| InterpreterError { source_id, errors })?;

        Ok(value_option)
    }

    pub fn load_std(&mut self) -> Result<(), InterpreterError> {
        let std_sources = [
            (
                Arc::from("std/io.ds"),
                Arc::from(include_str!("../../std/io.ds")),
            ),
            (
                Arc::from("std/thread.ds"),
                Arc::from(include_str!("../../std/thread.ds")),
            ),
        ];

        let error = std_sources
            .into_par_iter()
            .find_map_any(|(source_id, source)| self.run(source_id, source).err());

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn sources(&self) -> vec::IntoIter<(Arc<str>, Arc<str>)> {
        self.sources.read().unwrap().clone().into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub struct InterpreterError {
    source_id: Arc<str>,
    errors: Vec<Error>,
}

impl InterpreterError {
    pub fn errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

impl InterpreterError {
    pub fn build_reports<'a>(self) -> Vec<Report<'a, (Arc<str>, Range<usize>)>> {
        let mut reports = Vec::new();

        for error in self.errors {
            let (mut builder, validation_error) = match error {
            Error::Lex {
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
            Error::Parse {
                expected,
                span,
                reason,
            } => {
                let description = if expected.is_empty() {
                    "Invalid token.".to_string()
                } else {
                    format!("Expected {expected}.")
                };

                (
                    Report::build(
                        ReportKind::Custom("Parsing Error", Color::Yellow),
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
            Error::Validation { error, position } => (
                Report::build(
                    ReportKind::Custom("Validation Error", Color::Magenta),
                    self.source_id.clone(),
                    position.1,
                )
                .with_message("The syntax is valid but this code would cause an error.")
                .with_note("This error was detected by the interpreter before running the code."),
                Some(error),
            ),
            Error::Runtime { error, position } => (
                Report::build(
                    ReportKind::Custom("Runtime Error", Color::Red),
                    self.source_id.clone(),
                    position.1,
                )
                .with_message("An error occured that forced the program to exit.")
                .with_note(
                    "There may be unexpected side-effects because the program could not finish.",
                )
                .with_help(
                    "This is the interpreter's fault. Please submit a bug with this error message.",
                )
                .with_label(
                    Label::new((self.source_id.clone(), position.0..position.1)).with_message("Runtime error occured here.")
                ),
                if let RuntimeError::ValidationFailure(validation_error) = error {
                    Some(validation_error)
                } else {
                    None
                },
            ),
        };

            let type_color = Color::Green;
            let identifier_color = Color::Blue;

            if let Some(validation_error) = validation_error {
                match validation_error {
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
                    ValidationError::RwLockPoison(_) => todo!(),
                    ValidationError::TypeCheck {
                        conflict,
                        actual_position,
                        expected_position,
                    } => {
                        let TypeConflict { actual, expected } = conflict;

                        builder.add_labels([
                            Label::new((
                                self.source_id.clone(),
                                expected_position.0..expected_position.1,
                            ))
                            .with_message(format!(
                                "Type {} established here.",
                                expected.fg(type_color)
                            )),
                            Label::new((
                                self.source_id.clone(),
                                actual_position.0..actual_position.1,
                            ))
                            .with_message(format!("Got type {} here.", actual.fg(type_color))),
                        ]);
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
                    ValidationError::InterpreterExpectedReturn(_) => todo!(),
                    ValidationError::ExpectedFunction { .. } => todo!(),
                    ValidationError::ExpectedValue(_) => todo!(),
                    ValidationError::PropertyNotFound { .. } => todo!(),
                    ValidationError::WrongArguments { .. } => todo!(),
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
                }
            }
            let report = builder.finish();

            reports.push(report);
        }

        reports
    }
}
