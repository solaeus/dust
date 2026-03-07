//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::{
    fmt::{self, Display, Formatter},
    process,
};

use annotate_snippets::{Group, Level, Renderer};
use tracing::error;

use crate::{
    compiler::error::CompileError,
    parser::ParseError,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationMembers},
        scope_graph::ScopeId,
        symbol_table::SymbolId,
        type_graph::{TypeId, TypeMembers, TypeNode},
    },
    source::{Source, SourceError, SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind, SyntaxPayload},
};

#[derive(Debug)]
pub struct Error<'src> {
    errors: Vec<ErrorKind>,
    source: Source<'src>,
    resolver: Option<Box<Resolver>>,
}

impl<'src> Error<'src> {
    pub fn with_source(errors: Vec<ErrorKind>, source: Source<'src>) -> Self {
        Self {
            errors,
            source,
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
            source,
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
        let mut groups = Vec::with_capacity(self.errors.len());
        let renderer = Renderer::styled();

        for (index, error) in self.errors.iter().enumerate() {
            let start = groups.len();

            error.add_report((&self.source, self.resolver.as_deref()), &mut groups);

            let display = renderer.render(&groups[start..]);

            writeln!(f, "{display}")?;

            if index < self.errors.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum ErrorKind {
    Internal(InternalError),
    Source(SourceError),
    Parse(ParseError),
    Compile(CompileError),
}

impl From<InternalError> for ErrorKind {
    fn from(internal_error: InternalError) -> Self {
        ErrorKind::Internal(internal_error)
    }
}

impl From<SourceError> for ErrorKind {
    fn from(source_error: SourceError) -> Self {
        ErrorKind::Source(source_error)
    }
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

impl<'a> AnnotatedError<'a> for ErrorKind {
    type Context = (&'a Source<'a>, Option<&'a Resolver>);

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>) {
        let (source, resolver) = context;

        match self {
            ErrorKind::Internal(internal_error) => internal_error.add_report((), groups),
            ErrorKind::Source(source_error) => source_error.add_report((), groups),
            ErrorKind::Parse(parse_error) => parse_error.add_report(source, groups),
            ErrorKind::Compile(compile_error) => {
                if let Some(resolver) = resolver {
                    compile_error.add_report((source, resolver), groups)
                } else {
                    error!("Missing error messages due to incomplete error context.");
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum InternalError {
    InvalidConstantTable,

    MissingSourceFile(SourceFileId),
    MissingSourceFileContent { span: Span, length: usize },

    MissingSyntaxTree(SourceFileId),
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChild { total_children: usize },
    MissingSyntaxChildren(SyntaxPayload),
    InvalidSyntaxPayload(SyntaxPayload),
    ExpectedSyntaxChildren { expected: usize, actual: usize },

    MissingSymbol(SymbolId),

    MissingDeclaration(DeclarationId),
    MissingDeclarationMember(u32),
    MissingDeclarationMembers(DeclarationMembers),
    MissingDeclarationType(DeclarationId),
    MissingDeclarationBinding(SyntaxId),
    ExpectedModuleDeclaration(DeclarationId),
    ExpectedTypeDeclaration(DeclarationId),

    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),

    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),

    InvalidRegisterCount { expected: usize, found: usize },
    ExpectedListType { found: TypeNode },
    ExpectedFloatRegister,
    ExpectedIntegerRegister,
    ExpectedEmissionTarget { node_kind: SyntaxKind },
}

impl InternalError {
    pub fn print_and_exit(&self) -> ! {
        eprintln!("{self}");

        process::exit(1);
    }
}

impl<'a> AnnotatedError<'a> for InternalError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        let title = "Internal error".to_string();
        let message = format!("{self:?}");
        let help = "This is a bug. 🐛 If this is a released version of Dust, please report this to the developers.".to_string();
        let group = Group::with_title(Level::ERROR.primary_title(title))
            .element(Level::ERROR.message(message))
            .element(Level::NOTE.message(help));

        groups.push(group);
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut groups = Vec::with_capacity(1);

        self.add_report((), &mut groups);

        let renderer = Renderer::styled();
        let display = renderer.render(&groups);

        write!(f, "{display}")
    }
}

pub trait AnnotatedError<'a> {
    type Context;

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>);
}
