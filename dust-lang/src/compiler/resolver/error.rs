use crate::{
    error::AnnotatedError,
    compiler::resolver::{
        declarations::{DeclarationId, DeclarationMembers},
        scopes::ScopeId,
        symbols::SymbolId,
        types::{TypeId, TypeMembers},
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
    MissingTypeDeclaration(DeclarationId),
    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),
    MissingType(TypeId),
    MissingTypeMember(u32),
    MissingTypeMembers(TypeMembers),
    MissingTypeBinding(SyntaxId),
    MissingFunctionDeclaration(DeclarationId),
    ExpectedFieldDeclaration(DeclarationId),
    MissingAlgebraicTypeDeclaration(DeclarationId),
    MissingTypeArgument(DeclarationId),
    ExpectedConcreteType,
    ExpectedVariantDeclaration(DeclarationId),
}

impl<'a> AnnotatedError<'a> for ResolverError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        self.add_internal_report(groups);
    }
}
