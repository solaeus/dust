//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{CompileError, Span, VmError};

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug, PartialEq)]
pub enum DustError<'src> {
    Compile {
        error: CompileError,
        source: &'src str,
    },
    Runtime {
        error: VmError,
        source: &'src str,
    },
}

impl<'src> DustError<'src> {
    pub fn compile(error: CompileError, source: &'src str) -> Self {
        DustError::Compile { error, source }
    }

    pub fn runtime(error: VmError, source: &'src str) -> Self {
        DustError::Runtime { error, source }
    }

    pub fn report(&self) -> String {
        let (position, title, description, details) = self.error_data();
        let label = format!("{}: {}", title, description);
        let details = details.unwrap_or_else(|| "While parsing this code".to_string());
        let message = Level::Error.title(&label).snippet(
            Snippet::source(self.source())
                .fold(false)
                .annotation(Level::Error.span(position.0..position.1).label(&details)),
        );
        let mut report = String::new();
        let renderer = Renderer::styled();

        report.push_str(&renderer.render(message).to_string());

        report
    }

    fn error_data(&self) -> (Span, &str, &str, Option<String>) {
        match self {
            Self::Compile { error, .. } => (
                error.position(),
                CompileError::title(),
                error.description(),
                error.details(),
            ),
            Self::Runtime { error, .. } => (
                error.position(),
                VmError::title(),
                error.description(),
                error.details(),
            ),
        }
    }

    fn source(&self) -> &str {
        match self {
            Self::Compile { source, .. } => source,
            Self::Runtime { source, .. } => source,
        }
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError {
    fn title() -> &'static str;
    fn description(&self) -> &'static str;
    fn details(&self) -> Option<String>;
    fn position(&self) -> Span;
}
