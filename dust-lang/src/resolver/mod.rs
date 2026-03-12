pub mod declaration_graph;
pub mod error;
pub mod scope_graph;
pub mod symbol_table;
pub mod type_graph;

use std::collections::{HashMap, HashSet};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    dust_type::DustType,
    resolver::{
        declaration_graph::{
            Declaration, DeclarationGraph, DeclarationId, DeclarationMembers, Definition,
            Visibility,
        },
        error::ResolverError,
        scope_graph::{ScopeGraph, ScopeId},
        symbol_table::{SymbolId, SymbolTable},
        type_graph::{
            FloatType, SignedIntegerType, Type, TypeGraph, TypeId, TypeMembers, UnsignedIntegerType,
        },
    },
    source::Source,
    syntax::{SyntaxId, reader::SyntaxReader},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: SymbolTable,
    pub declarations: DeclarationGraph,
    pub scopes: ScopeGraph,
    pub types: TypeGraph,

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            symbols: SymbolTable::new(),
            declarations: DeclarationGraph::new(),
            scopes: ScopeGraph::new(),
            types: TypeGraph::new(),
            scope_search: HashSet::default(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
        };

        // resolver.add_core();

        resolver
    }

    fn add_core(&mut self) {
        todo!()
    }

    pub fn add_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(
        &self,
        syntax_id: &SyntaxId,
    ) -> Result<&DeclarationId, ResolverError> {
        self.declaration_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingDeclarationBinding(*syntax_id))
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, ResolverError> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingScopeBinding(*syntax_id))
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, ResolverError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(ResolverError::MissingTypeBinding(*syntax_id))
    }

    pub fn get_byte_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<usize, CompileError> {
        let r#type = self.types.get_type(type_id)?;

        match r#type {
            Type::Never => Ok(0),
            Type::Boolean
            | Type::SignedInteger(SignedIntegerType::I8)
            | Type::UnsignedInteger(UnsignedIntegerType::U8) => Ok(1),
            Type::SignedInteger(SignedIntegerType::I16)
            | Type::UnsignedInteger(UnsignedIntegerType::U16) => Ok(2),
            Type::Character
            | Type::SignedInteger(SignedIntegerType::I32)
            | Type::UnsignedInteger(UnsignedIntegerType::U32)
            | Type::Float(FloatType::F32)
            | Type::FunctionDefinition { .. }
            | Type::Closure { .. }
            | Type::Function { .. } => Ok(4),
            Type::Slice { .. } | Type::Heap { .. } => Ok(8),
            Type::SignedInteger(SignedIntegerType::I64)
            | Type::UnsignedInteger(UnsignedIntegerType::U64)
            | Type::Float(FloatType::F64) => Ok(8),
            Type::SignedInteger(SignedIntegerType::I128)
            | Type::UnsignedInteger(UnsignedIntegerType::U128) => Ok(16),
            Type::Tuple { element_type_ids } => {
                let type_ids = self.types.get_type_members(*element_type_ids)?;
                let mut total_size = 0;

                for type_id in type_ids {
                    total_size += self.get_byte_size(*type_id, node)?;
                }

                Ok(total_size)
            }
            Type::Array {
                element_type_id,
                length,
            } => {
                let element_size = self.get_byte_size(*element_type_id, node)?;

                Ok(element_size * (*length))
            }
            Type::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id)?;
                let mut type_arguments = self
                    .types
                    .get_type_members(*type_arguments)?
                    .iter()
                    .rev()
                    .copied()
                    .collect::<SmallVec<[TypeId; 4]>>();

                match &declaration.definition {
                    Definition::StructType { fields, .. } => {
                        let field_declaration_ids =
                            self.declarations.get_declaration_members(fields)?;
                        let mut total_size = 0;

                        for field_declaration_id in field_declaration_ids {
                            let field_type_id = self
                                .declarations
                                .get_declaration_type(*field_declaration_id)?;
                            let field_type = self.types.get_type(field_type_id)?;

                            if let Type::Generic { .. } = field_type {
                                let type_argument_id =
                                    type_arguments.pop().ok_or(CompileError::Resolver(
                                        ResolverError::MissingTypeArgument(*declaration_id),
                                    ))?;

                                total_size += self.get_byte_size(type_argument_id, node)?;
                            } else {
                                total_size += self.get_byte_size(field_type_id, node)?;
                            }
                        }

                        Ok(total_size)
                    }
                    _ => Err(CompileError::Resolver(
                        ResolverError::MissingAlgebraicTypeDeclaration(*declaration_id),
                    )),
                }
            }
            _ => todo!(),
        }
    }

    pub fn get_register_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<usize, CompileError> {
        self.get_byte_size(type_id, node)
            .map(|byte_size| byte_size.div_ceil(4))
    }

    pub fn find_declaration_in_scope(
        &mut self,
        symbol_id: SymbolId,
        target_scope_id: ScopeId,
        visibility: Visibility,
        path_segment: &SyntaxReader,
    ) -> Result<(DeclarationId, &Declaration), CompileError> {
        let mut current_scope_id = target_scope_id;

        loop {
            if current_scope_id == ScopeId::NONE || !self.scope_search.insert(current_scope_id) {
                break;
            }

            if let Some((declaration_id, declaration)) =
                self.declarations
                    .find_declaration(symbol_id, current_scope_id, visibility)
            {
                self.scope_search.clear();

                return Ok((declaration_id, declaration));
            }

            let current_scope = self.scopes.get_scope(current_scope_id)?;

            for module_scope_id in &current_scope.modules {
                if let Some((declaration_id, declaration)) =
                    self.declarations
                        .find_declaration(symbol_id, *module_scope_id, visibility)
                {
                    self.scope_search.clear();

                    return Ok((declaration_id, declaration));
                }
            }

            for import_declaration_id in &current_scope.imports {
                let import_declaration =
                    self.declarations.get_declaration(*import_declaration_id)?;

                if import_declaration.symbol_id == symbol_id {
                    self.scope_search.clear();

                    return Ok((*import_declaration_id, import_declaration));
                }
            }

            current_scope_id = current_scope.parent;
        }

        self.scope_search.clear();

        Err(CompileError::Undeclared {
            symbol_id,
            usage_position: path_segment.position(),
        })
    }

    pub fn add_external_type(&mut self, new_type: &DustType) -> TypeId {
        todo!()
    }

    pub fn get_external_type(&self, id: TypeId, source: &Source) -> Result<DustType, CompileError> {
        todo!()
    }

    fn get_declaration_member_names(
        &self,
        members: DeclarationMembers,
    ) -> impl Iterator<Item = Result<String, CompileError>> {
        members.as_range().map(|member_index| {
            let declaration_id = self.declarations.get_declaration_member(member_index)?;
            let declaration = self.declarations.get_declaration(*declaration_id)?;
            let name = self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

            Ok(name)
        })
    }

    fn get_type_members_as_full_types(
        &self,
        members: TypeMembers,
        source: &Source,
    ) -> impl Iterator<Item = Result<DustType, CompileError>> {
        members.as_range().map(|member_index| {
            let type_id = *self.types.get_type_member(member_index)?;

            self.get_external_type(type_id, source)
        })
    }

    pub fn declaration_display_iterator<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<String, CompileError>> + 'a {
        self.declarations.iter().map(|(id, declaration)| todo!())
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
