//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Level, Renderer, Snippet};
use smallvec::SmallVec;

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
        let (title, description, detail_snippets, help_snippets) = self.error_data();
        let label = format!("{}: {}", title, description);
        let message = Level::Error
            .title(&label)
            .snippets(detail_snippets.iter().map(|(details, position)| {
                Snippet::source(self.source())
                    .annotation(Level::Info.span(position.0..position.1).label(details))
            }))
            .snippets(help_snippets.iter().map(|(help, position)| {
                Snippet::source(self.source())
                    .annotation(Level::Help.span(position.0..position.1).label(help))
            }));
        let mut report = String::new();
        let renderer = Renderer::styled();

        report.push_str(&renderer.render(message).to_string());

        report
    }

    fn error_data(
        &self,
    ) -> (
        &str,
        &str,
        SmallVec<[(String, Span); 2]>,
        SmallVec<[(String, Span); 2]>,
    ) {
        match self {
            Self::Compile { error, .. } => (
                CompileError::title(),
                error.description(),
                error.detail_snippets(),
                error.help_snippets(),
            ),
            Self::Runtime { error, .. } => (
                VmError::title(),
                error.description(),
                error.detail_snippets(),
                error.help_snippets(),
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
    fn detail_snippets(&self) -> SmallVec<[(String, Span); 2]>;
    fn help_snippets(&self) -> SmallVec<[(String, Span); 2]>;
}
