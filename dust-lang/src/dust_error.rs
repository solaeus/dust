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

/// An error that can occur during the interpretation of Dust code.
#[derive(Debug)]
pub enum DustError {
    Internal(InternalError),
    Source(SourceError),
    Parse(ParseError),
    Compile(CompileError),
    Multiple(Vec<DustError>),
}

impl DustError {
    pub fn push(&mut self, error: DustError) {
        match self {
            DustError::Multiple(errors) => errors.push(error),
            _ => {
                let previous_self =
                    std::mem::replace(self, DustError::Multiple(Vec::with_capacity(2)));

                self.push(previous_self);
                self.push(error);
            }
        }
    }

    pub fn write_reports(
        &self,
        writer: &mut impl Write,
        source: &Source,
        resolver: &Resolver,
    ) -> io::Result<()> {
        let mut groups = Vec::new();
        let report_start = groups.len();

        self.add_report((source, resolver), &mut groups);

        let report = Renderer::styled().render(&groups[report_start..]);

        writer.write(report.as_bytes())?;
        writer.write(b"\n")?;
        writer.flush()?;

        Ok(())
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
            DustError::Multiple(errors) => {
                for error in errors {
                    error.add_report(context, groups);
                }
            }
        }
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

pub trait AnnotatedError<'a> {
    type Context;

    fn add_report(&self, context: Self::Context, groups: &mut Vec<Group<'a>>);
}
