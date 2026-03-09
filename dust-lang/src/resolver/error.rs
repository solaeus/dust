use crate::{
    error::AnnotatedError,
    resolver::{
        declaration_graph::{DeclarationId, DeclarationMembers},
        scope_graph::ScopeId,
        symbol_table::SymbolId,
        type_graph::{TypeId, TypeMembers, TypeNode},
    },
    syntax::SyntaxId,
};

#[derive(Debug)]
pub enum ResolverError {
    MissingSymbol(SymbolId),
    MissingDeclaration(DeclarationId),
    MissingDeclarationMember(u32),
    MissingDeclarationMembers(DeclarationMembers),
    MissingDeclarationType(DeclarationId),
    MissingDeclarationBinding(SyntaxId),
    ExpectedTypeDeclaration(DeclarationId),
    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),
    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),
    ExpectedListType { found: TypeNode },
}

impl<'a> AnnotatedError<'a> for ResolverError {
    type Context = ();

    fn is_internal(&self) -> bool {
        true
    }

    fn add_report(&self, _: Self::Context, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        self.add_internal_report(groups);
    }
}
