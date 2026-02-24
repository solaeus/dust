//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::fmt::{self, Display, Formatter};

use annotate_snippets::{Group, Level, Renderer};

use crate::{
    compiler::error::CompileError,
    parser::ParseError,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationMembers},
        scope_graph::ScopeId,
        symbol_table::SymbolId,
        type_graph::{TypeId, TypeMembers},
    },
    source::{Source, SourceError, SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind, SyntaxPayload},
};

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum DustError {
    Internal(InternalError),
    Source(SourceError),
    Parse(ParseError),
    Compile(CompileError),
}

impl DustError {
    pub fn into_internal(self) -> InternalError {
        match self {
            DustError::Internal(internal_error) => internal_error,
            DustError::Source(source_error) => InternalError::UnhandledSourceError(source_error),
            DustError::Parse(parse_error) => InternalError::UnhandledParseError(parse_error),
            DustError::Compile(compile_error) => {
                InternalError::UnhandledCompileError(compile_error)
            }
        }
    }
}

impl From<InternalError> for DustError {
    fn from(internal_error: InternalError) -> Self {
        DustError::Internal(internal_error)
    }
}

impl From<SourceError> for DustError {
    fn from(source_error: SourceError) -> Self {
        DustError::Source(source_error)
    }
}

impl From<ParseError> for DustError {
    fn from(parse_error: ParseError) -> Self {
        DustError::Parse(parse_error)
    }
}

impl From<CompileError> for DustError {
    fn from(compile_error: CompileError) -> Self {
        DustError::Compile(compile_error)
    }
}

impl<'a> AnnotatedError<'a> for DustError {
    type Context = (&'a Source<'a>, &'a Resolver);

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>) {
        match self {
            DustError::Internal(internal_error) => internal_error.add_report((), groups),
            DustError::Source(source_error) => source_error.add_report((), groups),
            DustError::Parse(parse_error) => parse_error.add_report(context.0, groups),
            DustError::Compile(compile_error) => compile_error.add_report(context, groups),
        }
    }
}

pub struct DustErrors<'a> {
    errors: Vec<DustError>,
    source: &'a Source<'a>,
    resolver: &'a Resolver,
}

impl<'a> DustErrors<'a> {
    pub fn new(errors: Vec<DustError>, source: &'a Source<'a>, resolver: &'a Resolver) -> Self {
        Self {
            errors,
            source,
            resolver,
        }
    }

    pub fn errors(&self) -> &Vec<DustError> {
        &self.errors
    }
}

impl<'a> Display for DustErrors<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut groups = Vec::with_capacity(self.errors.len());
        let renderer = Renderer::styled();

        for error in &self.errors {
            let start = groups.len();

            error.add_report((self.source, self.resolver), &mut groups);

            let display = renderer.render(&groups[start..]);

            write!(f, "{display}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum InternalError {
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
    ExpectedModuleDeclaration { declaration_id: DeclarationId },

    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),

    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),

    UnimplementedFeature(SyntaxKind),

    UnhandledSourceError(SourceError),
    UnhandledParseError(ParseError),
    UnhandledCompileError(CompileError),
}

impl<'a> AnnotatedError<'a> for InternalError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        let title = "Internal error".to_string();
        let message = format!("{self:#?}");
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
