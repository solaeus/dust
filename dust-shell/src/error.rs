use ariadne::{Color, Fmt, Label, Report, ReportKind};
use dust_lang::{
    abstract_tree::Type,
    error::{Error as DustError, RuntimeError, TypeConflict, ValidationError},
};
use std::{
    fmt::{self, Display, Formatter},
    io,
    ops::Range,
    rc::Rc,
};

#[derive(Debug)]
pub enum Error {
    Dust {
        errors: Vec<dust_lang::error::Error>,
    },
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl Error {
    pub fn build_reports<'id>(
        self,
        source_id: Rc<String>,
    ) -> Result<Vec<Report<'id, (Rc<String>, Range<usize>)>>, io::Error> {
        if let Error::Dust { errors } = self {
            let mut reports = Vec::new();

            for error in errors {
                let (mut builder, validation_error, error_position) = match error {
            DustError::Parse {
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
                        source_id.clone(),
                        span.1,
                    )
                    .with_message(description)
                    .with_label(
                        Label::new((source_id.clone(), span.0..span.1))
                            .with_message(reason)
                            .with_color(Color::Red),
                    ),
                    None,
                    span.into(),
                )
            }
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
                        source_id.clone(),
                        span.1,
                    )
                    .with_message(description)
                    .with_label(
                        Label::new((source_id.clone(), span.0..span.1))
                            .with_message(reason)
                            .with_color(Color::Red),
                    ),
                    None,
                    span.into(),
                )
            }
            DustError::Runtime { error, position } => (
                Report::build(
                    ReportKind::Custom("Runtime Error", Color::Red),
                    source_id.clone(),
                    position.1,
                )
                .with_message("An error occured that forced the program to exit.")
                .with_note(
                    "There may be unexpected side-effects because the program could not finish.",
                )
                .with_help(
                    "This is the interpreter's fault. Please submit a bug with this error message.",
                ),
                if let RuntimeError::ValidationFailure(validation_error) = error {
                    Some(validation_error)
                } else {
                    None
                },
                position,
            ),
            DustError::Validation { error, position } => (
                Report::build(
                    ReportKind::Custom("Validation Error", Color::Magenta),
                    source_id.clone(),
                    position.1,
                )
                .with_message("The syntax is valid but this code is not sound.")
                .with_note("This error was detected by the interpreter before running the code."),
                Some(error),
                position,
            ),
        };

                let type_color = Color::Green;
                let identifier_color = Color::Blue;

                if let Some(validation_error) = validation_error {
                    match validation_error {
                        ValidationError::ExpectedBoolean { actual, position } => {
                            builder.add_label(
                                Label::new((source_id.clone(), position.0..position.1))
                                    .with_message(format!(
                                        "Expected {} but got {}.",
                                        "boolean".fg(type_color),
                                        actual.fg(type_color)
                                    )),
                            );
                        }
                        ValidationError::ExpectedIntegerOrFloat(position) => {
                            builder.add_label(
                                Label::new((source_id.clone(), position.0..position.1))
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
                            expected_position: expected_postion,
                        } => {
                            let TypeConflict { actual, expected } = conflict;

                            builder = builder.with_message("A type conflict was found.");

                            builder.add_labels([
                                Label::new((
                                    source_id.clone(),
                                    expected_postion.0..expected_postion.1,
                                ))
                                .with_message(format!(
                                    "Type {} established here.",
                                    expected.fg(type_color)
                                )),
                                Label::new((
                                    source_id.clone(),
                                    actual_position.0..actual_position.1,
                                ))
                                .with_message(format!("Got type {} here.", actual.fg(type_color))),
                            ]);
                        }
                        ValidationError::VariableNotFound(identifier) => builder.add_label(
                            Label::new((source_id.clone(), error_position.0..error_position.1))
                                .with_message(format!(
                                    "Variable {} does not exist in this context.",
                                    identifier.fg(identifier_color)
                                )),
                        ),
                        ValidationError::CannotIndex { r#type, position } => builder.add_label(
                            Label::new((source_id.clone(), position.0..position.1)).with_message(
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
                                    source_id.clone(),
                                    collection_position.0..collection_position.1,
                                ))
                                .with_message(format!(
                                    "This has type {}.",
                                    collection_type.fg(type_color),
                                )),
                                Label::new((source_id.clone(), index_position.0..index_position.1))
                                    .with_message(format!(
                                        "This has type {}.",
                                        index_type.fg(type_color),
                                    )),
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

                            builder.add_labels([Label::new((
                                source_id.clone(),
                                position.0..position.1,
                            ))
                            .with_message(format!("This has type {}.", actual.fg(type_color),))])
                        }
                    }
                }
                let report = builder.finish();

                reports.push(report);
            }

            return Ok(reports);
        } else {
            return Ok(Vec::with_capacity(0));
        };
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Dust { errors } => {
                for error in errors {
                    writeln!(f, "{error:?}")?;
                }

                Ok(())
            }
            Error::Io(io_error) => {
                write!(f, "{io_error}")
            }
        }
    }
}
