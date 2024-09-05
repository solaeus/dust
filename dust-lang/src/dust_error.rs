//! Top-level error handling for the Dust language.
use annotate_snippets::{Level, Renderer, Snippet};
use std::fmt::Display;

use crate::{AnalysisError, ContextError, LexError, ParseError, RuntimeError};

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
                report.push_str("\n\n");
            }
            DustError::Analysis {
                analysis_errors,
                source,
            } => {
                for error in analysis_errors {
                    let position = error.position();
                    let label = error.to_string();
                    let message =
                        Level::Warning
                            .title("Analysis error")
                            .snippet(Snippet::source(source).fold(true).annotation(
                                Level::Warning.span(position.0..position.1).label(&label),
                            ))
                            .footer(
                                Level::Warning
                                    .title("This error was found without running the program."),
                            );

                    report.push_str(&renderer.render(message).to_string());
                    report.push_str("\n\n");
                }
            }
            DustError::Parse {
                parse_error,
                source,
            } => {
                if let ParseError::Lex(lex_error) = parse_error {
                    let lex_error_report = DustError::lex(lex_error.clone(), source).report();

                    report.push_str(&lex_error_report);

                    return report;
                }

                let position = parse_error.position();
                let label = parse_error.to_string();
                let message = Level::Error.title("Parse error").snippet(
                    Snippet::source(source)
                        .fold(true)
                        .annotation(Level::Error.span(position.0..position.1).label(&label)),
                );

                report.push_str(&renderer.render(message).to_string());
            }
            DustError::Lex { lex_error, source } => {
                let position = lex_error.position();
                let label = lex_error.to_string();
                let message = Level::Error.title("Lex error").snippet(
                    Snippet::source(source)
                        .fold(true)
                        .annotation(Level::Error.span(position.0..position.1).label(&label)),
                );

                report.push_str(&renderer.render(message).to_string());
            }
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
