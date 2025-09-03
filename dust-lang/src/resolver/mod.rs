mod scope;

use std::hash::{Hash, Hasher};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{ConstantTable, OperandType, Span, Type};

#[derive(Debug)]
pub struct Resolver {
    pub constants: ConstantTable,

    declarations: IndexMap<u64, Declaration, FxBuildHasher>,

    scopes: Vec<Scope>,

    types: IndexSet<TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            constants: ConstantTable::new(),
            declarations: IndexMap::default(),
            scopes: Vec::new(),
            r#types: IndexSet::default(),
            type_members: Vec::new(),
        }
    }

    pub fn get_declaration_from_id(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations
            .get_index(id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_declaration(&mut self, identifier: &str, declaration: Declaration) -> DeclarationId {
        let hash = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);
            declaration.scope.hash(&mut hasher);
            hasher.finish()
        };

        let id = DeclarationId(self.declarations.len() as u32);

        self.declarations.insert(hash, declaration);

        id
    }

    pub fn get_declaration(&mut self, identifier: &str, scope: ScopeId) -> Option<&Declaration> {
        let hash = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);
            scope.hash(&mut hasher);
            hasher.finish()
        };

        self.declarations.get(&hash)
    }

    pub fn get_declaration_full(
        &mut self,
        identifier: &str,
        scope: ScopeId,
    ) -> Option<(DeclarationId, &Declaration)> {
        let hash = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);
            scope.hash(&mut hasher);
            hasher.finish()
        };

        self.declarations
            .get_full(&hash)
            .map(|(index, _, value)| (DeclarationId(index as u32), value))
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
                    TypeNode::Function {
                        type_arguments: (type_args_start, type_args_end),
                        value_arguments: (value_args_start, value_args_end),
                        return_type,
                    } => {
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

    pub fn register_type(&mut self, type_node: TypeNode) -> TypeId {
        if let Some(existing) = self.types.get_index_of(&type_node) {
            TypeId(existing as u32)
        } else {
            let id = TypeId(self.types.len() as u32);

            self.types.insert(type_node);

            id
        }
    }

    pub fn push_type_members(&mut self, members: &[TypeId]) -> (u32, u32) {
        let start = self.type_members.len() as u32;

        self.type_members.extend_from_slice(members);

        let end = self.type_members.len() as u32;

        (start, end)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const MAIN: Self = ScopeId(u32::MAX);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub parent: ScopeId,
    pub imports: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeclarationId(pub u32);

impl DeclarationId {
    pub const MAIN: Self = DeclarationId(u32::MAX);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope: ScopeId,
    pub r#type: TypeId,
    pub identifier_position: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function,
    Local,
    Module,
    Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: Self = TypeId(u32::MAX);
    pub const BOOLEAN: Self = TypeId(u32::MAX - 1);
    pub const BYTE: Self = TypeId(u32::MAX - 2);
    pub const CHARACTER: Self = TypeId(u32::MAX - 3);
    pub const FLOAT: Self = TypeId(u32::MAX - 4);
    pub const INTEGER: Self = TypeId(u32::MAX - 5);
    pub const STRING: Self = TypeId(u32::MAX - 6);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeNode {
    Array(TypeId, u32),
    List(TypeId),
    Function {
        type_arguments: (u32, u32),
        value_arguments: (u32, u32),
        return_type: TypeId,
    },
}
