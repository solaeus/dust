//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Level, Renderer, Snippet};

// use crate::{JIT_ERROR_TEXT, JitError, Span};

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug, PartialEq)]
pub struct DustError<'src> {
    pub error: DustErrorKind,
    pub source: &'src str,
}

impl<'src> DustError<'src> {
    // pub fn compile(error: CompileError, source: &'src str) -> Self {
    //     DustError {
    //         error: DustErrorKind::Compile(error),
    //         source,
    //     }
    // }

    // pub fn jit(error: JitError) -> Self {
    //     DustError {
    //         error: DustErrorKind::Jit(error),
    //         source: JIT_ERROR_TEXT,
    //     }
    // }

    // pub fn report(&self) -> String {
    //     let ErrorMessage {
    //         title,
    //         description,
    //         detail_snippets,
    //         help_snippet,
    //     } = self.error.annotated_error();
    //     let label = format!("{title}: {description}");
    //     let mut message = Level::Error
    //         .title(&label)
    //         .snippets(detail_snippets.iter().map(|(details, position)| {
    //             Snippet::source(self.source)
    //                 .annotation(Level::Error.span(position.0..position.1).label(details))
    //         }));

    //     if let Some(help_snippet) = &help_snippet {
    //         message = message.footer(Level::Help.title(help_snippet));
    //     }

    //     let mut report = String::new();
    //     let renderer = Renderer::styled();

    //     report.push_str(&renderer.render(message).to_string());

    //     report
    // }
}

#[derive(Debug, PartialEq)]
pub enum DustErrorKind {
    // Compile(CompileError),
    // Jit(JitError),
}

// impl AnnotatedError for DustErrorKind {
//     fn annotated_error(&self) -> ErrorMessage {
//         match self {
//             // DustErrorKind::Compile(error) => error.annotated_error(),
//             DustErrorKind::Jit(error) => error.annotated_error(),
//         }
//     }
// }

// impl Display for DustError<'_> {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         write!(f, "{}", self.report())
//     }
// }

// pub trait AnnotatedError {
//     fn annotated_error(&self) -> ErrorMessage;
// }

// pub struct ErrorMessage {
//     pub title: &'static str,
//     pub description: &'static str,
//     pub detail_snippets: Vec<(String, Span)>,
//     pub help_snippet: Option<String>,
// }
