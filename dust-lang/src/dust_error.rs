//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Group, Renderer};

use crate::{
    compiler::error::CompileError, parser::ParseError, resolver::Resolver, source::Source,
};

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum DustError<'src> {
    Parse {
        errors: Vec<ParseError>,
        source: Source<'src>,
    },
    Compile {
        errors: Vec<CompileError>,
        source: Source<'src>,
        resolver: Box<Resolver>,
    },
    // Jit(JitError),
}

impl<'src> DustError<'src> {
    pub fn parse(errors: Vec<ParseError>, source: Source<'src>) -> Self {
        DustError::Parse { errors, source }
    }

    pub fn compile(errors: Vec<CompileError>, source: Source<'src>, resolver: Resolver) -> Self {
        DustError::Compile {
            errors,
            source,
            resolver: Box::new(resolver),
        }
    }

    // pub fn jit(error: JitError) -> Self {
    //     DustError::Jit(error)
    // }

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
                errors,
                source,
                resolver,
            } => {
                let mut report = Vec::new();

                for compile_error in errors {
                    let group = compile_error.annotated_error((source, resolver));

                    report.push(group);
                }

                let renderer = Renderer::styled();

                renderer.render(&report)
            } // DustError::Jit(jit_error) => {
              //     let report = [jit_error.annotated_error(())];
              //     let renderer = Renderer::styled();

              //     renderer.render(&report)
              // }
        }
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

pub trait AnnotatedError<'a> {
    type Input;

    fn annotated_error(&self, input: Self::Input) -> Group<'a>;
}
