use indexmap::{IndexSet, set::MutableValues};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::CompileError,
    instruction::OperandType,
    r#type::{FunctionType, Type},
};

#[derive(Debug)]
pub struct TypeGraph {
    type_nodes: IndexSet<TypeNode, FxBuildHasher>,

    type_members: Vec<TypeId>,

    next_inferred_type_id: u32,
}

impl TypeGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            type_nodes: IndexSet::default(),
            type_members: Vec::new(),
            next_inferred_type_id: 0,
        };

        graph.type_nodes.reserve(7);

        let _none_id = graph.add_type(TypeNode::None);
        let _boolean_id = graph.add_type(TypeNode::Boolean);
        let _byte_id = graph.add_type(TypeNode::Byte);
        let _character_id = graph.add_type(TypeNode::Character);
        let _float_id = graph.add_type(TypeNode::Float);
        let _integer_id = graph.add_type(TypeNode::Integer);
        let _string_id = graph.add_type(TypeNode::String);

        debug_assert_eq!(_none_id, TypeId::NONE);
        debug_assert_eq!(_boolean_id, TypeId::BOOLEAN);
        debug_assert_eq!(_byte_id, TypeId::BYTE);
        debug_assert_eq!(_character_id, TypeId::CHARACTER);
        debug_assert_eq!(_float_id, TypeId::FLOAT);
        debug_assert_eq!(_integer_id, TypeId::INTEGER);
        debug_assert_eq!(_string_id, TypeId::STRING);

        graph
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
        };

        self.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId) -> Option<Type> {
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
                let element_type = self.get_full_type(*element_type)?;

                Some(Type::list(element_type))
            }
            TypeNode::Function {
                type_parameters,
                value_parameters,
                return_type_id,
            } => {
                let type_parameters = self
                    .get_members_as_full_types(type_parameters.0, type_parameters.1)
                    .try_collect::<Vec<Type>>()?;
                let value_parameters = self
                    .get_members_as_full_types(value_parameters.0, value_parameters.1)
                    .try_collect::<Vec<Type>>()?;
                let return_type = self.get_full_type(*return_type_id)?;

                Some(Type::Function(Box::new(FunctionType {
                    type_parameters,
                    value_parameters,
                    return_type,
                })))
            }
            TypeNode::Inferred { resolved, .. } => {
                resolved.and_then(|resolved_id| self.get_full_type(resolved_id))
            }
        }
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

    pub fn create_inferred_type(&mut self) -> TypeId {
        let inferred_type_node = TypeNode::Inferred {
            id: self.next_inferred_type_id,
            resolved: None,
        };
        let type_id = self.add_type(inferred_type_node);

        self.next_inferred_type_id += 1;

        type_id
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
            (left, right) => Ok(left == right),
        }
    }

    fn get_members_as_full_types(
        &self,
        start_index: u32,
        count: u32,
    ) -> impl Iterator<Item = Option<Type>> {
        let range = start_index as usize..(start_index + count) as usize;

        self.type_members[range]
            .iter()
            .map(|type_id| self.get_full_type(*type_id))
    }
}

impl Default for TypeGraph {
    fn default() -> Self {
        Self::new()
    }
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
}
