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

#[derive(Debug)]
pub struct DustErrors<'src> {
    errors: Vec<DustError>,
    source: Source<'src>,
    resolver: Resolver,
}

impl<'src> DustErrors<'src> {
    pub fn new(errors: Vec<DustError>, source: Source<'src>, resolver: Resolver) -> Self {
        Self {
            errors,
            source,
            resolver,
        }
    }
}

impl Display for DustErrors<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let renderer = Renderer::styled();
        let mut reports = Vec::with_capacity(self.errors.len());

        for error in &self.errors {
            let start = reports.len();

            match error {
                DustError::Internal(internal_error) => {
                    let title = "Internal error".to_string();
                    let message = format!("{internal_error:#?}");
                    let help = "This is a bug. 🐛 If this is a released version of Dust, please report this to the developers.".to_string();
                    let group = Group::with_title(Level::ERROR.primary_title(title))
                        .element(Level::ERROR.message(message))
                        .element(Level::NOTE.message(help));

                    reports.push(group);
                }
                DustError::Source(source_error) => source_error.annotated_error(&(), &mut reports),
                _ => todo!(),
            }

            let end = reports.len();

            writeln!(f, "{}", renderer.render(&reports[start..end]))?;
        }

        Ok(())
    }
}

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum DustError {
    Internal(InternalError),
    Source(SourceError),
    Parse(ParseError),
    Compile(CompileError),
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

#[derive(Debug)]
pub enum InternalError {
    MissingSymbol(SymbolId),

    MissingDeclaration(DeclarationId),
    MissingDeclarationMember(u32),
    MissingDeclarationMembers(DeclarationMembers),
    MissingDeclarationType(DeclarationId),
    MissingDeclarationBinding(SyntaxId),

    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),

    MissingSourceFile(SourceFileId),
    MissingSourceFileContent { span: Span, length: usize },

    MissingSyntaxTree(SourceFileId),
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChildren(SyntaxPayload),

    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),

    UnimplementedFeature(SyntaxKind),
}

impl InternalError {
    /// Returns `true` if the internal error is [`ScopeBindingMissing`].
    ///
    /// [`ScopeBindingMissing`]: InternalError::ScopeBindingMissing
    #[must_use]
    pub fn is_scope_binding_missing(&self) -> bool {
        matches!(self, Self::MissingScopeBinding(..))
    }
}

pub trait AnnotatedError<'src> {
    type Context;

    fn annotated_error(&self, context: &Self::Context, reports: &mut Vec<Group<'src>>);
}
