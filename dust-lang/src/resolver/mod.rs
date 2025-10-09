use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{
    instruction::OperandType,
    source::Position,
    syntax_tree::SyntaxId,
    r#type::{FunctionType, Type},
};

#[derive(Debug)]
pub struct Resolver {
    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,

    scopes: Vec<Scope>,

    types: IndexMap<TypeKey, TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            declarations: IndexMap::default(),
            scopes: vec![],
            types: IndexMap::default(),
            type_members: Vec::new(),
        };

        let _main_scope_id = resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: ScopeId::MAIN,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let _main_function_declaration_id = resolver.add_declaration(
            "main",
            Declaration {
                kind: DeclarationKind::Function {
                    inner_scope_id: ScopeId::MAIN,
                    syntax_id: SyntaxId(0),
                },
                scope_id: ScopeId::MAIN,
                type_id: TypeId::NONE,
                position: Position::default(),
                is_public: true,
            },
        );

        debug_assert_eq!(_main_function_declaration_id, DeclarationId::MAIN);

        resolver
    }

    pub fn declarations(&self) -> &IndexMap<DeclarationKey, Declaration, FxBuildHasher> {
        &self.declarations
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
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

    pub fn get_declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_declaration(&mut self, identifier: &str, declaration: Declaration) -> DeclarationId {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        let declaration_id = DeclarationId(self.declarations.len() as u32);
        let key = DeclarationKey(symbol, declaration.scope_id);

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

    pub fn add_type(&mut self, new_type: &Type) -> TypeId {
        let type_key = {
            let mut hasher = FxHasher::default();

            new_type.hash(&mut hasher);

            TypeKey {
                hash: hasher.finish(),
            }
        };

        if let Some(existing) = self.types.get_index_of(&type_key) {
            return TypeId(existing as u32);
        }

        match new_type {
            Type::None => TypeId::NONE,
            Type::Boolean => TypeId::BOOLEAN,
            Type::Byte => TypeId::BYTE,
            Type::Character => TypeId::CHARACTER,
            Type::Float => TypeId::FLOAT,
            Type::Integer => TypeId::INTEGER,
            Type::String => TypeId::STRING,
            Type::Array(element_type, size) => {
                let element_type_id = self.add_type(element_type);
                let type_node = TypeNode::Array(element_type_id, *size as u32);
                let type_id = TypeId(self.types.len() as u32);

                self.types.insert(type_key, type_node);

                type_id
            }
            Type::List(element_type) => {
                let element_type_id = self.add_type(element_type);
                let type_node = TypeNode::List(element_type_id);
                let type_id = TypeId(self.types.len() as u32);

                self.types.insert(type_key, type_node);

                type_id
            }
            Type::Function(function_type) => self.add_function_type(function_type),
        }
    }

    pub fn add_function_type(&mut self, function_type: &FunctionType) -> TypeId {
        let mut type_parameters: SmallVec<[TypeId; 4]> =
            SmallVec::with_capacity(function_type.type_parameters.len());

        for type_parameter in &function_type.type_parameters {
            let type_parameter_id = self.add_type(type_parameter);

            type_parameters.push(type_parameter_id);
        }

        let mut value_parameters: SmallVec<[TypeId; 4]> =
            SmallVec::with_capacity(function_type.value_parameters.len());

        for value_parameter in &function_type.value_parameters {
            let value_parameter_id = self.add_type(value_parameter);

            value_parameters.push(value_parameter_id);
        }

        let type_parameters = self.add_type_members(&type_parameters);
        let value_parameters = self.add_type_members(&value_parameters);
        let return_type_id = self.add_type(&function_type.return_type);
        let function_type_node = FunctionTypeNode {
            type_parameters,
            value_parameters,
            return_type: return_type_id,
        };
        let type_node = TypeNode::Function(function_type_node);

        self.add_type_node(type_node)
    }

    pub fn add_type_node(&mut self, type_node: TypeNode) -> TypeId {
        let type_key = {
            let mut hasher = FxHasher::default();

            type_node.hash(&mut hasher);

            TypeKey {
                hash: hasher.finish(),
            }
        };

        if let Some(existing) = self.types.get_index_of(&type_key) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.types.len() as u32);

        self.types.insert(type_key, type_node);

        type_id
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
                let (_, type_node) = self.types.get_index(index as usize)?;

                match type_node {
                    TypeNode::Array(element_type_id, size) => {
                        let element_type = self.resolve_type(*element_type_id)?;

                        Some(Type::array(element_type, *size as usize))
                    }
                    TypeNode::List(element_type_id) => {
                        let element_type = self.resolve_type(*element_type_id)?;

                        Some(Type::list(element_type))
                    }
                    TypeNode::Function(function_type_node) => {
                        let function_type = self.resolve_function_type(function_type_node)?;

                        Some(Type::Function(Box::new(function_type)))
                    }
                }
            }
        }
    }

    pub fn resolve_function_type(&self, function_type: &FunctionTypeNode) -> Option<FunctionType> {
        let type_parameters = self
            .resolve_type_members(
                function_type.type_parameters.0,
                function_type.type_parameters.1,
            )
            .try_collect::<Vec<Type>>()?;
        let value_parameters = self
            .resolve_type_members(
                function_type.value_parameters.0,
                function_type.value_parameters.1,
            )
            .try_collect::<Vec<Type>>()?;
        let return_type = self.resolve_type(function_type.return_type)?;

        Some(FunctionType {
            type_parameters,
            value_parameters,
            return_type,
        })
    }

    pub fn get_type_node(&self, id: TypeId) -> Option<&TypeNode> {
        self.types
            .get_index(id.0 as usize)
            .map(|(_, type_node)| type_node)
    }

    pub fn add_type_members(&mut self, members: &[TypeId]) -> (u32, u32) {
        let start = self.type_members.len() as u32;
        let count = members.len() as u32;

        self.type_members.extend_from_slice(members);

        (start, count)
    }

    fn resolve_type_members(
        &self,
        start_index: u32,
        count: u32,
    ) -> impl Iterator<Item = Option<Type>> {
        let range = start_index as usize..(start_index + count) as usize;

        self.type_members[range]
            .iter()
            .map(|type_id| self.resolve_type(*type_id))
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
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

impl Default for DeclarationId {
    fn default() -> Self {
        DeclarationId::ANONYMOUS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationKey(Symbol, ScopeId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub type_id: TypeId,
    pub position: Position,
    pub is_public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function {
        inner_scope_id: ScopeId,
        syntax_id: SyntaxId,
    },
    NativeFunction,
    Local {
        shadowed: Option<DeclarationId>,
    },
    LocalMutable {
        shadowed: Option<DeclarationId>,
    },
    Module {
        kind: ModuleKind,
        inner_scope_id: ScopeId,
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
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeclarationKind::Function { .. } => write!(f, "function"),
            DeclarationKind::NativeFunction => write!(f, "native function"),
            DeclarationKind::Local { .. } => write!(f, "local variable"),
            DeclarationKind::LocalMutable { .. } => write!(f, "mutable local variable"),
            DeclarationKind::Module { .. } => write!(f, "module"),
            DeclarationKind::Type => write!(f, "type"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKind {
    File,
    Inline,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeKey {
    hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeNode {
    Array(TypeId, u32),
    List(TypeId),
    Function(FunctionTypeNode),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionTypeNode {
    pub type_parameters: (u32, u32),
    pub value_parameters: (u32, u32),
    pub return_type: TypeId,
}
