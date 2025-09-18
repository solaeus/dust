use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{NativeFunction, OperandType, Position, Type};

#[derive(Debug)]
pub struct Resolver {
    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,

    scopes: Vec<Scope>,

    types: IndexSet<TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,
}

impl Resolver {
    pub fn new(with_native_functions: bool) -> Self {
        let mut resolver = Self {
            declarations: IndexMap::default(),
            scopes: vec![Scope {
                kind: ScopeKind::Module,
                parent: ScopeId::MAIN,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            }],
            types: IndexSet::default(),
            type_members: Vec::new(),
        };

        let _main_function_declaration_id = resolver.add_declaration(
            DeclarationKind::Function,
            ScopeId::MAIN,
            TypeId::NONE,
            true,
            "main",
            Position::default(),
        );

        debug_assert_eq!(_main_function_declaration_id, DeclarationId::MAIN);

        if with_native_functions {
            let read_line_type_id = resolver.add_type(TypeNode::Function(FunctionTypeNode {
                type_parameters: (0, 0),
                value_parameters: (0, 0),
                return_type: TypeId::STRING,
            }));

            resolver.add_declaration(
                DeclarationKind::NativeFunction,
                ScopeId::GLOBAL,
                read_line_type_id,
                false,
                NativeFunction { index: 1 }.name(),
                Position::default(),
            );

            let value_parameters = resolver.push_type_members(&[TypeId::STRING]);
            let write_line_type_id = resolver.add_type(TypeNode::Function(FunctionTypeNode {
                type_parameters: (0, 0),
                value_parameters,
                return_type: TypeId::NONE,
            }));

            resolver.add_declaration(
                DeclarationKind::NativeFunction,
                ScopeId::GLOBAL,
                write_line_type_id,
                false,
                NativeFunction { index: 2 }.name(),
                Position::default(),
            );
        }

        #[cfg(test)]
        {
            while resolver.declarations.len() < 256 {
                let identifier = format!("__reserved_{}__", resolver.declarations.len());

                resolver.add_declaration(
                    DeclarationKind::NativeFunction,
                    ScopeId::GLOBAL,
                    TypeId::NONE,
                    false,
                    &identifier,
                    Position::default(),
                );
            }
        }

        resolver
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    pub fn mark_file_declaration_as_parsed(&mut self, id: DeclarationId) {
        if let Some(declaration) = self.declarations.get_index_mut(id.0 as usize)
            && let DeclarationKind::FileModule {
                inner_scope_id: _,
                is_parsed,
            } = &mut declaration.1.kind
        {
            *is_parsed = true;
        }
    }

    pub fn add_import_to_scope(&mut self, parent: ScopeId, child: DeclarationId) {
        if let Some(scope) = self.scopes.get_mut(parent.0 as usize) {
            scope.imports.push(child);
        }
    }

    pub fn add_module_to_scope(&mut self, parent: ScopeId, child: DeclarationId) {
        if let Some(scope) = self.scopes.get_mut(parent.0 as usize) {
            scope.modules.push(child);
        }
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);

        self.scopes.push(scope);

        id
    }

    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0 as usize)
    }

    pub fn get_declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_declaration(
        &mut self,
        kind: DeclarationKind,
        scope_id: ScopeId,
        type_id: TypeId,
        is_public: bool,
        identifier: &str,
        identifier_position: Position,
    ) -> DeclarationId {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        let declaration = Declaration {
            kind,
            scope_id,
            type_id,
            identifier_position,
            is_public,
        };
        let declaration_id = DeclarationId(self.declarations.len() as u32);
        let key = DeclarationKey(symbol, scope_id);

        self.declarations.insert(key, declaration);

        declaration_id
    }

    pub fn find_declarations(
        &self,
        identifier: &str,
    ) -> Option<SmallVec<[(DeclarationId, Declaration); 4]>> {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };
        let mut found = SmallVec::<[(DeclarationId, Declaration); 4]>::new();

        for (index, (DeclarationKey(found_symbol, _), declaration)) in
            self.declarations.iter().enumerate()
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
    ) -> Option<(DeclarationId, Declaration)> {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        let mut current_scope_id = target_scope_id;
        let mut current_scope = self.get_scope(current_scope_id)?;

        loop {
            let key = DeclarationKey(symbol, current_scope_id);

            if let Some((index, _, declaration)) = self.declarations.get_full(&key) {
                return Some((DeclarationId(index as u32), *declaration));
            }

            for import_id in &current_scope.imports {
                let import_declaration = self.get_declaration(*import_id)?;
                let key = DeclarationKey(symbol, import_declaration.scope_id);

                if self.declarations.contains_key(&key) {
                    return Some((*import_id, *import_declaration));
                }
            }

            for module_id in &current_scope.modules {
                let module_declaration = self.get_declaration(*module_id)?;
                let key = DeclarationKey(symbol, module_declaration.scope_id);

                if self.declarations.contains_key(&key) {
                    return Some((*module_id, *module_declaration));
                }
            }

            if current_scope.kind != ScopeKind::Block || current_scope_id == ScopeId(0) {
                break;
            }

            current_scope_id = current_scope.parent;
            current_scope = self.get_scope(current_scope_id)?;
        }

        None
    }

    pub fn resolve_type(&self, id: TypeId) -> Option<Type> {
        match id {
            TypeId::NONE => Some(Type::None),
            TypeId::BOOLEAN => Some(Type::Boolean),
            TypeId::BYTE => Some(Type::Byte),
            TypeId::CHARACTER => Some(Type::Character),
            TypeId::FLOAT => Some(Type::Float),
            TypeId::INTEGER => Some(Type::Integer),
            TypeId::STRING => Some(Type::String),
            TypeId(index) => {
                let type_node = self.types.get_index(index as usize)?;

                match type_node {
                    TypeNode::Array(element_type_id, size) => {
                        let element_type = self.resolve_type(*element_type_id)?;

                        Some(Type::array(element_type, *size as usize))
                    }
                    TypeNode::List(element_type_id) => {
                        let element_type = self.resolve_type(*element_type_id)?;

                        Some(Type::list(element_type))
                    }
                    TypeNode::Function(FunctionTypeNode {
                        type_parameters: (type_args_start, type_args_end),
                        value_parameters: (value_args_start, value_args_end),
                        return_type,
                    }) => {
                        let type_arguments = self.type_members
                            [*type_args_start as usize..*type_args_end as usize]
                            .iter()
                            .map(|id| self.resolve_type(*id))
                            .collect::<Option<Vec<Type>>>()?;
                        let value_arguments = self.type_members
                            [*value_args_start as usize..*value_args_end as usize]
                            .iter()
                            .map(|id| self.resolve_type(*id))
                            .collect::<Option<Vec<Type>>>()?;
                        let return_type = self.resolve_type(*return_type)?;

                        Some(Type::function(type_arguments, value_arguments, return_type))
                    }
                }
            }
        }
    }

    pub fn resolve_types(&self, start_index: u32, count: u32) -> Option<Vec<Type>> {
        (start_index..start_index + count)
            .map(|index| self.resolve_type(TypeId(index)))
            .collect()
    }

    pub fn add_type(&mut self, type_node: TypeNode) -> TypeId {
        if let Some(existing) = self.types.get_index_of(&type_node) {
            TypeId(existing as u32)
        } else {
            let id = TypeId(self.types.len() as u32);

            self.types.insert(type_node);

            id
        }
    }

    pub fn get_type_node(&self, id: TypeId) -> Option<&TypeNode> {
        self.types.get_index(id.0 as usize)
    }

    pub fn push_type_members(&mut self, members: &[TypeId]) -> (u32, u32) {
        let start = self.type_members.len() as u32;
        let count = members.len() as u32;

        self.type_members.extend_from_slice(members);

        (start, count)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const MAIN: Self = ScopeId(0);
    pub const GLOBAL: Self = ScopeId(u32::MAX);
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
    pub const ANONYMOUS: Self = DeclarationId(u32::MAX);
    pub const NATIVE: Self = DeclarationId(u32::MAX - 1);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeclarationKey(Symbol, ScopeId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub type_id: TypeId,
    pub identifier_position: Position,
    pub is_public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function,
    NativeFunction,
    Local {
        shadowed: Option<DeclarationId>,
    },
    LocalMutable {
        shadowed: Option<DeclarationId>,
    },
    InlineModule {
        inner_scope_id: ScopeId,
    },
    FileModule {
        inner_scope_id: ScopeId,
        is_parsed: bool,
    },
    Type,
}

impl DeclarationKind {
    pub fn is_local(&self) -> bool {
        matches!(
            self,
            DeclarationKind::Local { .. } | DeclarationKind::LocalMutable { .. }
        )
    }

    pub fn is_module(&self) -> bool {
        matches!(
            self,
            DeclarationKind::InlineModule { .. } | DeclarationKind::FileModule { .. }
        )
    }
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeclarationKind::Function => write!(f, "function"),
            DeclarationKind::NativeFunction => write!(f, "native function"),
            DeclarationKind::Local { .. } => write!(f, "local variable"),
            DeclarationKind::LocalMutable { .. } => write!(f, "mutable local variable"),
            DeclarationKind::InlineModule { .. } => write!(f, "inline module"),
            DeclarationKind::FileModule { .. } => write!(f, "file module"),
            DeclarationKind::Type => write!(f, "type"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: Self = TypeId(u32::MAX);
    pub const BOOLEAN: Self = TypeId(u32::MAX - 1);
    pub const BYTE: Self = TypeId(u32::MAX - 2);
    pub const CHARACTER: Self = TypeId(u32::MAX - 3);
    pub const FLOAT: Self = TypeId(u32::MAX - 4);
    pub const INTEGER: Self = TypeId(u32::MAX - 5);
    pub const STRING: Self = TypeId(u32::MAX - 6);

    pub fn as_operand_type(&self) -> OperandType {
        match *self {
            TypeId::BOOLEAN => OperandType::BOOLEAN,
            TypeId::BYTE => OperandType::BYTE,
            TypeId::CHARACTER => OperandType::CHARACTER,
            TypeId::FLOAT => OperandType::FLOAT,
            TypeId::INTEGER => OperandType::INTEGER,
            TypeId::STRING => OperandType::STRING,
            _ => OperandType::NONE,
        }
    }
}

impl Default for TypeId {
    fn default() -> Self {
        TypeId::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeNode {
    Array(TypeId, u32),
    List(TypeId),
    Function(FunctionTypeNode),
}

impl TypeNode {
    pub fn as_function(&self) -> Option<&FunctionTypeNode> {
        if let TypeNode::Function(function_type) = self {
            Some(function_type)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionTypeNode {
    pub type_parameters: (u32, u32),
    pub value_parameters: (u32, u32),
    pub return_type: TypeId,
}
