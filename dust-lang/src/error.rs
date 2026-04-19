//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::{
    fmt::{self, Debug, Display, Formatter},
    process,
};

use annotate_snippets::{Group, Level, Renderer};

use crate::{
    compiler::{error::CompileError, resolver::Resolver},
    constants::ConstantsError,
    parser::error::ParseError,
    source::{Source, SourceError},
    syntax::{Syntax, error::SyntaxError},
    vm::error::VmError,
};

#[derive(Debug)]
pub struct Error<'src> {
    errors: Vec<ErrorKind>,
    context: ErrorContext<'src>,
}

impl<'src> Error<'src> {
    pub fn new(errors: Vec<ErrorKind>, context: ErrorContext<'src>) -> Self {
        Self { errors, context }
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
            error.add_report(self.context.parts(), &mut report);

            let display = renderer.render(&report);

            report.clear();

            writeln!(f, "{display}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ErrorContext<'src> {
    None,
    Source(Source<'src>),
    Full(Source<'src>, Syntax, Box<Resolver>),
}

impl<'src> ErrorContext<'src> {
    pub fn parts(&self) -> (Option<&Source<'src>>, Option<&Syntax>, Option<&Resolver>) {
        match self {
            ErrorContext::None => (None, None, None),
            ErrorContext::Source(source) => (Some(source), None, None),
            ErrorContext::Full(source, syntax, resolver) => {
                (Some(source), Some(syntax), Some(resolver))
            }
        }
    }
}

/// An error that can occur while interpreting Dust code.
#[derive(Debug)]
pub enum ErrorKind {
    Source(SourceError),
    Parse(ParseError),
    Compile(CompileError),
    Vm(VmError),
    Meta(MissingErrorContext),
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

impl From<ConstantsError> for ErrorKind {
    fn from(error: ConstantsError) -> Self {
        ErrorKind::Compile(CompileError::ConstantList(error))
    }
}

impl From<SourceError> for ErrorKind {
    fn from(error: SourceError) -> Self {
        ErrorKind::Source(error)
    }
}

impl From<MissingErrorContext> for ErrorKind {
    fn from(error: MissingErrorContext) -> Self {
        ErrorKind::Meta(error)
    }
}

impl<'a> DustError<'a> for ErrorKind {
    type Context = (
        Option<&'a Source<'a>>,
        Option<&'a Syntax>,
        Option<&'a Resolver>,
    );

    fn add_report(&self, (source, syntax, resolver): Self::Context, groups: &mut Vec<Group<'a>>) {
        match self {
            ErrorKind::Source(source_error) => source_error.add_report((), groups),
            ErrorKind::Parse(parse_error) => {
                if let Some(source) = source {
                    parse_error.add_report(source, groups)
                } else {
                    MissingErrorContext.add_report((), groups);
                }
            }
            ErrorKind::Compile(compile_error) => {
                if let Some(source) = source
                    && let Some(syntax) = syntax
                    && let Some(resolver) = resolver
                {
                    compile_error.add_report((source, syntax, resolver), groups)
                } else {
                    MissingErrorContext.add_report((), groups);
                }
            }
            ErrorKind::Vm(vm_error) => {
                vm_error.add_report((), groups);
            }
            ErrorKind::Meta(meta_error) => {
                meta_error.add_report((), groups);
            }
        }
    }
}

/// An error occurred but the context needed to generate a report is missing.
#[derive(Debug)]
pub struct MissingErrorContext;

impl<'a> DustError<'a> for MissingErrorContext {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        self.add_internal_report(groups);
    }
}

pub trait DustError<'a>: Sized + Debug
where
    ErrorKind: From<Self>,
{
    type Context;

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>);

    fn to_full_error(self) -> Error<'a> {
        Error::new(vec![ErrorKind::from(self)], ErrorContext::None)
    }

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
