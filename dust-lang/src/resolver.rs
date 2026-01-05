use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::{FxBuildHasher, FxHasher};
use smallvec::SmallVec;

use crate::{
    instruction::OperandType,
    native_function::NativeFunction,
    source::{Position, SourceFileId},
    syntax_tree::SyntaxId,
    r#type::{FunctionType, Type},
};

#[derive(Debug)]
pub struct Resolver {
    declarations: IndexMap<DeclarationKey, Declaration, FxBuildHasher>,

    parameters: IndexSet<DeclarationId, FxBuildHasher>,

    scopes: Vec<Scope>,

    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,

    type_nodes: IndexMap<TypeKey, TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,

    next_inferred_type_id: u32,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            scope_bindings: HashMap::default(),
            declarations: IndexMap::default(),
            parameters: IndexSet::default(),
            scopes: vec![],
            type_nodes: IndexMap::default(),
            type_members: Vec::new(),
            next_inferred_type_id: 0,
        };

        let _native_scope_id = resolver.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::MAIN,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
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
                    file_id: SourceFileId::MAIN,
                    syntax_id: SyntaxId(0),
                    parameters: (0, 0),
                    prototype_index: Some(0),
                },
                scope_id: ScopeId::MAIN,
                type_id: TypeId::NONE,
                position: Position::default(),
                is_public: true,
            },
        );

        debug_assert_eq!(_native_scope_id, ScopeId::NATIVE);
        debug_assert_eq!(_main_scope_id, ScopeId::MAIN);
        debug_assert_eq!(_main_function_declaration_id, DeclarationId::MAIN);

        resolver.add_native_functions();

        resolver
    }

    pub fn add_native_functions(&mut self) {
        for native_function in NativeFunction::ALL {
            let type_id = self.add_type(&Type::Function(Box::new(native_function.r#type())));

            self.add_declaration(
                native_function.name(),
                Declaration {
                    kind: DeclarationKind::NativeFunction,
                    scope_id: ScopeId::NATIVE,
                    type_id,
                    position: Position::default(),
                    is_public: true,
                },
            );
        }
    }

    pub fn declarations(&self) -> &IndexMap<DeclarationKey, Declaration, FxBuildHasher> {
        &self.declarations
    }

    pub fn type_count(&self) -> usize {
        self.type_nodes.len()
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

    pub fn get_declaration_mut(
        &mut self,
        declaration_id: &DeclarationId,
    ) -> Option<&mut Declaration> {
        self.declarations
            .get_index_mut(declaration_id.0 as usize)
            .map(|(_, declaration)| declaration)
    }

    pub fn add_parameters(&mut self, parameter_ids: &[DeclarationId]) -> (u32, u32) {
        let start = self.parameters.len() as u32;
        let count = parameter_ids.len() as u32;

        self.parameters.extend(parameter_ids);

        (start, count)
    }

    pub fn get_parameter(&self, index: u32) -> Option<DeclarationId> {
        self.parameters.get_index(index as usize).copied()
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

        if let Some(existing) = self.type_nodes.get_index_of(&type_key) {
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
            Type::List(element_type) => {
                let element_type_id = self.add_type(element_type);
                let type_node = TypeNode::List(element_type_id);
                let type_id = TypeId(self.type_nodes.len() as u32);

                self.type_nodes.insert(type_key, type_node);

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

        if let Some(existing) = self.type_nodes.get_index_of(&type_key) {
            return TypeId(existing as u32);
        }

        let type_id = TypeId(self.type_nodes.len() as u32);

        self.type_nodes.insert(type_key, type_node);

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
                let (_, type_node) = self.type_nodes.get_index(index as usize)?;

                match type_node {
                    TypeNode::List(element_type_id) => {
                        let element_type = self.resolve_type(*element_type_id)?;

                        Some(Type::list(element_type))
                    }
                    TypeNode::Function(function_type_node) => {
                        let function_type = self.resolve_function_type(function_type_node)?;

                        Some(Type::Function(Box::new(function_type)))
                    }
                    TypeNode::Inferred(_) => None,
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
        self.type_nodes
            .get_index(id.0 as usize)
            .map(|(_, type_node)| type_node)
    }

    pub fn get_operand_type(&self, id: TypeId) -> Option<OperandType> {
        match id {
            TypeId::NONE => return Some(OperandType::NONE),
            TypeId::BOOLEAN => return Some(OperandType::BOOLEAN),
            TypeId::BYTE => return Some(OperandType::BYTE),
            TypeId::CHARACTER => return Some(OperandType::CHARACTER),
            TypeId::FLOAT => return Some(OperandType::FLOAT),
            TypeId::INTEGER => return Some(OperandType::INTEGER),
            TypeId::STRING => return Some(OperandType::STRING),
            _ => {}
        }

        let type_node = self.get_type_node(id)?;
        let operand_type = match type_node {
            TypeNode::List(element_type_id) => {
                let element_operand_type = self.get_operand_type(*element_type_id)?;

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
            TypeNode::Function(_) => OperandType::FUNCTION,
            TypeNode::Inferred(_) => return None,
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
        let inferred_type_node = TypeNode::Inferred(InferredTypeNode {
            id: self.next_inferred_type_id,
            resolved: None,
        });
        let type_id = self.add_type_node(inferred_type_node);

        self.next_inferred_type_id += 1;

        type_id
    }

    pub fn infer_type(&mut self, inferred_type_id: TypeId) -> TypeId {
        if let TypeId::NONE
        | TypeId::BOOLEAN
        | TypeId::BYTE
        | TypeId::CHARACTER
        | TypeId::FLOAT
        | TypeId::INTEGER
        | TypeId::STRING = inferred_type_id
        {
            return inferred_type_id;
        }

        match self.get_type_node(inferred_type_id) {
            Some(TypeNode::Inferred(InferredTypeNode {
                resolved: Some(resolved),
                ..
            })) => self.infer_type(*resolved),
            _ => inferred_type_id,
        }
    }

    pub fn unify_types(&mut self, left: TypeId, right: TypeId) -> Option<TypeId> {
        let left = self.infer_type(left);
        let right = self.infer_type(right);

        if left == right {
            return Some(left);
        }

        let left_node = *self.get_type_node(left)?;
        let right_node = *self.get_type_node(right)?;

        match (left_node, right_node) {
            (TypeNode::Inferred(InferredTypeNode { id, resolved: None }), _) => {
                if let Some((_, node)) = self.type_nodes.get_index_mut(left.0 as usize) {
                    *node = TypeNode::Inferred(InferredTypeNode {
                        id,
                        resolved: Some(right),
                    });
                }

                Some(right)
            }
            (_, TypeNode::Inferred(InferredTypeNode { id, resolved: None })) => {
                if let Some((_, node)) = self.type_nodes.get_index_mut(right.0 as usize) {
                    *node = TypeNode::Inferred(InferredTypeNode {
                        id,
                        resolved: Some(left),
                    });
                }

                Some(left)
            }
            (TypeNode::List(left_element_type), TypeNode::List(right_element_type)) => {
                self.unify_types(left_element_type, right_element_type)?;

                Some(left)
            }
            (TypeNode::Function(left_function_type), TypeNode::Function(right_function_type)) => {
                let mut unify_members = |left: (u32, u32), right: (u32, u32)| -> Option<()> {
                    let left_members = self
                        .get_type_members(left.0, left.1)?
                        .iter()
                        .copied()
                        .collect::<SmallVec<[TypeId; 8]>>();
                    let right_members = self
                        .get_type_members(right.0, right.1)?
                        .iter()
                        .copied()
                        .collect::<SmallVec<[TypeId; 8]>>();

                    if left_members.len() != right_members.len() {
                        return None;
                    }

                    for (left_member, right_member) in left_members.iter().zip(right_members.iter())
                    {
                        self.unify_types(*left_member, *right_member)?;
                    }

                    Some(())
                };

                unify_members(
                    left_function_type.type_parameters,
                    right_function_type.type_parameters,
                )?;
                unify_members(
                    left_function_type.value_parameters,
                    right_function_type.value_parameters,
                )?;
                self.unify_types(
                    left_function_type.return_type,
                    right_function_type.return_type,
                )?;

                Some(left)
            }
            _ => None,
        }
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
    pub const NATIVE: Self = ScopeId(0);
    pub const MAIN: Self = ScopeId(1);
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
        file_id: SourceFileId,
        syntax_id: SyntaxId,
        parameters: (u32, u32),
        prototype_index: Option<u16>,
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
    List(TypeId),
    Function(FunctionTypeNode),
    Inferred(InferredTypeNode),
}

impl TypeNode {
    pub fn into_function_type(self) -> Option<FunctionTypeNode> {
        if let TypeNode::Function(function_type_node) = self {
            Some(function_type_node)
        } else {
            None
        }
    }

    pub fn into_list_element_type(self) -> Option<TypeId> {
        if let TypeNode::List(element_type_id) = self {
            Some(element_type_id)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionTypeNode {
    pub type_parameters: (u32, u32),
    pub value_parameters: (u32, u32),
    pub return_type: TypeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferredTypeNode {
    id: u32,
    resolved: Option<TypeId>,
}
