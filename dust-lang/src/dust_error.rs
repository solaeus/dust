//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Group, Renderer};

use crate::{CompileError, parser::ParseError};

/// A top-level error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub struct DustError<'src> {
    pub error: DustErrorKind,
    pub source: &'src str,
}

impl<'src> DustError<'src> {
    pub fn parse(errors: Vec<ParseError>, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Parse(errors),
            source,
        }
    }

    pub fn compile(error: CompileError, source: &'src str) -> Self {
        DustError {
            error: DustErrorKind::Compile(error),
            source,
        }
    }

    // pub fn jit(error: JitError) -> Self {
    //     DustError {
    //         error: DustErrorKind::Jit(error),
    //         source: JIT_ERROR_TEXT,
    //     }
    // }

    pub fn report(&self) -> String {
        match &self.error {
            DustErrorKind::Parse(parse_errors) => {
                let mut report = Vec::new();

                for parse_error in parse_errors {
                    let group = parse_error.annotated_error(self.source);

                    report.push(group);
                }

                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustErrorKind::Compile(compile_error) => {
                let report = [compile_error.annotated_error(self.source)];
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
    // Jit(JitError),
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError {
    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a>;
}
