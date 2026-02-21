//! Top-level error for the Dust language API that can create detailed reports with source code
//! annotations.
use std::io::{self, Write};

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
pub struct DustErrors {
    errors: Vec<DustError>,
}

impl DustErrors {
    pub fn new(errors: Vec<DustError>) -> Self {
        Self { errors }
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[DustError] {
        &self.errors
    }

    pub fn write_reports<'src>(
        &self,
        writer: &mut impl Write,
        source: &Source<'src>,
        resolver: &Resolver,
    ) -> io::Result<()> {
        let renderer = Renderer::styled();
        let mut groups = Vec::new();

        for error in &self.errors {
            let report_start = groups.len();

            match error {
                DustError::Internal(internal_error) => {
                    internal_error.annotated_error((), &mut groups)
                }
                DustError::Source(source_error) => source_error.annotated_error((), &mut groups),
                DustError::Parse(parse_error) => parse_error.annotated_error(source, &mut groups),
                DustError::Compile(compile_error) => {
                    compile_error.annotated_error((source, resolver), &mut groups)
                }
            }

            let report = renderer.render(&groups[report_start..]);

            writeln!(writer, "{report}")?;
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
    MissingSyntaxChild { total_children: usize },
    MissingSyntaxChildren(SyntaxPayload),

    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),

    UnimplementedFeature(SyntaxKind),
}

impl<'a> AnnotatedError<'a> for InternalError {
    type Context = ();

    fn annotated_error(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        let title = "Internal error".to_string();
        let message = format!("{self:#?}");
        let help = "This is a bug. 🐛 If this is a released version of Dust, please report this to the developers.".to_string();
        let group = Group::with_title(Level::ERROR.primary_title(title))
            .element(Level::ERROR.message(message))
            .element(Level::NOTE.message(help));

        groups.push(group);
    }
}

pub trait AnnotatedError<'a> {
    type Context;

    fn annotated_error(&self, context: Self::Context, groups: &mut Vec<Group<'a>>);
}
