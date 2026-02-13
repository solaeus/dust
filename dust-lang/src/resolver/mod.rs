pub mod declaration_graph;
pub mod scope_graph;
pub mod symbol_table;
pub mod type_graph;

use std::collections::{HashMap, HashSet};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    dust_type::{DustFunctionType, DustStructType, DustType},
    instruction::OperandType,
    native_function::NativeFunction,
    resolver::{
        declaration_graph::{
            Declaration, DeclarationGraph, DeclarationId, DeclarationKind, DeclarationMembers,
            ModuleKind,
        },
        scope_graph::{Scope, ScopeGraph, ScopeId, ScopeKind},
        symbol_table::{SymbolId, SymbolTable},
        type_graph::{TypeGraph, TypeId, TypeMembers, TypeNode},
    },
    source::{Position, Source},
    syntax::{SyntaxId, SyntaxReader},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: SymbolTable,
    pub declarations: DeclarationGraph,
    pub scopes: ScopeGraph,
    pub types: TypeGraph,

    scope_search: HashSet<ScopeId, FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut declarations = DeclarationGraph::new();
        let mut scopes = ScopeGraph::new();
        let mut symbols = SymbolTable::new();
        let mut types = TypeGraph::new();

        let mut core_imports =
            SmallVec::<[DeclarationId; 4]>::with_capacity(NativeFunction::ALL.len());

        for native_function in NativeFunction::ALL {
            let function_symbol = symbols.add_named_symbol(native_function.name());
            let declaration_id = declarations.add_declaration(Declaration {
                symbol_id: function_symbol,
                position: None,
                kind: DeclarationKind::NativeFunction(native_function),
                scope_id: ScopeId::CORE,
                is_public: true,
            });
            let type_id = native_function.signature(&mut types);

            declarations.set_declaration_type(declaration_id, type_id);
            core_imports.push(declaration_id);
        }

        let core_symbol = symbols.add_anonymous_symbol();
        let _core_declaration_id = declarations.add_declaration(Declaration {
            symbol_id: core_symbol,
            position: None,
            kind: DeclarationKind::Module {
                kind: ModuleKind::Inline,
                inner_scope_id: ScopeId::CORE,
            },
            scope_id: ScopeId::NONE,
            is_public: true,
        });

        let _core_scope_id = scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: core_imports,
        });

        debug_assert_eq!(_core_scope_id, ScopeId::CORE);

        Self {
            symbols,
            declarations,
            scopes,
            types,
            scope_search: HashSet::default(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
        }
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
                InternalCompileError::MissingDeclarationBinding(*syntax_id),
            ))
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, CompileError> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingScopeBinding(*syntax_id),
            ))
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, CompileError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(CompileError::Internal(
                InternalCompileError::MissingTypeBinding(*syntax_id),
            ))
    }

    pub fn find_declaration_in_scope(
        &self,
        _symbol: SymbolId,
        _path_segment: &SyntaxReader,
        _target_scope_id: ScopeId,
        _parent: Option<DeclarationId>,
        _is_type_lookup: bool,
    ) -> Result<(DeclarationId, Declaration), CompileError> {
        todo!()
    }

    pub fn infer_type(&mut self, type_id: TypeId) -> Result<TypeId, CompileError> {
        if let TypeNode::Inferred {
            resolved: Some(resolved),
            ..
        } = self.types.get_type(type_id)?
        {
            self.infer_type(*resolved)
        } else {
            Ok(type_id)
        }
    }

    pub fn unify_types(
        &mut self,
        left: TypeId,
        left_position: Option<Position>,
        right: TypeId,
        right_position: Position,
    ) -> Result<(), CompileError> {
        let left_inferred = self.infer_type(left)?;
        let right_inferred = self.infer_type(right)?;

        self.unify_inferred_types(left_inferred, left_position, right_inferred, right_position)
    }

    pub fn unify_inferred_types(
        &mut self,
        left: TypeId,
        left_position: Option<Position>,
        right: TypeId,
        right_position: Position,
    ) -> Result<(), CompileError> {
        if left == right {
            return Ok(());
        }

        let left_type_node = *self.types.get_type(left)?;
        let right_type_node = *self.types.get_type(right)?;

        match (left_type_node, right_type_node) {
            (
                TypeNode::Inferred {
                    inferred_id,
                    resolved: None,
                },
                _,
            ) => {
                let left_node = self.types.get_type_mut(left)?;

                *left_node = TypeNode::Inferred {
                    inferred_id,
                    resolved: Some(right),
                };

                Ok(())
            }
            (
                _,
                TypeNode::Inferred {
                    inferred_id,
                    resolved: None,
                },
            ) => {
                let right_node = self.types.get_type_mut(right)?;

                *right_node = TypeNode::Inferred {
                    inferred_id,
                    resolved: Some(left),
                };

                Ok(())
            }
            (
                TypeNode::List {
                    element_type: left_element_type,
                },
                TypeNode::List {
                    element_type: right_element_type,
                },
            ) => self.unify_types(
                left_element_type,
                left_position,
                right_element_type,
                right_position,
            ),
            (
                TypeNode::Function {
                    type_parameters: _left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type,
                },
                TypeNode::Function {
                    type_parameters: _right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type,
                },
            ) => {
                let left_value_types = self
                    .types
                    .get_type_members(left_value_parameters)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_value_types = self
                    .types
                    .get_type_members(right_value_parameters)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();

                for (left_type_id, right_type_id) in left_value_types
                    .into_iter()
                    .zip(right_value_types.into_iter())
                {
                    self.unify_types(left_type_id, left_position, right_type_id, right_position)?;
                }

                self.unify_types(
                    left_return_type,
                    left_position,
                    right_return_type,
                    right_position,
                )?;

                Ok(())
            }
            (
                TypeNode::Struct {
                    declaration_id: left_declaration_id,
                    generics: _left_generics,
                    fields: left_fields,
                },
                TypeNode::Struct {
                    declaration_id: right_declaration_id,
                    generics: _right_generics,
                    fields: right_fields,
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    return Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position: left_position,
                        found_type: right,
                        found_position: right_position,
                    });
                }

                let left_field_types = self
                    .declarations
                    .get_declaration_members(left_fields)?
                    .iter()
                    .map(|declaration_id| {
                        self.declarations
                            .get_declaration_type(declaration_id)
                            .copied()
                    })
                    .try_collect::<SmallVec<[TypeId; 8]>>()?;
                let right_field_types = self
                    .declarations
                    .get_declaration_members(right_fields)?
                    .iter()
                    .map(|declaration_id| {
                        self.declarations
                            .get_declaration_type(declaration_id)
                            .copied()
                    })
                    .try_collect::<SmallVec<[TypeId; 8]>>()?;

                for (left_field_type, right_field_type) in
                    left_field_types.iter().zip(right_field_types.iter())
                {
                    self.unify_types(
                        *left_field_type,
                        left_position,
                        *right_field_type,
                        right_position,
                    )?;
                }

                Ok(())
            }
            (left_type_node, right_type_node) => {
                if left_type_node == right_type_node {
                    Ok(())
                } else {
                    Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position: left_position,
                        found_type: right,
                        found_position: right_position,
                    })
                }
            }
        }
    }

    pub fn add_external_type(&mut self, new_type: &DustType) -> TypeId {
        let node = match new_type {
            DustType::None => TypeNode::None,
            DustType::Boolean => TypeNode::Boolean,
            DustType::Byte => TypeNode::Byte,
            DustType::Character => TypeNode::Character,
            DustType::Float => TypeNode::Float,
            DustType::Integer => TypeNode::Integer,
            DustType::String => TypeNode::String,
            DustType::List(element_type) => {
                let element_type = self.add_external_type(element_type);

                TypeNode::List { element_type }
            }
            DustType::Function(function_type) => {
                let mut type_parameters = SmallVec::<[DeclarationId; 4]>::with_capacity(
                    function_type.type_parameters.len(),
                );

                for type_parameter_name in &function_type.type_parameters {
                    let symbol = self.symbols.add_named_symbol(type_parameter_name);
                    let type_parameter_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type { parent: None },
                        scope_id: ScopeId::NONE,
                        is_public: false,
                        position: None,
                    });
                    let type_parameter_type_id = self.types.create_inferred_type();

                    type_parameters.push(type_parameter_id);
                    self.declarations
                        .set_declaration_type(type_parameter_id, type_parameter_type_id);
                }

                let mut value_parameter_types: SmallVec<[TypeId; 8]> =
                    SmallVec::with_capacity(function_type.value_parameters.len());

                for r#type in &function_type.value_parameters {
                    value_parameter_types.push(self.add_external_type(r#type));
                }

                TypeNode::Function {
                    type_parameters: self.declarations.add_declaration_members(&type_parameters),
                    value_parameters: self.types.add_type_members(&value_parameter_types),
                    return_type_id: self.add_external_type(&function_type.return_type),
                }
            }
            DustType::Struct(struct_type) => {
                let DustStructType { name, fields } = struct_type.as_ref();

                let symbol = self.symbols.add_named_symbol(name);
                let struct_declaration_id = self.declarations.add_declaration(Declaration {
                    symbol_id: symbol,
                    kind: DeclarationKind::Type { parent: None },
                    scope_id: ScopeId::NONE,
                    is_public: false,
                    position: None,
                });

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let symbol = self.symbols.add_named_symbol(field_name);
                    let declaration_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type {
                            parent: Some(struct_declaration_id),
                        },
                        scope_id: ScopeId::NONE,
                        is_public: false,
                        position: None,
                    });
                    let type_id = self.add_external_type(field_type);

                    field_declaration_ids.push(declaration_id);
                    self.declarations
                        .set_declaration_type(declaration_id, type_id);
                }

                let fields = self
                    .declarations
                    .add_declaration_members(&field_declaration_ids);

                TypeNode::Struct {
                    declaration_id: struct_declaration_id,
                    generics: DeclarationMembers::default(),
                    fields,
                }
            }
        };

        self.types.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId, source: &Source) -> Result<DustType, CompileError> {
        let type_node = self.types.get_type(id)?;

        match type_node {
            TypeNode::None => Ok(DustType::None),
            TypeNode::Boolean => Ok(DustType::Boolean),
            TypeNode::Byte => Ok(DustType::Byte),
            TypeNode::Character => Ok(DustType::Character),
            TypeNode::Float => Ok(DustType::Float),
            TypeNode::Integer => Ok(DustType::Integer),
            TypeNode::String => Ok(DustType::String),
            TypeNode::List { element_type } => {
                let element_type = self.get_full_type(*element_type, source)?;

                Ok(DustType::list(element_type))
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

                Ok(DustType::Function(Box::new(DustFunctionType {
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
                let struct_declaration = self.declarations.get_declaration(*declaration_id)?;
                let struct_name = self
                    .symbols
                    .get_symbol(&struct_declaration.symbol_id)?
                    .to_string();

                let fields = self.declarations.get_declaration_members(*fields)?;
                let mut field_types = Vec::with_capacity(fields.len());

                for field_id in fields {
                    let field_declaration = self.declarations.get_declaration(*field_id)?;
                    let field_name = self
                        .symbols
                        .get_symbol(&field_declaration.symbol_id)?
                        .to_string();

                    let field_type_id = self.declarations.get_declaration_type(field_id)?;
                    let field_type = self.get_full_type(*field_type_id, source)?;

                    field_types.push((field_name, field_type));
                }

                Ok(DustType::Struct(Box::new(DustStructType {
                    name: struct_name,
                    fields: field_types,
                })))
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
            let declaration_id = self.declarations.get_declaration_member(member_index)?;
            let declaration = self.declarations.get_declaration(*declaration_id)?;
            let name = self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

            Ok(name)
        })
    }

    fn get_type_members_as_full_types(
        &self,
        members: TypeMembers,
        source: &Source,
    ) -> impl Iterator<Item = Result<DustType, CompileError>> {
        members.as_range().map(|member_index| {
            let type_id = *self.types.get_type_member(member_index)?;

            self.get_full_type(type_id, source)
        })
    }

    pub fn get_operand_type(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<OperandType, CompileError> {
        let operand_type = match self.types.get_type(type_id)? {
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

    pub fn get_register_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<u16, CompileError> {
        match self.types.get_type(type_id)? {
            TypeNode::None => Err(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            }),
            TypeNode::Struct { fields, .. } => {
                let mut leaf_count: u32 = 0;

                for index in fields.start..(fields.start + fields.count) {
                    let field_declaration_id = self.declarations.get_declaration_member(index)?;
                    let field_type_id = *self
                        .declarations
                        .get_declaration_type(field_declaration_id)?;
                    let field_register_size = self.get_register_size(field_type_id, node)? as u32;

                    let mut resolved_field_type_id = field_type_id;

                    while let TypeNode::Inferred {
                        resolved: Some(resolved),
                        ..
                    } = self.types.get_type(resolved_field_type_id)?
                    {
                        resolved_field_type_id = *resolved;
                    }

                    let field_leaf_count = match self.types.get_type(resolved_field_type_id)? {
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
