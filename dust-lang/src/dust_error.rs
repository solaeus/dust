//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use annotate_snippets::{Group, Renderer};

use crate::{CompileError, Source, jit_vm::JitError, parser::ParseError, source::SourceFile};

const SOURCE_NOT_FOUND: &str = "<source not found>";

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub struct DustError {
    pub error: DustErrorKind,
    pub source: Source,
}

impl DustError {
    pub fn parse(errors: Vec<ParseError>, source: Source) -> Self {
        DustError {
            error: DustErrorKind::Parse(errors),
            source,
        }
    }

    pub fn compile(error: CompileError, source: Source) -> Self {
        DustError {
            error: DustErrorKind::Compile(error),
            source,
        }
    }

    pub fn jit(error: JitError) -> Self {
        DustError {
            error: DustErrorKind::Jit(error),
            source: Source::Script(SourceFile {
                name: Arc::new(SOURCE_NOT_FOUND.to_string()),
                source_code: Arc::new(SOURCE_NOT_FOUND.to_string()),
            }),
        }
    }

    pub fn report(&self) -> String {
        match &self.error {
            DustErrorKind::Parse(parse_errors) => {
                let mut report = Vec::new();

                for parse_error in parse_errors {
                    let source_file = self.source.get_file(parse_error.file_index());
                    let source = match source_file {
                        Some(file) => &file.source_code,
                        None => SOURCE_NOT_FOUND,
                    };
                    let group = parse_error.annotated_error(source);

                    report.push(group);
                }

                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustErrorKind::Compile(compile_error) => {
                let source_file = self.source.get_file(compile_error.file_index());
                let source = match source_file {
                    Some(file) => &file.source_code,
                    None => SOURCE_NOT_FOUND,
                };
                let report = [compile_error.annotated_error(source)];
                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustErrorKind::Jit(jit_error) => {
                let source_file = self.source.get_file(jit_error.file_index());
                let source = match source_file {
                    Some(file) => &file.source_code,
                    None => SOURCE_NOT_FOUND,
                };
                let report = [jit_error.annotated_error(source)];
                let renderer = Renderer::styled();

                renderer.render(&report)
            }
        }
    }
}

#[derive(Debug)]
pub enum DustErrorKind {
    Parse(Vec<ParseError>),
    Compile(CompileError),
    Jit(JitError),
}

impl Display for DustError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError {
    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a>;
    fn file_index(&self) -> usize;
}
