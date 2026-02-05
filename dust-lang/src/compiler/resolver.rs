use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::{IndexMap, IndexSet, set::MutableValues};
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{
    compiler::CompileError,
    constant_table::ConstantTable,
    instruction::OperandType,
    native_function::NativeFunction,
    prototype::Prototype,
    source::{Position, Source},
    syntax::SyntaxId,
    r#type::{FunctionType, Type},
};

#[derive(Debug)]
pub struct Resolver {
    pub constants: ConstantTable,

    pub prototypes: Vec<Prototype>,

    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,

    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,

    declaration_members: IndexSet<DeclarationId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,

    declaration_types: HashMap<DeclarationId, TypeId, FxBuildHasher>,

    scopes: Vec<Scope>,

    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,

    type_nodes: IndexSet<TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,

    next_type_declaration_id: TypeDeclarationId,

    next_inferred_type_id: u32,

    next_anonymous_symbol_id: u32,
}

impl Resolver {
    pub fn new() -> Self {
        let mut context = Self {
            constants: ConstantTable::new(),
            prototypes: Vec::new(),
            type_bindings: HashMap::default(),
            declarations: IndexMap::default(),
            declaration_members: IndexSet::default(),
            declaration_bindings: HashMap::default(),
            declaration_types: HashMap::default(),
            scopes: vec![],
            scope_bindings: HashMap::default(),
            type_nodes: IndexSet::default(),
            type_members: Vec::new(),
            next_type_declaration_id: TypeDeclarationId(0),
            next_inferred_type_id: 0,
            next_anonymous_symbol_id: 0,
        };

        let _main_declaration_id = context.add_anonymous_declaration(Declaration::MAIN);

        debug_assert_eq!(_main_declaration_id, DeclarationId::MAIN);

        let _none_id = context.add_type(TypeNode::None);
        let _boolean_id = context.add_type(TypeNode::Boolean);
        let _byte_id = context.add_type(TypeNode::Byte);
        let _character_id = context.add_type(TypeNode::Character);
        let _float_id = context.add_type(TypeNode::Float);
        let _integer_id = context.add_type(TypeNode::Integer);
        let _string_id = context.add_type(TypeNode::String);

        debug_assert_eq!(_none_id, TypeId::NONE);
        debug_assert_eq!(_boolean_id, TypeId::BOOLEAN);
        debug_assert_eq!(_byte_id, TypeId::BYTE);
        debug_assert_eq!(_character_id, TypeId::CHARACTER);
        debug_assert_eq!(_float_id, TypeId::FLOAT);
        debug_assert_eq!(_integer_id, TypeId::INTEGER);
        debug_assert_eq!(_string_id, TypeId::STRING);

        let _project_scope_id = context.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let _native_scope_id = context.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        debug_assert_eq!(_project_scope_id, ScopeId::PROJECT);
        debug_assert_eq!(_native_scope_id, ScopeId::NATIVE);

        context.add_native_functions();

        context
    }

