use annotate_snippets::{Level, Renderer, Snippet};

use crate::{vm::VmError, LexError, ParseError, Span};

#[derive(Debug, PartialEq)]
pub enum DustError<'src> {
    Lex {
        error: LexError,
        source: &'src str,
    },
    Parse {
        error: ParseError,
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
                let label = format!("Runtime error: {}", error.description());
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
            DustError::Parse { error, source } => {
                let position = error.position();
                let label = format!("Parse error: {}", error.description());
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
            _ => todo!(),
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
