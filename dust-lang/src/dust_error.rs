//! Top-level Dust errors with source code annotations.
use annotate_snippets::{Level, Renderer, Snippet};

use crate::{vm::VmError, CompileError, Span};

/// A top-level error that can occur during the execution of Dust code.
///
/// This error can display nicely formatted messages with source code annotations.
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
    pub fn report(&self) -> String {
        let mut report = String::new();
        let renderer = Renderer::styled();

        match self {
            DustError::Runtime { error, source } => {
                let position = error.position();
                let label = format!("{}: {}", VmError::title(), error.description());
                let details = error
                    .details()
                    .unwrap_or_else(|| "While running this code".to_string());
                let message = Level::Error.title(&label).snippet(
                    Snippet::source(source)
                        .fold(false)
                        .annotation(Level::Error.span(position.0..position.1).label(&details)),
                );

                report.push_str(&renderer.render(message).to_string());
            }
            DustError::Compile { error, source } => {
                let position = error.position();
                let label = format!("{}: {}", CompileError::title(), error.description());
                let details = error
                    .details()
                    .unwrap_or_else(|| "While parsing this code".to_string());
                let message = Level::Error.title(&label).snippet(
                    Snippet::source(source)
                        .fold(false)
                        .annotation(Level::Error.span(position.0..position.1).label(&details)),
                );

                report.push_str(&renderer.render(message).to_string());
            }
        }

        report
    }
}

pub trait AnnotatedError {
    fn title() -> &'static str;
    fn description(&self) -> &'static str;
    fn details(&self) -> Option<String>;
    fn position(&self) -> Span;
}
