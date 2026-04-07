//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::{
    fmt::{self, Debug, Display, Formatter},
    process,
};

use annotate_snippets::{Group, Level, Renderer};

use crate::{
    compiler::error::CompileError,
    constant_list::ConstantListError,
    parser::error::ParseError,
    compiler::resolver::{Resolver, error::ResolverError},
    source::{Source, SourceError},
    syntax::error::SyntaxError,
    vm::error::VmError,
};

#[derive(Debug)]
pub struct Error<'src> {
    errors: Vec<ErrorKind>,
    source: Option<Source<'src>>,
    resolver: Option<Box<Resolver>>,
}

impl<'src> Error<'src> {
    pub fn without_context(errors: Vec<ErrorKind>) -> Self {
        Self {
            errors,
            source: None,
            resolver: None,
        }
    }

    pub fn with_source(errors: Vec<ErrorKind>, source: Source<'src>) -> Self {
        Self {
            errors,
            source: Some(source),
            resolver: None,
        }
    }

    pub fn with_source_and_resolver(
        errors: Vec<ErrorKind>,
        source: Source<'src>,
        resolver: Resolver,
    ) -> Self {
        Self {
            errors,
            source: Some(source),
            resolver: Some(Box::new(resolver)),
        }
    }

    pub fn errors(&self) -> &Vec<ErrorKind> {
        &self.errors
    }

    pub fn print_and_exit(&self) -> ! {
        eprintln!("{self}");

        if self.errors.len() == 1 {
            eprintln!("1 error found.");
        } else {
            eprintln!("{} errors found.", self.errors.len());
        }

        process::exit(1);
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut report = Vec::with_capacity(self.errors.len());
        let renderer = Renderer::styled();

        for error in &self.errors {
            error.add_report(
                (self.source.as_ref(), self.resolver.as_deref()),
                &mut report,
            );

            let display = renderer.render(&report);

            report.clear();

            writeln!(f, "{display}")?;
        }

        Ok(())
    }
}

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum ErrorKind {
    Parse(ParseError),
    Compile(CompileError),
    Vm(VmError),
}

impl From<ParseError> for ErrorKind {
    fn from(parse_error: ParseError) -> Self {
        ErrorKind::Parse(parse_error)
    }
}

impl From<CompileError> for ErrorKind {
    fn from(compile_error: CompileError) -> Self {
        ErrorKind::Compile(compile_error)
    }
}

impl From<VmError> for ErrorKind {
    fn from(vm_error: VmError) -> Self {
        ErrorKind::Vm(vm_error)
    }
}

impl From<SyntaxError> for ErrorKind {
    fn from(error: SyntaxError) -> Self {
        ErrorKind::Compile(CompileError::Syntax(error))
    }
}

impl From<ResolverError> for ErrorKind {
    fn from(error: ResolverError) -> Self {
        ErrorKind::Compile(CompileError::Resolver(error))
    }
}

impl From<ConstantListError> for ErrorKind {
    fn from(error: ConstantListError) -> Self {
        ErrorKind::Compile(CompileError::ConstantList(error))
    }
}

impl From<SourceError> for ErrorKind {
    fn from(error: SourceError) -> Self {
        ErrorKind::Compile(CompileError::Source(error))
    }
}

impl<'a> AnnotatedError<'a> for ErrorKind {
    type Context = (Option<&'a Source<'a>>, Option<&'a Resolver>);

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>) {
        let (source, resolver) = context;

        match self {
            ErrorKind::Parse(parse_error) => {
                if let Some(source) = source {
                    parse_error.add_report(source, groups)
                } else {
                    MissingErrorContext.add_report((), groups);
                }
            }
            ErrorKind::Compile(compile_error) => {
                if let Some(resolver) = resolver
                    && let Some(source) = source
                {
                    compile_error.add_report((source, resolver), groups)
                } else {
                    MissingErrorContext.add_report((), groups);
                }
            }
            ErrorKind::Vm(vm_error) => {
                vm_error.add_report((), groups);
            }
        }
    }
}

/// An error occurred but the context needed to generate a report is missing.
#[derive(Debug)]
pub struct MissingErrorContext;

impl<'a> AnnotatedError<'a> for MissingErrorContext {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        self.add_internal_report(groups);
    }
}

pub trait AnnotatedError<'a> {
    type Context;

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>);

    fn add_internal_report(&self, groups: &mut Vec<Group<'a>>)
    where
        Self: Debug,
    {
        let group = Group::with_title(Level::ERROR.primary_title("Internal error"))
            .element(Level::ERROR.message(format!("{self:?}")))
            .element(Level::NOTE.message(
                "Woops! This is a bug. 🐛 If this is a released version of Dust, please report this to the developers.",
            ));

        groups.push(group);
    }
}
