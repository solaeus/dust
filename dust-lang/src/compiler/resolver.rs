use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use indexmap::{
    IndexMap, IndexSet,
    set::{MutableValues, Slice},
};
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{
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

    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,
    declaration_members: IndexSet<DeclarationId, FxBuildHasher>,
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
            declaration_members: IndexSet::default(),
            declaration_types: HashMap::default(),
            declaration_prototypes: HashMap::default(),
            declaration_bindings: HashMap::default(),
            scopes: vec![],
            scope_bindings: HashMap::default(),
            type_nodes: IndexSet::default(),
            type_members: Vec::new(),
            type_bindings: HashMap::default(),
            next_inferred_type_id: InferredTypeId(0),
            next_anonymous_symbol_id: AnonymousSymbolId(0),
        };

        let _main_declaration_id = resolver.add_anonymous_declaration(Declaration {
            kind: DeclarationKind::Function { parameters: (0, 0) },
            scope_id: ScopeId::PROJECT,
            name: DeclarationName::BuiltIn("main"),
            is_public: false,
        });

        debug_assert_eq!(_main_declaration_id, DeclarationId::MAIN);

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
        let _native_scope_id = resolver.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        debug_assert_eq!(_project_scope_id, ScopeId::PROJECT);
        debug_assert_eq!(_native_scope_id, ScopeId::NATIVE);

        resolver.add_native_functions();

        resolver
    }

    pub fn add_native_functions(&mut self) {
        for native_function in NativeFunction::ALL {
            self.add_named_declaration(
                native_function.name(),
                Declaration {
                    kind: DeclarationKind::NativeFunction,
                    scope_id: ScopeId::NATIVE,
                    name: DeclarationName::BuiltIn(native_function.name()),
                    is_public: true,
                },
            );
        }
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

        self.next_anonymous_symbol_id.0 += 1;

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

    pub fn get_declaration_members(
        &self,
        start_index: u32,
        count: u32,
    ) -> Option<&Slice<DeclarationId>> {
        let range = start_index as usize..(start_index + count) as usize;

        self.declaration_members.get_range(range)
    }

    pub fn get_declaration_name<'a>(
        &'a self,
        name: &DeclarationName,
        source: &'a Source,
    ) -> Option<&'a str> {
        match name {
            DeclarationName::BuiltIn(name) => Some(name),
            DeclarationName::Source(position) => Some(source.get_source_str(*position)),
            DeclarationName::External(id) => self.constants.get_string(*id),
        }
    }

    pub fn find_declarations(
        &self,
        identifier: &str,
    ) -> SmallVec<[(DeclarationId, Declaration); 4]> {
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
            if *found_symbol == symbol {
                let declaration_id = DeclarationId(index as u32);

                found.push((declaration_id, *declaration));
            }
        }

        found
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
                let name_id = self.constants.add_string(name.as_bytes());
                let struct_declaration_id = self.add_named_declaration(
                    name,
                    Declaration {
                        kind: DeclarationKind::Type { parent: None },
                        scope_id: ScopeId::PROJECT,
                        name: DeclarationName::External(name_id),
                        is_public: false,
                    },
                );

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let name_id = self.constants.add_string(field_name.as_bytes());
                    let declaration_id = self.add_named_declaration(
                        field_name,
                        Declaration {
                            kind: DeclarationKind::Type {
                                parent: Some(struct_declaration_id),
                            },
                            scope_id: ScopeId::PROJECT,
                            name: DeclarationName::External(name_id),
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
                let name = self
                    .get_declaration_name(&declaration.name, source)?
                    .to_string();

                let start = fields.0 as usize;
                let count = fields.1 as usize;

                let mut fields = Vec::with_capacity(count);

                for index in start..(start + count) {
                    let field_declaration_id = *self.declaration_members.get_index(index)?;
                    let field_declaration = self.get_declaration(field_declaration_id)?;
                    let field_name = self
                        .get_declaration_name(&field_declaration.name, source)?
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

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            inferred_id: self.next_inferred_type_id,
            resolved: None,
        };

        self.next_inferred_type_id.0 += 1;

        self.add_type(inferred_type_node)
    }

    pub fn get_register_size(&self, type_id: TypeId) -> Option<u16> {
        match self.get_type(type_id)? {
            TypeNode::None => Some(0),
            TypeNode::Struct { fields, .. } => {
                let mut leaf_count: u32 = 0;

                for index in fields.0..(fields.0 + fields.1) {
                    let field_declaration_id = self.get_declaration_member(index)?;
                    let field_type_id = *self.get_declaration_type(&field_declaration_id)?;
                    let field_register_size = self.get_register_size(field_type_id)? as u32;

                    let mut resolved_field_type_id = field_type_id;

                    while let Some(TypeNode::Inferred {
                        resolved: Some(resolved),
                        ..
                    }) = self.get_type(resolved_field_type_id)
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

                Some(leaf_count as u16 + 1)
            }
            TypeNode::Inferred { resolved, .. } => match resolved {
                Some(resolved) => self.get_register_size(*resolved),
                None => None,
            },
            _ => Some(1),
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
    Anonymous { id: AnonymousSymbolId },
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
pub struct AnonymousSymbolId(u32);

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
    pub name: DeclarationName,
    pub is_public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationName {
    BuiltIn(&'static str),
    Source(Position),
    External(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Local {
        shadowed: Option<DeclarationId>,
        is_mutable: bool,
    },
    Function {
        parameters: (u32, u32),
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
    Struct {
        declaration_id: DeclarationId,
        generics: (u32, u32),
        fields: (u32, u32),
    },
    Enum {
        declaration_id: DeclarationId,
        generics: (u32, u32),
        variants: (u32, u32),
    },
    Inferred {
        inferred_id: InferredTypeId,
        resolved: Option<TypeId>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferredTypeId(u32);
