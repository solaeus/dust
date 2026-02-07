use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::{IndexMap, IndexSet, set::MutableValues};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    constant_table::{ConstantId, ConstantTable},
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
            parent: ScopeId::NONE,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        debug_assert_eq!(_project_scope_id, ScopeId::PROJECT);

        for native_function in NativeFunction::ALL {
            resolver.add_declaration(Declaration {
                symbol: Symbol::BuiltIn(native_function.name()),
                position: None,
                kind: DeclarationKind::Type { parent: None },
                scope_id: ScopeId::NONE,
                is_public: true,
            });
        }

        resolver
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

    pub fn add_declaration(&mut self, declaration: Declaration) -> DeclarationId {
        let parent = if let DeclarationKind::Type { parent } = declaration.kind {
            parent
        } else {
            None
        };
        let key = DeclarationStorageKey {
            symbol: declaration.symbol,
            scope_id: declaration.scope_id,
            parent,
        };

        if let Some((existing_index, _, _)) = self.declarations.get_full(&key) {
            return DeclarationId(existing_index as u32);
        }

        let declaration_id = DeclarationId(self.declarations.len() as u32);
        let value = DeclarationStorageValue {
            position: declaration.position,
            kind: declaration.kind,
            is_public: declaration.is_public,
        };

        self.declarations.insert(key, value);

        declaration_id
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Option<Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(key, value)| Declaration::from_key_and_value(key, value))
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

    pub fn add_declaration_members(
        &mut self,
        parameter_ids: &[DeclarationId],
    ) -> DeclarationMembers {
        let start = self.declaration_members.len() as u32;
        let count = parameter_ids.len() as u32;

        self.declaration_members.extend(parameter_ids);

        DeclarationMembers { start, count }
    }

    pub fn get_declaration_member(&self, index: u32) -> Option<DeclarationId> {
        self.declaration_members.get(index as usize).copied()
    }

    pub fn get_declaration_members(&self, range: DeclarationMembers) -> Option<&[DeclarationId]> {
        let range = range.start as usize..(range.start + range.count) as usize;

        self.declaration_members.get(range)
    }

    pub fn find_declaration_in_scope(
        &self,
        symbol: Symbol,
        target_scope_id: ScopeId,
        parent: Option<DeclarationId>,
    ) -> Option<(DeclarationId, Declaration)> {
        let mut current_scope_id = target_scope_id;
        let mut current_scope = self.get_scope(current_scope_id)?;

        loop {
            let key = DeclarationStorageKey {
                symbol,
                scope_id: current_scope_id,
                parent,
            };

            if let Some((index, _, declaration)) = self.declarations.get_full(&key) {
                return Some((
                    DeclarationId(index as u32),
                    Declaration::from_key_and_value(&key, declaration),
                ));
            }

            for import_id in &current_scope.imports {
                let import = self.get_declaration(*import_id)?;
                let key = DeclarationStorageKey {
                    symbol,
                    scope_id: import.scope_id,
                    parent,
                };

                if self.declarations.contains_key(&key) {
                    return Some((*import_id, import));
                }
            }

            for module_id in &current_scope.modules {
                let module = self.get_declaration(*module_id)?;
                let key = DeclarationStorageKey {
                    symbol,
                    scope_id: module.scope_id,
                    parent,
                };

                if self.declarations.contains_key(&key) {
                    return Some((*module_id, module));
                }
            }

            if current_scope.kind != ScopeKind::Block || current_scope_id == ScopeId::PROJECT {
                break;
            }

            current_scope_id = current_scope.parent;
            current_scope = self.get_scope(current_scope_id)?;
        }

        if current_scope.kind == ScopeKind::Function {
            let key = DeclarationStorageKey {
                symbol,
                scope_id: current_scope.parent,
                parent,
            };

            if let Some((index, _, declaration)) = self.declarations.get_full(&key)
                && matches!(declaration.kind, DeclarationKind::Type { .. })
            {
                return Some((
                    DeclarationId(index as u32),
                    Declaration::from_key_and_value(&key, declaration),
                ));
            }
        }

        None
    }

    pub fn set_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Option<&TypeId> {
        self.type_bindings.get(syntax_id)
    }

    pub fn add_type_members(&mut self, types: &[TypeId]) -> TypeMembers {
        let members = TypeMembers {
            start: self.type_members.len() as u32,
            count: types.len() as u32,
        };

        self.type_members.extend_from_slice(types);

        members
    }

    pub fn get_type_members(&self, members: TypeMembers) -> Option<&[TypeId]> {
        self.type_members.get(members.as_usize_range())
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
                    let name_id = self.constants.add_string(type_parameter_name.as_bytes());
                    let type_parameter_id = self.add_declaration(Declaration {
                        symbol: Symbol::External {
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
                let name_id = self.constants.add_string(name.as_bytes());
                let struct_declaration_id = self.add_declaration(Declaration {
                    kind: DeclarationKind::Type { parent: None },
                    scope_id: ScopeId::PROJECT,
                    symbol: Symbol::External {
                        constant_id: name_id,
                    },
                    is_public: false,
                    position: None,
                });

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let name_id = self.constants.add_string(field_name.as_bytes());
                    let declaration_id = self.add_declaration(Declaration {
                        kind: DeclarationKind::Type {
                            parent: Some(struct_declaration_id),
                        },
                        scope_id: ScopeId::PROJECT,
                        symbol: Symbol::External {
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
                let type_parameters = self
                    .get_declaration_members_as_full_types(*type_parameters, source)
                    .map(|(name, _)| name)
                    .collect();
                let value_parameters = self
                    .get_type_members_as_full_types(*value_parameters, source)
                    .collect();
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
                let struct_declaration = self.get_declaration(*declaration_id)?;
                let name = struct_declaration
                    .symbol
                    .get_str(&self.constants)?
                    .to_string();

                let start = fields.start as usize;
                let count = fields.count as usize;

                let mut fields = Vec::with_capacity(count);

                for index in start..(start + count) {
                    let field_declaration_id = *self.declaration_members.get(index)?;
                    let field_declaration = self.get_declaration(field_declaration_id)?;
                    let field_name = field_declaration
                        .symbol
                        .get_str(&self.constants)?
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

    fn get_declaration_members_as_full_types(
        &self,
        members: DeclarationMembers,
        source: &Source,
    ) -> impl Iterator<Item = (String, Type)> {
        self.get_declaration_members(members)
            .unwrap_or_default()
            .into_iter()
            .map_while(|declaration_id| {
                let declaration = self.get_declaration(*declaration_id)?;
                let name = declaration.symbol.get_str(&self.constants)?.to_string();
                let type_id = self.get_declaration_type(declaration_id)?;
                let full_type = self.get_full_type(*type_id, source)?;

                Some((name, full_type))
            })
    }

    fn get_type_members_as_full_types(
        &self,
        members: TypeMembers,
        source: &Source,
    ) -> impl Iterator<Item = Type> {
        self.get_type_members(members)
            .unwrap_or_default()
            .into_iter()
            .map_while(|type_id| {
                let full_type = self.get_full_type(*type_id, source)?;

                Some(full_type)
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

                for index in fields.start..(fields.start + fields.count) {
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
pub struct AnonymousSymbolId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const NONE: Self = ScopeId(u32::MAX);
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
            scope_id: key.scope_id,
            is_public: value.is_public,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Anonymous(AnonymousSymbolId),
    BuiltIn(&'static str),
    External {
        constant_id: ConstantId,
    },
    Source {
        constant_id: ConstantId,
        position: Position,
    },
}

impl Symbol {
    pub const MAIN: Self = Symbol::BuiltIn("main");

    pub fn get_str<'a>(&'a self, constants: &'a ConstantTable) -> Option<&'a str> {
        match self {
            Symbol::Anonymous(_) => None,
            Symbol::BuiltIn(name) => Some(name),
            Symbol::External { constant_id } | Symbol::Source { constant_id, .. } => {
                constants.get_string(*constant_id)
            }
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
            Symbol::External { constant_id } => {
                hasher.write_u8(2);
                constant_id.hash(hasher);
            }
            Symbol::Source {
                constant_id,
                position: _,
            } => {
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
    scope_id: ScopeId,
    parent: Option<DeclarationId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationStorageValue {
    kind: DeclarationKind,
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
    pub fn as_usize_range(&self) -> Range<usize> {
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
