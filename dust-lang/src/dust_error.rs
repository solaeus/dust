//! Top-level error handling for the Dust language.
use annotate_snippets::{Level, Renderer, Snippet};
use std::fmt::Display;

use crate::{AnalysisError, LexError, ParseError, RuntimeError};

/// An error that occurred during the execution of the Dust language and its
/// corresponding source code.
#[derive(Debug, Clone, PartialEq)]
pub enum DustError<'src> {
    Runtime {
        runtime_error: RuntimeError,
        source: &'src str,
    },
    Analysis {
        analysis_error: AnalysisError,
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

impl<'src> DustError<'src> {
    pub fn runtime(runtime_error: RuntimeError, source: &'src str) -> Self {
        DustError::Runtime {
            runtime_error,
            source,
        }
    }

    pub fn analysis(analysis_error: AnalysisError, source: &'src str) -> Self {
        DustError::Analysis {
            analysis_error,
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
            DustError::Runtime { .. } => "Runtime error",
            DustError::Analysis { .. } => "Analysis error",
            DustError::Parse { .. } => "Parse error",
            DustError::Lex { .. } => "Lex error",
        }
    }

    pub fn position(&self) -> (usize, usize) {
        match self {
            DustError::Runtime { runtime_error, .. } => runtime_error.position(),
            DustError::Analysis { analysis_error, .. } => analysis_error.position(),
            DustError::Parse { parse_error, .. } => parse_error.position(),
            DustError::Lex { lex_error, .. } => lex_error.position(),
        }
    }

    pub fn source(&self) -> &'src str {
        match self {
            DustError::Runtime { source, .. } => source,
            DustError::Analysis { source, .. } => source,
            DustError::Parse { source, .. } => source,
            DustError::Lex { source, .. } => source,
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
            DustError::Runtime { runtime_error, .. } => write!(f, "{runtime_error}"),
            DustError::Analysis { analysis_error, .. } => write!(f, "{analysis_error}"),
            DustError::Parse { parse_error, .. } => write!(f, "{parse_error}"),
            DustError::Lex { lex_error, .. } => write!(f, "{lex_error}"),
        }
    }
}
