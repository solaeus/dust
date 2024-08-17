//! Top-level error handling for the Dust language.
use annotate_snippets::{Level, Renderer, Snippet};
use std::fmt::Display;

use crate::{AnalysisError, LexError, ParseError, RuntimeError};

/// An error that occurred during the execution of the Dust language and its
/// corresponding source code.
#[derive(Debug, Clone, PartialEq)]
pub enum DustError<'src> {
    VmError {
        runtime_error: RuntimeError,
        source: &'src str,
    },
    AnalysisError {
        analysis_error: AnalysisError,
        source: &'src str,
    },
    ParseError {
        parse_error: ParseError,
        source: &'src str,
    },
    LexError {
        lex_error: LexError,
        source: &'src str,
    },
}

impl<'src> DustError<'src> {
    pub fn title(&self) -> &'static str {
        match self {
            DustError::VmError { .. } => "Runtime error",
            DustError::AnalysisError { .. } => "Analysis error",
            DustError::ParseError { .. } => "Parse error",
            DustError::LexError { .. } => "Lex error",
        }
    }

    pub fn position(&self) -> (usize, usize) {
        match self {
            DustError::VmError { runtime_error, .. } => runtime_error.position(),
            DustError::AnalysisError { analysis_error, .. } => analysis_error.position(),
            DustError::ParseError { parse_error, .. } => parse_error.position(),
            DustError::LexError { lex_error, .. } => lex_error.position(),
        }
    }

    pub fn source(&self) -> &'src str {
        match self {
            DustError::VmError { source, .. } => source,
            DustError::AnalysisError { source, .. } => source,
            DustError::ParseError { source, .. } => source,
            DustError::LexError { source, .. } => source,
        }
    }

    pub fn report(&self) -> String {
        let title = self.title();
        let span = self.position();
        let label = self.to_string();
        let message = Level::Error.title(title).snippet(
            Snippet::source(self.source())
                .annotation(Level::Info.span(span.0..span.1).label(&label)),
        );
        let renderer = Renderer::styled();

        format!("{}", renderer.render(message))
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DustError::VmError { runtime_error, .. } => write!(f, "{runtime_error}"),
            DustError::AnalysisError { analysis_error, .. } => write!(f, "{analysis_error}"),
            DustError::ParseError { parse_error, .. } => write!(f, "{parse_error}"),
            DustError::LexError { lex_error, .. } => write!(f, "{lex_error}"),
        }
    }
}
