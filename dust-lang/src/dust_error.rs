//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{CompileError, LexError, Span, parser::ParseError};

// use crate::{JIT_ERROR_TEXT, JitError, Span};

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub struct DustError<'src> {
    pub error: DustErrorKind,
    pub source: &'src str,
}

impl<'src> DustError<'src> {
    pub fn lex(error: LexError, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Lex(error),
            source,
        }
    }

    pub fn parse(errors: Vec<ParseError>, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Parse(errors),
            source,
        }
    }

    pub fn compile(error: CompileError, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Compile(error),
            source,
        }
    }

    // pub fn jit(error: JitError) -> Self {
    //     DustError {
    //         error: DustErrorKind::Jit(error),
    //         source: JIT_ERROR_TEXT,
    //     }
    // }

    pub fn report(&self) -> String {
        fn push_to_report(message: ErrorMessage, report: &mut String, source: &str) {
            let ErrorMessage {
                title,
                description,
                detail_snippets,
                help_snippet,
            } = message;

            let label = format!("{title}: {description}");
            let mut message = Level::Error
                .title(&label)
                .snippets(detail_snippets.iter().map(|(details, position)| {
                    Snippet::source(source).annotation(
                        Level::Error
                            .span(position.0 as usize..position.1 as usize)
                            .label(details),
                    )
                }));

            if let Some(help_snippet) = &help_snippet {
                message = message.footer(Level::Help.title(help_snippet));
            }

            let renderer = Renderer::styled();
            report.push_str(&renderer.render(message).to_string());
        }

        let mut report = String::new();

        match &self.error {
            DustErrorKind::Lex(error) => {
                let message = error.annotated_error();

                push_to_report(message, &mut report, self.source);
            }
            DustErrorKind::Parse(errors) => {
                for error in errors {
                    let message = error.annotated_error();

                    push_to_report(message, &mut report, self.source);
                }
            }
            DustErrorKind::Compile(error) => {
                let message = error.annotated_error();

                push_to_report(message, &mut report, self.source);
            }
        }

        report
    }
}

#[derive(Debug)]
pub enum DustErrorKind {
    Lex(LexError),
    Parse(Vec<ParseError>),
    Compile(CompileError),
    // Jit(JitError),
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError {
    fn annotated_error(&self) -> ErrorMessage;
}

pub struct ErrorMessage {
    pub title: &'static str,
    pub description: &'static str,
    pub detail_snippets: Vec<(String, Span)>,
    pub help_snippet: Option<String>,
}
