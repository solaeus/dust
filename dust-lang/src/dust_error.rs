//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Group, Renderer};

use crate::{
    compiler::{CompileError, Resolver},
    jit_vm::JitError,
    parser::ParseError,
    source::Source,
};

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum DustError {
    Parse {
        errors: Vec<ParseError>,
        source: Source,
    },
    Compile {
        error: Box<CompileError>,
        source: Source,
        resolver: Box<Resolver>,
    },
    Jit(JitError),
}

impl DustError {
    pub fn parse(errors: Vec<ParseError>, source: Source) -> Self {
        DustError::Parse { errors, source }
    }

    pub fn compile(error: CompileError, source: Source, resolver: Resolver) -> Self {
        DustError::Compile {
            error: Box::new(error),
            source,
            resolver: Box::new(resolver),
        }
    }

    pub fn jit(error: JitError) -> Self {
        DustError::Jit(error)
    }

    pub fn report(&self) -> String {
        match self {
            DustError::Parse { errors, source } => {
                let mut report = Vec::new();

                for parse_error in errors {
                    let group = parse_error.annotated_error(source);

                    report.push(group);
                }

                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustError::Compile {
                error,
                source,
                resolver,
            } => {
                let report = [error.annotated_error((source, resolver))];
                let renderer = Renderer::styled();

                renderer.render(&report)
            }
            DustError::Jit(jit_error) => {
                let report = [jit_error.annotated_error(())];
                let renderer = Renderer::styled();

                renderer.render(&report)
            }
        }
    }
}

impl Display for DustError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError<'a> {
    type Input;

    fn annotated_error(&self, input: Self::Input) -> Group<'a>;
}
