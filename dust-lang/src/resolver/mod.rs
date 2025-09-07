use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::{ConstantTable, OperandType, Span, Type};

#[derive(Debug)]
pub struct Resolver {
    pub constants: ConstantTable,

    declarations: IndexMap<(Symbol, ScopeId), Declaration, FxBuildHasher>,

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
        identifier: &str,
        identifier_position: Span,
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
        };
        let declaration_id = DeclarationId(self.declarations.len() as u32);

        self.declarations.insert((symbol, scope_id), declaration);

        declaration_id
    }

    pub fn find_declaration_in_scope_chain(
        &mut self,
        identifier: &str,
        mut scope: ScopeId,
    ) -> Option<DeclarationId> {
        let symbol = {
            let mut hasher = FxHasher::default();

            identifier.hash(&mut hasher);

            Symbol {
                hash: hasher.finish(),
            }
        };

        loop {
            if let Some(declaration_id) = self.get_scoped_declaration(symbol, scope) {
                return Some(declaration_id);
            }

            if scope == ScopeId::MAIN {
                break;
            }

            scope = self.scopes.get(scope.0 as usize)?.parent;
        }

        None
    }

    fn get_scoped_declaration(&mut self, symbol: Symbol, scope: ScopeId) -> Option<DeclarationId> {
        self.declarations
            .get_index_of(&(symbol, scope))
            .map(|index| DeclarationId(index as u32))
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
pub struct Symbol {
    hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const MAIN: Self = ScopeId(0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    pub parent: ScopeId,
    pub imports: (u32, u32),
    pub depth: u8,
    pub index: u16,
}

impl Scope {
    pub fn contains(&self, other: &Scope) -> bool {
        self.depth >= other.depth && self.index <= other.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(pub u32);

impl DeclarationId {
    pub const MAIN: Self = DeclarationId(u32::MAX);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub scope_id: ScopeId,
    pub type_id: TypeId,
    pub identifier_position: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    Function,
    Local,
    LocalMutable,
    Module,
    Type,
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeclarationKind::Function => write!(f, "function"),
            DeclarationKind::Local => write!(f, "local variable"),
            DeclarationKind::LocalMutable => write!(f, "mutable local variable"),
            DeclarationKind::Module => write!(f, "module"),
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
