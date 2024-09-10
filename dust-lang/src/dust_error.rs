use annotate_snippets::{Level, Renderer, Snippet};

use crate::{vm::VmError, LexError, ParseError};

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
                let label = format!("Runtime {}", error.title());
                let description = error.description();
                let message = Level::Error.title(&label).snippet(
                    Snippet::source(source).fold(true).annotation(
                        Level::Error
                            .span(position.0..position.1)
                            .label(&description),
                    ),
                );

                report.push_str(&renderer.render(message).to_string());
            }
            _ => todo!(),
        }

        report
    }
}