    pub fn add_native_functions(&mut self) {
        self.add_named_declaration(
            NativeFunction::NO_OP.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                name_position: None,
                is_public: true,
            },
        );
        self.add_named_declaration(
            NativeFunction::READ_LINE.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                name_position: None,
                is_public: true,
            },
        );
        self.add_named_declaration(
            NativeFunction::WRITE_LINE.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                name_position: None,
                is_public: true,
            },
        );
        self.add_named_declaration(
            NativeFunction::SPAWN.name(),
            Declaration {
                kind: DeclarationKind::NativeFunction,
                scope_id: ScopeId::NATIVE,
                name_position: None,
                is_public: true,
            },
        );
    }

    pub fn declarations(&self) -> &IndexMap<DeclarationKey, Declaration, FxBuildHasher> {
        &self.declarations
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0 as usize)
    }

    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.0 as usize)
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Option<&ScopeId> {
        self.scope_bindings.get(syntax_id)
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_named_declaration(&mut self, name: &str, declaration: Declaration) -> DeclarationId {
        let parent = if let DeclarationKind::Type { parent } = declaration.kind {
            parent
        } else {
            None
        };
        let key = DeclarationKey {
            symbol: Symbol::named(name),
            scope_id: declaration.scope_id,
            parent,
        };

        if let Some((existing_index, _, _)) = self.declarations.get_full(&key) {
            return DeclarationId(existing_index as u32);
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.insert(key, declaration);

        declaration_id
    }

    pub fn add_anonymous_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let symbol = Symbol::Anonymous {
            id: self.next_anonymous_symbol_id,
        };

        self.next_anonymous_symbol_id += 1;

        let parent = if let DeclarationKind::Type { parent } = declaration.kind {
            parent
        } else {
            None
        };
        let key = DeclarationKey {
            symbol,
            scope_id: declaration.scope_id,
            parent,
        };
        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.insert(key, declaration);

        declaration_id
    }

    pub fn get_declaration_mut(
        &mut self,
        declaration_id: &DeclarationId,
    ) -> Option<&mut Declaration> {
        self.declarations
            .get_index_mut(declaration_id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn set_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(&self, syntax_id: &SyntaxId) -> Option<&DeclarationId> {
        self.declaration_bindings.get(syntax_id)
    }

    pub fn set_declaration_type(&mut self, declaration_id: DeclarationId, type_id: TypeId) {
        self.declaration_types.insert(declaration_id, type_id);
    }

    pub fn get_declaration_type(&self, declaration_id: &DeclarationId) -> Option<&TypeId> {
        self.declaration_types.get(declaration_id)
    }

    pub fn set_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Option<&TypeId> {
        self.type_bindings.get(syntax_id)
    }

    pub fn add_declaration_members(&mut self, parameter_ids: &[DeclarationId]) -> (u32, u32) {
        let start = self.declaration_members.len() as u32;
        let count = parameter_ids.len() as u32;

        self.declaration_members.extend(parameter_ids);

        (start, count)
    }

    pub fn get_declaration_member(&self, index: u32) -> Option<DeclarationId> {
        self.declaration_members.get_index(index as usize).copied()
    }

    pub fn find_declarations(
        &self,
        identifier: &str,
    ) -> Option<SmallVec<[(DeclarationId, Declaration); 4]>> {
        let symbol = Symbol::named(identifier);
        let mut found = SmallVec::<[(DeclarationId, Declaration); 4]>::new();

        for (
            index,
            (
                DeclarationKey {
                    symbol: found_symbol,
                    ..
                },
                declaration,
            ),
        ) in self.declarations.iter().enumerate()
        {
            let declaration_id = DeclarationId(index as u32);

            if *found_symbol == symbol {
                found.push((declaration_id, *declaration));
            }
        }

        if found.is_empty() { None } else { Some(found) }
    }

    pub fn find_declaration_in_scope(
        &self,
        identifier: &str,
        target_scope_id: ScopeId,
        parent: Option<DeclarationId>,
    ) -> Option<(DeclarationId, Declaration)> {
        let symbol = Symbol::named(identifier);
        let mut current_scope_id = target_scope_id;
        let mut current_scope = self.get_scope(current_scope_id)?;

        loop {
            let key = DeclarationKey {
                symbol,
                scope_id: current_scope_id,
                parent,
            };

            if let Some((index, _, declaration)) = self.declarations.get_full(&key) {
                return Some((DeclarationId(index as u32), *declaration));
            }

            for import_id in &current_scope.imports {
                let import_declaration = self.get_declaration(*import_id)?;
                let key = DeclarationKey {
                    symbol,
                    scope_id: import_declaration.scope_id,
                    parent,
                };

                if self.declarations.contains_key(&key) {
                    return Some((*import_id, *import_declaration));
                }
            }

            for module_id in &current_scope.modules {
                let module_declaration = self.get_declaration(*module_id)?;
                let key = DeclarationKey {
                    symbol,
                    scope_id: module_declaration.scope_id,
                    parent,
                };

                if self.declarations.contains_key(&key) {
                    return Some((*module_id, *module_declaration));
                }
            }

            if current_scope.kind != ScopeKind::Block || current_scope_id == ScopeId::PROJECT {
                break;
            }

            current_scope_id = current_scope.parent;
            current_scope = self.get_scope(current_scope_id)?;
        }

        if current_scope.kind == ScopeKind::Function {
            let key = DeclarationKey {
                symbol,
                scope_id: current_scope.parent,
                parent,
            };

            if let Some((index, _, declaration)) = self.declarations.get_full(&key)
                && matches!(declaration.kind, DeclarationKind::Function { .. })
            {
                return Some((DeclarationId(index as u32), *declaration));
            }
        }

        None
    }

    pub fn add_full_type(&mut self, new_type: &Type) -> TypeId {
        let node = match new_type {
            Type::None => TypeNode::None,
            Type::Boolean => TypeNode::Boolean,
            Type::Byte => TypeNode::Byte,
            Type::Character => TypeNode::Character,
            Type::Float => TypeNode::Float,
            Type::Integer => TypeNode::Integer,
            Type::String => TypeNode::String,
            Type::List(element_type) => {
                let element_type = self.add_full_type(element_type);

                TypeNode::List { element_type }
            }
            Type::Function(function_type) => {
                let mut type_parameters =
                    SmallVec::<[TypeId; 4]>::with_capacity(function_type.type_parameters.len());

                for type_parameter in &function_type.type_parameters {
                    let type_parameter_id = self.add_full_type(type_parameter);

                    type_parameters.push(type_parameter_id);
                }

                let mut value_parameters: SmallVec<[TypeId; 4]> =
                    SmallVec::with_capacity(function_type.value_parameters.len());

                for value_parameter in &function_type.value_parameters {
                    let value_parameter_id = self.add_full_type(value_parameter);

                    value_parameters.push(value_parameter_id);
                }

                TypeNode::Function {
                    type_parameters: self.add_type_members(&type_parameters),
                    value_parameters: self.add_type_members(&value_parameters),
                    return_type_id: self.add_full_type(&function_type.return_type),
                }
            }
            Type::Struct { name, fields } => {
                let struct_declaration_id = self.add_named_declaration(
                    name,
                    Declaration {
                        kind: DeclarationKind::Type { parent: None },
                        scope_id: ScopeId::PROJECT,
                        name_position: None,
                        is_public: false,
                    },
                );

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let declaration_id = self.add_named_declaration(
                        field_name,
                        Declaration {
                            kind: DeclarationKind::Type {
                                parent: Some(struct_declaration_id),
                            },
                            scope_id: ScopeId::PROJECT,
                            name_position: None,
                            is_public: false,
                        },
                    );
                    let type_id = self.add_full_type(field_type);

                    field_declaration_ids.push(declaration_id);
                    self.set_declaration_type(declaration_id, type_id);
                }

                let fields = self.add_declaration_members(&field_declaration_ids);

                TypeNode::Struct {
                    declaration_id: struct_declaration_id,
                    generics: (0, 0),
                    fields,
                }
            }
        };

        self.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId, source: &Source) -> Option<Type> {
        let type_node = self.get_type(id)?;

        match type_node {
            TypeNode::None => Some(Type::None),
            TypeNode::Boolean => Some(Type::Boolean),
            TypeNode::Byte => Some(Type::Byte),
            TypeNode::Character => Some(Type::Character),
            TypeNode::Float => Some(Type::Float),
            TypeNode::Integer => Some(Type::Integer),
            TypeNode::String => Some(Type::String),
            TypeNode::List { element_type } => {
                let element_type = self.get_full_type(*element_type, source)?;

                Some(Type::list(element_type))
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                let type_parameters = self.get_type_members_as_full_types(
                    type_parameters.0,
                    type_parameters.1,
                    source,
                )?;
                let value_parameters = self.get_type_members_as_full_types(
                    value_parameters.0,
                    value_parameters.1,
                    source,
                )?;
                let return_type = self.get_full_type(*return_type_id, source)?;

                Some(Type::Function(Box::new(FunctionType {
                    type_parameters,
                    value_parameters,
                    return_type,
                })))
            }
            TypeNode::Inferred { resolved, .. } => {
                resolved.and_then(|resolved_id| self.get_full_type(resolved_id, source))
            }
            TypeNode::Struct {
                declaration_id,
                fields,
                ..
            } => {
                let declaration = self.get_declaration(*declaration_id)?;
                let name = declaration
                    .name_position
                    .and_then(|position| {
                        source
                            .get_file(position.file_id)
                            .map(|file| file.source_code.get_span(position.span))
                    })?
                    .to_string();

                let start = fields.0 as usize;
                let count = fields.1 as usize;

                let mut fields = Vec::with_capacity(count);

                for index in start..(start + count) {
                    let field_declaration_id = *self.declaration_members.get_index(index)?;
                    let field_declaration = self.get_declaration(field_declaration_id)?;
                    let field_name = field_declaration
                        .name_position
                        .and_then(|position| {
                            source
                                .get_file(position.file_id)
                                .map(|file| file.source_code.get_span(position.span))
                        })?
                        .to_string();

                    let field_type_id = self.get_declaration_type(&field_declaration_id)?;
                    let field_type = self.get_full_type(*field_type_id, source)?;

                    fields.push((field_name, field_type));
                }

                Some(Type::Struct { name, fields })
            }
            TypeNode::Enum { .. } => {
                todo!()
            }
        }
    }

    fn get_type_members_as_full_types(
        &self,
        start: u32,
        count: u32,
        source: &Source,
    ) -> Option<Vec<Type>> {
        Some(
            self.get_type_members(start, count)?
                .iter()
                .flat_map(|type_id| self.get_full_type(*type_id, source))
                .collect(),
        )
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> TypeId {
        if let Some(existing) = self.type_nodes.get_index_of(&type_node) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.type_nodes.len() as u32);

        self.type_nodes.insert(type_node);

        type_id
    }

    pub fn get_type(&self, id: TypeId) -> Option<&TypeNode> {
        self.type_nodes.get_index(id.0 as usize)
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> Option<&mut TypeNode> {
        self.type_nodes.get_index_mut2(id.0 as usize)
    }

    pub fn get_operand_type(&self, id: TypeId) -> Option<OperandType> {
        let operand_type = match self.get_type(id)? {
            TypeNode::None => OperandType::NONE,
            TypeNode::Boolean => OperandType::BOOLEAN,
            TypeNode::Byte => OperandType::BYTE,
            TypeNode::Character => OperandType::CHARACTER,
            TypeNode::Float => OperandType::FLOAT,
            TypeNode::Integer => OperandType::INTEGER,
            TypeNode::String => OperandType::STRING,
            TypeNode::List { element_type } => {
                let element_operand_type = self.get_operand_type(*element_type)?;

                match element_operand_type {
                    OperandType::BOOLEAN => OperandType::LIST_BOOLEAN,
                    OperandType::BYTE => OperandType::LIST_BYTE,
                    OperandType::CHARACTER => OperandType::LIST_CHARACTER,
                    OperandType::FLOAT => OperandType::LIST_FLOAT,
                    OperandType::INTEGER => OperandType::LIST_INTEGER,
                    OperandType::STRING => OperandType::LIST_STRING,
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING => OperandType::LIST_LIST,
                    _ => return None,
                }
            }
            TypeNode::Function { .. } => OperandType::FUNCTION,
            TypeNode::Struct { .. } => OperandType::COMPOUND,
            TypeNode::Inferred {
                resolved: Some(inferred),
                ..
            } => self.get_operand_type(*inferred)?,
            _ => return None,
        };

        Some(operand_type)
    }

    pub fn add_type_members(&mut self, members: &[TypeId]) -> (u32, u32) {
        let start = self.type_members.len() as u32;
        let count = members.len() as u32;

        self.type_members.extend_from_slice(members);

        (start, count)
    }

    pub fn get_type_members(&self, start_index: u32, count: u32) -> Option<&[TypeId]> {
        let range = start_index as usize..(start_index + count) as usize;

        self.type_members.get(range)
    }

    pub fn get_many_type_members<const COUNT: usize>(
        &self,
        members: [(u32, u32); COUNT],
    ) -> Option<[&[TypeId]; COUNT]> {
        let mut result: [&[TypeId]; COUNT] = [&[]; COUNT];

        for (i, (start_index, count)) in members.iter().enumerate() {
            result[i] = self.get_type_members(*start_index, *count)?;
        }

        Some(result)
    }

    pub fn create_type_declaration_id(&mut self) -> TypeDeclarationId {
        let id = self.next_type_declaration_id;

        self.next_type_declaration_id.0 += 1;

        id
    }

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            id: self.next_inferred_type_id,
            resolved: None,
        };

        self.next_inferred_type_id += 1;

        self.add_type(inferred_type_node)
    }

    pub fn infer_type(&mut self, type_id: TypeId) -> TypeId {
        if let Some(TypeNode::Inferred {
            resolved: Some(resolved),
            ..
        }) = self.get_type(type_id)
        {
            self.infer_type(*resolved)
        } else {
            type_id
        }
    }

    pub fn unify_types(&mut self, left: TypeId, right: TypeId) -> Result<bool, CompileError> {
        let left_inferred = self.infer_type(left);
        let right_inferred = self.infer_type(right);

        self.unify_inferred_types(left_inferred, right_inferred)
    }

    pub fn unify_inferred_types(
        &mut self,
        left: TypeId,
        right: TypeId,
    ) -> Result<bool, CompileError> {
        if left == right {
            return Ok(true);
        }

        let left_node = *self
            .get_type(left)
            .ok_or(CompileError::MissingType { type_id: left })?;
        let right_node = *self
            .get_type(right)
            .ok_or(CompileError::MissingType { type_id: right })?;

        match (left_node, right_node) {
            (TypeNode::Inferred { id, resolved: None }, _) => {
                if let Some(node) = self.get_type_mut(left) {
                    *node = TypeNode::Inferred {
                        id,
                        resolved: Some(right),
                    };
                }

                Ok(true)
            }
            (_, TypeNode::Inferred { id, resolved: None }) => {
                if let Some(node) = self.get_type_mut(right) {
                    *node = TypeNode::Inferred {
                        id,
                        resolved: Some(left),
                    };
                }

                Ok(true)
            }
            (
                TypeNode::List {
                    element_type: left_element_type,
                },
                TypeNode::List {
                    element_type: right_element_type,
                },
            ) => self.unify_types(left_element_type, right_element_type),
            (
                TypeNode::Function {
                    type_parameters: left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type,
                },
                TypeNode::Function {
                    type_parameters: right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type,
                },
            ) => {
                let mut unify_members =
                    |left: (u32, u32), right: (u32, u32)| -> Result<bool, CompileError> {
                        let left_members = self
                            .get_type_members(left.0, left.1)
                            .ok_or(CompileError::MissingTypeMembers {
                                start_index: right.0,
                                count: right.1,
                            })?
                            .to_vec();
                        let right_members = self
                            .get_type_members(right.0, right.1)
                            .ok_or(CompileError::MissingTypeMembers {
                                start_index: right.0,
                                count: right.1,
                            })?
                            .to_vec();

                        if left_members.len() != right_members.len() {
                            return Ok(false);
                        }

                        for (left_member, right_member) in
                            left_members.into_iter().zip(right_members.into_iter())
                        {
                            let unified = self.unify_types(left_member, right_member)?;

                            if !unified {
                                return Ok(false);
                            }
                        }

                        Ok(true)
                    };

                let unified = unify_members(left_type_parameters, right_type_parameters)?
                    && unify_members(left_value_parameters, right_value_parameters)?
                    && self.unify_types(left_return_type, right_return_type)?;

                Ok(unified)
            }
            (
                TypeNode::Struct {
                    declaration_id: left_declaration_id,
                    generics: left_generics,
                    fields: left_fields,
                },
                TypeNode::Struct {
                    declaration_id: right_declaration_id,
                    generics: right_generics,
                    fields: right_fields,
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    return Ok(false);
                }

                let mut unify_members =
                    |left: (u32, u32), right: (u32, u32)| -> Result<bool, CompileError> {
                        let left_members = self
                            .get_type_members(left.0, left.1)
                            .ok_or(CompileError::MissingTypeMembers {
                                start_index: right.0,
                                count: right.1,
                            })?
                            .to_vec();
                        let right_members = self
                            .get_type_members(right.0, right.1)
                            .ok_or(CompileError::MissingTypeMembers {
                                start_index: right.0,
                                count: right.1,
                            })?
                            .to_vec();

                        if left_members.len() != right_members.len() {
                            return Ok(false);
                        }

                        for (left_member, right_member) in
                            left_members.into_iter().zip(right_members.into_iter())
                        {
                            let unified = self.unify_types(left_member, right_member)?;

                            if !unified {
                                return Ok(false);
                            }
                        }

                        Ok(true)
                    };

                let unified = unify_members(left_generics, right_generics)?
                    && unify_members(left_fields, right_fields)?;

                Ok(unified)
            }
            (left, right) => Ok(left == right),
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Named { hash: u64 },
    Anonymous { id: u32 },
}

impl Symbol {
    pub fn named(name: &str) -> Self {
        let mut hasher = FxHasher::default();

        name.hash(&mut hasher);

        Symbol::Named {
            hash: hasher.finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const PROJECT: Self = ScopeId(0);
    pub const NATIVE: Self = ScopeId(1);
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

impl DeclarationId {
    pub const MAIN: Self = DeclarationId(0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationKey {
    symbol: Symbol,
    scope_id: ScopeId,
    parent: Option<DeclarationId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub name_position: Option<Position>,
    pub is_public: bool,
}

impl Declaration {
    pub const MAIN: Self = Declaration {
        kind: DeclarationKind::Function {
            parameters: (0, 0),
            prototype_index: Some(0),
        },
        scope_id: ScopeId::PROJECT,
        name_position: None,
        is_public: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Local {
        shadowed: Option<DeclarationId>,
        is_mutable: bool,
    },
    Function {
        parameters: (u32, u32),
        prototype_index: Option<u16>,
    },
    NativeFunction,
    Module {
        kind: ModuleKind,
        inner_scope_id: ScopeId,
    },
    Type {
        parent: Option<DeclarationId>,
    },
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeclarationKind::Function { .. } => write!(f, "function"),
            DeclarationKind::NativeFunction => write!(f, "native function"),
            DeclarationKind::Local { .. } => write!(f, "local variable"),
            DeclarationKind::Module { .. } => write!(f, "module"),
            DeclarationKind::Type { .. } => write!(f, "type"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeDeclarationId(pub u32);

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
        type_parameters: (u32, u32),
        value_parameters: (u32, u32),
        return_type_id: TypeId,
    },
    Inferred {
        id: u32,
        resolved: Option<TypeId>,
    },
    Struct {
        declaration_id: DeclarationId,
        generics: (u32, u32),
        fields: (u32, u32),
    },
    Enum {
        id: DeclarationId,
        generics: (u32, u32),
        variants: (u32, u32),
    },
}
