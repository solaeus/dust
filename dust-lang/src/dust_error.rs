//! Top-level error handling for the Dust language.
use annotate_snippets::{Level, Message, Renderer, Snippet};
use std::fmt::Display;

use crate::{AnalysisError, ContextError, LexError, ParseError, RuntimeError, Span};

/// An error that occurred during the execution of the Dust language and its
/// corresponding source code.
#[derive(Debug, Clone, PartialEq)]
pub enum DustError<'src> {
    ContextError(ContextError),
    Runtime {
        runtime_error: RuntimeError,
        source: &'src str,
    },
    Analysis {
        analysis_errors: Vec<AnalysisError>,
        source: &'src str,
    },
    Parse {
        parse_error: ParseError,
        source: &'src str,
    },
    Lex {
        lex_error: LexError,
        source: &'src str,
    },
}

impl<'src> From<ContextError> for DustError<'src> {
    fn from(error: ContextError) -> Self {
        Self::ContextError(error)
    }
}

impl<'src> DustError<'src> {
    pub fn runtime(runtime_error: RuntimeError, source: &'src str) -> Self {
        DustError::Runtime {
            runtime_error,
            source,
        }
    }

    pub fn analysis<T: Into<Vec<AnalysisError>>>(analysis_errors: T, source: &'src str) -> Self {
        DustError::Analysis {
            analysis_errors: analysis_errors.into(),
            source,
        }
    }

    pub fn parse(parse_error: ParseError, source: &'src str) -> Self {
        DustError::Parse {
            parse_error,
            source,
        }
    }

    pub fn lex(lex_error: LexError, source: &'src str) -> Self {
        DustError::Lex { lex_error, source }
    }

    pub fn title(&self) -> &'static str {
        match self {
            DustError::ContextError(_) => "Context error",
            DustError::Runtime { .. } => "Runtime error",
            DustError::Analysis { .. } => "Analysis error",
            DustError::Parse { .. } => "Parse error",
            DustError::Lex { .. } => "Lex error",
        }
    }

    pub fn source(&self) -> &'src str {
        match self {
            DustError::ContextError(_) => "",
            DustError::Runtime { source, .. } => source,
            DustError::Analysis { source, .. } => source,
            DustError::Parse { source, .. } => source,
            DustError::Lex { source, .. } => source,
        }
    }

    fn footer(&self) -> Vec<(&'static str, Span, String)> {
        match self {
            DustError::ContextError(_) => vec![],
            DustError::Runtime { runtime_error, .. } => {
                let mut error_data = vec![(
                    "Runtime error",
                    runtime_error.position(),
                    runtime_error.to_string(),
                )];

                if let RuntimeError::Expression { error, position } = runtime_error {
                    error_data.push((
                        "Error occured at this expression",
                        *position,
                        error.to_string(),
                    ));
                }

                error_data
            }
            DustError::Analysis {
                analysis_errors, ..
            } => analysis_errors
                .iter()
                .map(|error| ("Analysis error", error.position(), error.to_string()))
                .collect(),
            DustError::Parse { parse_error, .. } => vec![(
                "Parse error",
                parse_error.position(),
                parse_error.to_string(),
            )],
            DustError::Lex { lex_error, .. } => {
                vec![("Lex error", lex_error.position(), lex_error.to_string())]
            }
        }
    }

    pub fn report(&self) -> String {
        let mut report = String::new();
        let renderer = Renderer::styled();

        match self {
            DustError::ContextError(_) => {
                let message = Level::Error.title("Context error");

                report.push_str(&renderer.render(message).to_string());
            }
            DustError::Runtime {
                runtime_error,
                source,
            } => {
                let error = runtime_error.root_error();
                let position = error.position();
                let label = error.to_string();
                let message = Level::Error
                    .title("Runtime error")
                    .snippet(
                        Snippet::source(source)
                            .fold(true)
                            .annotation(Level::Error.span(position.0..position.1).label(&label)),
                    )
                    .footer(
                        Level::Error
                            .title("This error occured during the execution of the Dust program."),
                    );

                report.push_str(&renderer.render(message).to_string());
                report.push('\n');
            }
            DustError::Analysis {
                analysis_errors,
                source,
            } => todo!(),
            DustError::Parse {
                parse_error,
                source,
            } => todo!(),
            DustError::Lex { lex_error, source } => todo!(),
        }

        report
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DustError::ContextError(context_error) => write!(f, "{context_error}"),
            DustError::Runtime { runtime_error, .. } => write!(f, "{runtime_error}"),
            DustError::Analysis {
                analysis_errors, ..
            } => {
                for error in analysis_errors {
                    write!(f, "{error} ")?;
                }

                Ok(())
            }
            DustError::Parse { parse_error, .. } => write!(f, "{parse_error}"),
            DustError::Lex { lex_error, .. } => write!(f, "{lex_error}"),
        }
    }
}
