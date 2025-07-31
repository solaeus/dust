//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{CompileError, JIT_ERROR_TEXT, JitError, Span};

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug, PartialEq)]
pub struct DustError<'src> {
    pub error: DustErrorKind,
    pub source: &'src str,
}

impl<'src> DustError<'src> {
    pub fn compile(error: CompileError, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Compile(error),
            source,
        }
    }

    pub fn jit(error: JitError) -> Self {
        DustError {
            error: DustErrorKind::Jit(error),
            source: JIT_ERROR_TEXT,
        }
    }

    pub fn report(&self) -> String {
        let (title, description, detail_snippets, help_snippets) = (
            self.error.title(),
            self.error.description(),
            self.error.detail_snippets(),
            self.error.help_snippets(),
        );
        let label = format!("{title}: {description}");
        let message = Level::Error
            .title(&label)
            .snippets(detail_snippets.iter().map(|(details, position)| {
                Snippet::source(self.source)
                    .annotation(Level::Info.span(position.0..position.1).label(details))
            }))
            .snippets(help_snippets.iter().map(|(help, position)| {
                Snippet::source(self.source)
                    .annotation(Level::Help.span(position.0..position.1).label(help))
            }));
        let mut report = String::new();
        let renderer = Renderer::styled();

        report.push_str(&renderer.render(message).to_string());

        report
    }
}

#[derive(Debug, PartialEq)]
pub enum DustErrorKind {
    Compile(CompileError),
    Jit(JitError),
}

impl DustErrorKind {
    fn title(&self) -> &'static str {
        match self {
            DustErrorKind::Compile(error) => error.title(),
            DustErrorKind::Jit(error) => error.title(),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            DustErrorKind::Compile(error) => error.description(),
            DustErrorKind::Jit(error) => error.description(),
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        match self {
            DustErrorKind::Compile(error) => error.detail_snippets(),
            DustErrorKind::Jit(error) => error.detail_snippets(),
        }
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        match self {
            DustErrorKind::Compile(error) => error.help_snippets(),
            DustErrorKind::Jit(error) => error.help_snippets(),
        }
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detail_snippets(&self) -> Vec<(String, Span)>;
    fn help_snippets(&self) -> Vec<(String, Span)>;
}
