//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Debug, Display, Formatter};

use annotate_snippets::{Group, Level, Renderer};

use crate::{
    compiler::{context::Context, error::CompileError},
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

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl<'src> From<SourceError> for Error<'src> {
    fn from(source_error: SourceError) -> Self {
        Self::new(vec![ErrorKind::Source(source_error)], ErrorContext::None)
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (source, syntax, context) = match &self.context {
            ErrorContext::Full(source, syntax, context) => {
                (Some(source), Some(syntax), Some(context))
            }
            ErrorContext::Source(source) => (Some(source), None, None),
            ErrorContext::None => (None, None, None),
        };
        let mut groups = Vec::new();

        for error in &self.errors {
            match error {
                ErrorKind::Source(source_error) => source_error.add_report((), &mut groups),
                ErrorKind::Parse(parse_error) => {
                    if let Some(source) = source {
                        parse_error.add_report(source, &mut groups)
                    } else {
                        MissingErrorContext.add_report((), &mut groups);
                    }
                }
                ErrorKind::Compile(compile_error) => {
                    if let Some(source) = source
                        && let Some(syntax) = syntax
                        && let Some(context) = context
                    {
                        compile_error.add_report((source, syntax, context), &mut groups)
                    } else {
                        MissingErrorContext.add_report((), &mut groups);
                    }
                }
                ErrorKind::Vm(vm_error) => {
                    vm_error.add_report((), &mut groups);
                }
                ErrorKind::Meta(meta_error) => {
                    meta_error.add_report((), &mut groups);
                }
            }

            let report_string = Renderer::styled().render(&groups);

            groups.clear();
            writeln!(f, "{report_string}")?;
        }

        Ok(())
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

#[derive(Debug)]
pub enum ErrorContext<'src> {
    None,
    Source(Source<'src>),
    Full(Source<'src>, Syntax, Box<Context>),
}

impl<'src> ErrorContext<'src> {
    pub fn parts(&self) -> (Option<&Source<'src>>, Option<&Syntax>, Option<&Context>) {
        match self {
            ErrorContext::None => (None, None, None),
            ErrorContext::Source(source) => (Some(source), None, None),
            ErrorContext::Full(source, syntax, context) => {
                (Some(source), Some(syntax), Some(context))
            }
        }
    }
}

/// An error occurred but the context needed to generate a report is missing.
#[derive(Debug)]
pub struct MissingErrorContext;

impl<'a> DustError<'a> for MissingErrorContext {
    type Info = ();

    fn add_report(&self, _: Self::Info, groups: &mut Vec<Group<'a>>) {
        self.add_internal_report(groups);
    }
}

pub trait DustError<'a>: Sized + Debug
where
    ErrorKind: From<Self>,
{
    type Info;

    fn add_report(&self, context: Self::Info, groups: &mut Vec<Group<'a>>);

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
