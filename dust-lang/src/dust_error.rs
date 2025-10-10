//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Group, Renderer};

use crate::{
    compiler::CompileError,
    jit_vm::JitError,
    parser::ParseError,
    source::{Source, SourceCode, SourceFileId},
};

const SOURCE_NOT_AVAILABLE: &str = "<source not available>";

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
        let source = Source::new();

        source.add_file(crate::source::SourceFile {
            name: SOURCE_NOT_AVAILABLE.to_string(),
            source_code: SourceCode::String(SOURCE_NOT_AVAILABLE.to_string()),
        });

        DustError {
            error: DustErrorKind::Jit(error),
            source,
        }
    }

    pub fn report(&self) -> String {
        let source_files = &self.source.read_files();

        match &self.error {
            DustErrorKind::Parse(parse_errors) => {
                let mut report = Vec::new();

                for parse_error in parse_errors {
                    let source = source_files
                        .get(parse_error.file_id().0 as usize)
                        .map_or(SOURCE_NOT_AVAILABLE, |file| unsafe {
                            str::from_utf8_unchecked(file.source_code.as_ref())
                        });
                    let group = parse_error.annotated_error(source);

                    report.push(group);
                }

                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustErrorKind::Compile(compile_error) => {
                let source = source_files
                    .get(compile_error.file_id().0 as usize)
                    .map_or(SOURCE_NOT_AVAILABLE, |file| unsafe {
                        str::from_utf8_unchecked(file.source_code.as_ref())
                    });
                let report = [compile_error.annotated_error(source)];
                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustErrorKind::Jit(jit_error) => {
                let report = [jit_error.annotated_error(SOURCE_NOT_AVAILABLE)];
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
    fn file_id(&self) -> SourceFileId;
}
