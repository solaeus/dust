use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexMap, IndexSet, set::MutableValues};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::{CompileError, InternalError},
    constant_table::{ConstantId, ConstantTable},
    instruction::OperandType,
    native_function::NativeFunction,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxReader},
    prototype::Prototype,
    source::{Position, Source},
    r#type::{FunctionType, Type},
};

#[derive(Debug)]
pub struct Resolver {
    pub constants: ConstantTable,
    pub prototypes: Vec<Prototype>,

    declarations: IndexMap<DeclarationStorageKey, DeclarationStorageValue, FxBuildHasher>,
    declaration_members: Vec<DeclarationId>,
    declaration_types: HashMap<DeclarationId, TypeId, FxBuildHasher>,
    declaration_prototypes: HashMap<DeclarationId, u16, FxBuildHasher>,
    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,

    scopes: Vec<Scope>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,

    type_nodes: IndexSet<TypeNode, FxBuildHasher>,
    type_members: Vec<TypeId>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,

    next_inferred_type_id: InferredTypeId,
    next_anonymous_symbol_id: AnonymousSymbolId,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            constants: ConstantTable::new(),
            prototypes: Vec::new(),
            declarations: IndexMap::default(),
            declaration_members: Vec::new(),
            declaration_types: HashMap::default(),
            declaration_prototypes: HashMap::default(),
            declaration_bindings: HashMap::default(),
            scopes: Vec::new(),
            scope_bindings: HashMap::default(),
            type_nodes: IndexSet::default(),
            type_members: Vec::new(),
            type_bindings: HashMap::default(),
            next_inferred_type_id: InferredTypeId(0),
            next_anonymous_symbol_id: AnonymousSymbolId(0),
        };

        let _none_id = resolver.add_type(TypeNode::None);
        let _boolean_id = resolver.add_type(TypeNode::Boolean);
        let _byte_id = resolver.add_type(TypeNode::Byte);
        let _character_id = resolver.add_type(TypeNode::Character);
        let _float_id = resolver.add_type(TypeNode::Float);
        let _integer_id = resolver.add_type(TypeNode::Integer);
        let _string_id = resolver.add_type(TypeNode::String);

        debug_assert_eq!(_none_id, TypeId::NONE);
        debug_assert_eq!(_boolean_id, TypeId::BOOLEAN);
        debug_assert_eq!(_byte_id, TypeId::BYTE);
        debug_assert_eq!(_character_id, TypeId::CHARACTER);
        debug_assert_eq!(_float_id, TypeId::FLOAT);
        debug_assert_eq!(_integer_id, TypeId::INTEGER);
        debug_assert_eq!(_string_id, TypeId::STRING);

        let _project_scope_id = resolver.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        debug_assert_eq!(_project_scope_id, ScopeId::PROJECT);

        for native_function in NativeFunction::ALL {
            resolver.add_declaration(Declaration {
                symbol: Symbol::BuiltIn(native_function.name()),
                position: None,
                kind: DeclarationKind::Type { parent: None },
                scope_id: ScopeId::PROJECT,
                is_public: true,
            });
        }

        resolver
    }

    pub fn create_symbol(&mut self, path_segment: &SyntaxReader, source: &Source) -> Symbol {
        debug_assert_eq!(path_segment.kind(), SyntaxKind::PathSegment);

        let bytes = source
            .get_file(path_segment.file_id())
            .source_bytes(path_segment.span());
        let constant_id = self.constants.add_utf8(bytes);

        Symbol::Constant { constant_id }
    }

    pub fn create_anonymous_symbol(&mut self) -> Symbol {
        let id = self.next_anonymous_symbol_id;
        self.next_anonymous_symbol_id.0 += 1;

        Symbol::Anonymous(id)
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Result<&Scope, CompileError> {
        self.scopes
            .get(id.0 as usize)
            .ok_or(CompileError::Internal(InternalError::MissingScope(id)))
    }

    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.0 as usize)
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, CompileError> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(CompileError::Internal(InternalError::MissingScopeBinding(
                *syntax_id,
            )))
    }

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let parent = if let DeclarationKind::Type { parent } = declaration.kind {
            parent
        } else {
            None
        };
        let key = DeclarationStorageKey {
            symbol: declaration.symbol,
            parent,
        };

        if let Some((existing_index, _, _)) = self.declarations.get_full(&key) {
            return DeclarationId(existing_index as u32);
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);
        let value = DeclarationStorageValue {
            kind: declaration.kind,
            scope_id: declaration.scope_id,
            is_public: declaration.is_public,
            position: declaration.position,
        };

        self.declarations.insert(key, value);

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Result<Declaration, CompileError> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(key, value)| Declaration::from_key_and_value(key, value))
            .ok_or(CompileError::Internal(InternalError::MissingDeclaration(
                id,
            )))
    }

    pub fn add_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(
        &self,
        syntax_id: &SyntaxId,
    ) -> Result<&DeclarationId, CompileError> {
        self.declaration_bindings
            .get(syntax_id)
            .ok_or(CompileError::Internal(
                InternalError::MissingDeclarationBinding(*syntax_id),
            ))
    }

    pub fn set_declaration_type(&mut self, declaration_id: DeclarationId, type_id: TypeId) {
        self.declaration_types.insert(declaration_id, type_id);
    }

    pub fn get_declaration_type(
        &self,
        declaration_id: &DeclarationId,
    ) -> Result<&TypeId, CompileError> {
        self.declaration_types
            .get(declaration_id)
            .ok_or(CompileError::Internal(
                InternalError::MissingDeclarationType(*declaration_id),
            ))
    }

    pub fn set_declaration_prototype(
        &mut self,
        declaration_id: DeclarationId,
        prototype_index: u16,
    ) {
        self.declaration_prototypes
            .insert(declaration_id, prototype_index);
    }

    pub fn get_declaration_prototype(&self, declaration_id: &DeclarationId) -> Option<&u16> {
        self.declaration_prototypes.get(declaration_id)
    }

    pub fn add_declaration_members(
        &mut self,
        parameter_ids: &[DeclarationId],
    ) -> DeclarationMembers {
        let start = self.declaration_members.len() as u32;
        let count = parameter_ids.len() as u32;

        self.declaration_members.extend(parameter_ids);

        DeclarationMembers { start, count }
    }

    pub fn get_declaration_member(&self, index: u32) -> Result<&DeclarationId, CompileError> {
        self.declaration_members
            .get(index as usize)
            .ok_or(CompileError::Internal(
                InternalError::MissingDeclarationMember(index),
            ))
    }

    pub fn get_declaration_members(
        &self,
        members: DeclarationMembers,
    ) -> Result<&[DeclarationId], CompileError> {
        self.declaration_members
            .get(members.as_usize_range())
            .ok_or(CompileError::Internal(
                InternalError::MissingDeclarationMembers(members),
            ))
    }

    pub fn find_declaration_in_scope(
        &self,
        symbol: Symbol,
        path_segment: &SyntaxReader,
        target_scope_id: ScopeId,
        parent: Option<DeclarationId>,
        is_type_lookup: bool,
    ) -> Result<(DeclarationId, Declaration), CompileError> {
        debug_assert_eq!(path_segment.kind(), SyntaxKind::PathSegment);

        let mut current_scope_id = target_scope_id;
        let mut current_scope = self.get_scope(target_scope_id)?;

        loop {
            if (!is_type_lookup && current_scope.kind != ScopeKind::Block)
                || current_scope_id == ScopeId::PROJECT
            {
                break;
            }

            let key = DeclarationStorageKey { symbol, parent };

            if let Some((index, _, declaration_value)) = self.declarations.get_full(&key)
                && declaration_value.scope_id == current_scope_id
            {
                return Ok((
                    DeclarationId(index as u32),
                    Declaration::from_key_and_value(&key, declaration_value),
                ));
            }

            for import_id in &current_scope.imports {
                let import = self.get_declaration(*import_id)?;
                let key = DeclarationStorageKey { symbol, parent };

                if self.declarations.contains_key(&key) {
                    return Ok((*import_id, import));
                }
            }

            for module_id in &current_scope.modules {
                let module = self.get_declaration(*module_id)?;
                let key = DeclarationStorageKey { symbol, parent };

                if self.declarations.contains_key(&key) {
                    return Ok((*module_id, module));
                }
            }

            current_scope_id = current_scope.parent;
            current_scope = self.get_scope(current_scope_id)?;
        }

        if current_scope.kind == ScopeKind::Function {
            let key = DeclarationStorageKey { symbol, parent };

            if let Some((index, _, declaration)) = self.declarations.get_full(&key)
                && (matches!(declaration.kind, DeclarationKind::Type { .. })
                    || matches!(declaration.kind, DeclarationKind::Local { .. }))
                && declaration.scope_id == current_scope_id
            {
                return Ok((
                    DeclarationId(index as u32),
                    Declaration::from_key_and_value(&key, declaration),
                ));
            }
        }

        Err(CompileError::UndeclaredVariable {
            name: symbol,
            position: path_segment.position(),
        })
    }

    pub fn set_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, CompileError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(CompileError::Internal(InternalError::MissingTypeBinding(
                *syntax_id,
            )))
    }

    pub fn add_type_members(&mut self, types: &[TypeId]) -> TypeMembers {
        let members = TypeMembers {
            start: self.type_members.len() as u32,
            count: types.len() as u32,
        };

        self.type_members.extend_from_slice(types);

        members
    }

    pub fn get_type_members(&self, members: TypeMembers) -> Result<&[TypeId], CompileError> {
        self.type_members
            .get(members.as_usize_range())
            .ok_or(CompileError::Internal(InternalError::MissingTypeMembers(
                members,
            )))
    }

    pub fn get_type_member(&self, index: u32) -> Result<&TypeId, CompileError> {
        self.type_members
            .get(index as usize)
            .ok_or(CompileError::Internal(InternalError::MissingTypeMember(
                index,
            )))
    }

    pub fn add_external_type(&mut self, new_type: &Type) -> TypeId {
        let node = match new_type {
            Type::None => TypeNode::None,
            Type::Boolean => TypeNode::Boolean,
            Type::Byte => TypeNode::Byte,
            Type::Character => TypeNode::Character,
            Type::Float => TypeNode::Float,
            Type::Integer => TypeNode::Integer,
            Type::String => TypeNode::String,
            Type::List(element_type) => {
                let element_type = self.add_external_type(element_type);

                TypeNode::List { element_type }
            }
            Type::Function(function_type) => {
                let mut type_parameters = SmallVec::<[DeclarationId; 4]>::with_capacity(
                    function_type.type_parameters.len(),
                );

                for type_parameter_name in &function_type.type_parameters {
                    let name_id = self.constants.add_string(type_parameter_name);
                    let type_parameter_id = self.add_declaration(Declaration {
                        symbol: Symbol::Constant {
                            constant_id: name_id,
                        },
                        kind: DeclarationKind::Type { parent: None },
                        scope_id: ScopeId::PROJECT,
                        is_public: false,
                        position: None,
                    });
                    let type_parameter_type_id = self.create_inferred_type();

                    type_parameters.push(type_parameter_id);
                    self.set_declaration_type(type_parameter_id, type_parameter_type_id);
                }

                let mut value_parameter_types: SmallVec<[TypeId; 8]> =
                    SmallVec::with_capacity(function_type.value_parameters.len());

                for r#type in &function_type.value_parameters {
                    value_parameter_types.push(self.add_external_type(r#type));
                }

                TypeNode::Function {
                    type_parameters: self.add_declaration_members(&type_parameters),
                    value_parameters: self.add_type_members(&value_parameter_types),
                    return_type_id: self.add_external_type(&function_type.return_type),
                }
            }
            Type::Struct { name, fields } => {
                let name_id = self.constants.add_string(name);
                let struct_declaration_id = self.add_declaration(Declaration {
                    kind: DeclarationKind::Type { parent: None },
                    scope_id: ScopeId::PROJECT,
                    symbol: Symbol::Constant {
                        constant_id: name_id,
                    },
                    is_public: false,
                    position: None,
                });

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let name_id = self.constants.add_string(field_name);
                    let declaration_id = self.add_declaration(Declaration {
                        kind: DeclarationKind::Type {
                            parent: Some(struct_declaration_id),
                        },
                        scope_id: ScopeId::PROJECT,
                        symbol: Symbol::Constant {
                            constant_id: name_id,
                        },
                        is_public: false,
                        position: None,
                    });
                    let type_id = self.add_external_type(field_type);

                    field_declaration_ids.push(declaration_id);
                    self.set_declaration_type(declaration_id, type_id);
                }

                let fields = self.add_declaration_members(&field_declaration_ids);

                TypeNode::Struct {
                    declaration_id: struct_declaration_id,
                    generics: DeclarationMembers::default(),
                    fields,
                }
            }
        };

        self.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId, source: &Source) -> Result<Type, CompileError> {
        let type_node = self.get_type(id)?;

        match type_node {
            TypeNode::None => Ok(Type::None),
            TypeNode::Boolean => Ok(Type::Boolean),
            TypeNode::Byte => Ok(Type::Byte),
            TypeNode::Character => Ok(Type::Character),
            TypeNode::Float => Ok(Type::Float),
            TypeNode::Integer => Ok(Type::Integer),
            TypeNode::String => Ok(Type::String),
            TypeNode::List { element_type } => {
                let element_type = self.get_full_type(*element_type, source)?;

                Ok(Type::list(element_type))
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                let type_parameters = self
                    .get_declaration_member_names(*type_parameters)
                    .try_collect()?;
                let value_parameters = self
                    .get_type_members_as_full_types(*value_parameters, source)
                    .try_collect()?;
                let return_type = self.get_full_type(*return_type_id, source)?;

                Ok(Type::Function(Box::new(FunctionType {
                    type_parameters,
                    value_parameters,
                    return_type,
                })))
            }
            TypeNode::Inferred { resolved, .. } => {
                if let Some(resolved) = resolved {
                    self.get_full_type(*resolved, source)
                } else {
                    Err(CompileError::CannotInferType {
                        type_id: id,
                        position: None,
                    })
                }
            }
            TypeNode::Struct {
                declaration_id,
                fields,
                ..
            } => {
                let struct_declaration = self.get_declaration(*declaration_id)?;
                let name = struct_declaration
                    .symbol
                    .get_str(&self.constants)
                    .to_string();

                let fields = self.get_declaration_members(*fields)?;
                let mut field_types = Vec::with_capacity(fields.len());

                for field_id in fields {
                    let field_declaration = self.get_declaration(*field_id)?;
                    let field_name = field_declaration
                        .symbol
                        .get_str(&self.constants)
                        .to_string();

                    let field_type_id = self.get_declaration_type(field_id)?;
                    let field_type = self.get_full_type(*field_type_id, source)?;

                    field_types.push((field_name, field_type));
                }

                Ok(Type::Struct {
                    name,
                    fields: field_types,
                })
            }
            TypeNode::Enum { .. } => {
                todo!()
            }
        }
    }

    fn get_declaration_member_names(
        &self,
        members: DeclarationMembers,
    ) -> impl Iterator<Item = Result<String, CompileError>> {
        members.as_range().map(|member_index| {
            let declaration_id = self.get_declaration_member(member_index)?;
            let declaration = self.get_declaration(*declaration_id)?;
            let name = declaration.symbol.get_str(&self.constants).to_string();

            Ok(name)
        })
    }

    fn get_type_members_as_full_types(
        &self,
        members: TypeMembers,
        source: &Source,
    ) -> impl Iterator<Item = Result<Type, CompileError>> {
        members.as_range().map(|member_index| {
            let type_id = *self.get_type_member(member_index)?;

            self.get_full_type(type_id, source)
        })
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> TypeId {
        if let Some(existing) = self.type_nodes.get_index_of(&type_node) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.type_nodes.len() as u32);

        self.type_nodes.insert(type_node);

        type_id
    }

    pub fn get_type(&self, id: TypeId) -> Result<&TypeNode, CompileError> {
        self.type_nodes
            .get_index(id.0 as usize)
            .ok_or(CompileError::Internal(InternalError::MissingType(id)))
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Option<&mut TypeNode> {
        self.type_nodes.get_index_mut2(id.0 as usize)
    }

    pub fn get_operand_type(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<OperandType, CompileError> {
        let operand_type = match self.get_type(type_id)? {
            TypeNode::None => OperandType::NONE,
            TypeNode::Boolean => OperandType::BOOLEAN,
            TypeNode::Byte => OperandType::BYTE,
            TypeNode::Character => OperandType::CHARACTER,
            TypeNode::Float => OperandType::FLOAT,
            TypeNode::Integer => OperandType::INTEGER,
            TypeNode::String => OperandType::STRING,
            TypeNode::List { element_type } => match *element_type {
                TypeId::BOOLEAN => OperandType::LIST_BOOLEAN,
                TypeId::BYTE => OperandType::LIST_BYTE,
                TypeId::CHARACTER => OperandType::LIST_CHARACTER,
                TypeId::FLOAT => OperandType::LIST_FLOAT,
                TypeId::INTEGER => OperandType::LIST_INTEGER,
                TypeId::STRING => OperandType::LIST_STRING,
                _ => {
                    let element_operand_type = self.get_operand_type(*element_type, node)?;

                    match element_operand_type {
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => OperandType::LIST_LIST,
                        _ => {
                            return Err(CompileError::CannotInferType {
                                type_id,
                                position: Some(node.position()),
                            });
                        }
                    }
                }
            },
            TypeNode::Function { .. } => OperandType::FUNCTION,
            TypeNode::Struct { .. } => OperandType::COMPOUND,
            TypeNode::Inferred {
                resolved: Some(inferred),
                ..
            } => self.get_operand_type(*inferred, node)?,
            TypeNode::Inferred { resolved: None, .. } | TypeNode::Enum { .. } => {
                return Err(CompileError::CannotInferType {
                    type_id,
                    position: Some(node.position()),
                });
            }
        };

        Ok(operand_type)
    }

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            inferred_id: self.next_inferred_type_id,
            resolved: None,
        };

        self.next_inferred_type_id.0 += 1;

        self.add_type(inferred_type_node)
    }

    pub fn get_register_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<u16, CompileError> {
        match self.get_type(type_id)? {
            TypeNode::None => Err(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            }),
            TypeNode::Struct { fields, .. } => {
                let mut leaf_count: u32 = 0;

                for index in fields.start..(fields.start + fields.count) {
                    let field_declaration_id = self.get_declaration_member(index)?;
                    let field_type_id = *self.get_declaration_type(field_declaration_id)?;
                    let field_register_size = self.get_register_size(field_type_id, node)? as u32;

                    let mut resolved_field_type_id = field_type_id;

                    while let TypeNode::Inferred {
                        resolved: Some(resolved),
                        ..
                    } = self.get_type(resolved_field_type_id)?
                    {
                        resolved_field_type_id = *resolved;
                    }

                    let field_leaf_count = match self.get_type(resolved_field_type_id)? {
                        TypeNode::None => 0,
                        TypeNode::Struct { .. } => field_register_size.saturating_sub(1),
                        _ => 1,
                    };

                    leaf_count = leaf_count.saturating_add(field_leaf_count);
                }

                Ok(leaf_count as u16 + 1)
            }
            TypeNode::Inferred { resolved, .. } => match resolved {
                Some(resolved) => self.get_register_size(*resolved, node),
                None => Err(CompileError::CannotInferType {
                    type_id,
                    position: Some(node.position()),
                }),
            },
            _ => Ok(1),
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnonymousSymbolId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const PROJECT: Self = ScopeId(0);
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: ScopeId,
    pub imports: SmallVec<[DeclarationId; 4]>,
    pub modules: SmallVec<[DeclarationId; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    Block,
    Function,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(pub u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationMembers {
    pub start: u32,
    pub count: u32,
}

impl DeclarationMembers {
    fn as_range(&self) -> Range<u32> {
        let start = self.start;
        let end = start.saturating_add(self.count);

        Range { start, end }
    }

    fn as_usize_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start.saturating_add(self.count as usize);

        Range { start, end }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    pub symbol: Symbol,
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub is_public: bool,
    pub position: Option<Position>,
}

impl Declaration {
    fn from_key_and_value(key: &DeclarationStorageKey, value: &DeclarationStorageValue) -> Self {
        Self {
            symbol: key.symbol,
            position: value.position,
            kind: value.kind,
            scope_id: value.scope_id,
            is_public: value.is_public,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Anonymous(AnonymousSymbolId),
    BuiltIn(&'static str),
    Constant { constant_id: ConstantId },
}

impl Symbol {
    pub const MAIN: Self = Symbol::BuiltIn("main");

    pub fn is_anonymous(&self) -> bool {
        matches!(self, Symbol::Anonymous(_))
    }

    pub fn get_str<'a>(&self, constants: &'a ConstantTable) -> &'a str {
        match self {
            Symbol::Anonymous(_) => "<anonymous>",
            Symbol::BuiltIn(name) => name,
            Symbol::Constant { constant_id } => constants.get_string(*constant_id),
        }
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Symbol::Anonymous(id) => {
                hasher.write_u8(0);
                id.hash(hasher);
            }
            Symbol::BuiltIn(name) => {
                hasher.write_u8(1);
                name.hash(hasher);
            }
            Symbol::Constant { constant_id, .. } => {
                hasher.write_u8(2);
                constant_id.hash(hasher);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Local {
        shadowed: Option<DeclarationId>,
        is_mutable: bool,
    },
    Module {
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },
    Type {
        parent: Option<DeclarationId>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationStorageKey {
    symbol: Symbol,
    parent: Option<DeclarationId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationStorageValue {
    kind: DeclarationKind,
    scope_id: ScopeId,
    is_public: bool,
    position: Option<Position>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File,
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: Self = TypeId(0);
    pub const BOOLEAN: Self = TypeId(1);
    pub const BYTE: Self = TypeId(2);
    pub const CHARACTER: Self = TypeId(3);
    pub const FLOAT: Self = TypeId(4);
    pub const INTEGER: Self = TypeId(5);
    pub const STRING: Self = TypeId(6);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeMembers {
    pub start: u32,
    pub count: u32,
}

impl TypeMembers {
    fn as_range(&self) -> Range<u32> {
        let start = self.start;
        let end = start.saturating_add(self.count);

        Range { start, end }
    }

    fn as_usize_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start.saturating_add(self.count as usize);

        Range { start, end }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeNode {
    None,
    Boolean,
    Byte,
    Character,
    Float,
    Integer,
    String,
    List {
        element_type: TypeId,
    },
    Function {
        type_parameters: DeclarationMembers,
        value_parameters: TypeMembers,
        return_type_id: TypeId,
    },
    Struct {
        declaration_id: DeclarationId,
        generics: DeclarationMembers,
        fields: DeclarationMembers,
    },
    Enum {
        declaration_id: DeclarationId,
        generics: DeclarationMembers,
        variants: DeclarationMembers,
    },
    Inferred {
        inferred_id: InferredTypeId,
        resolved: Option<TypeId>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferredTypeId(u32);
