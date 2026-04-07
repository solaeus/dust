pub mod declarations;
pub mod scopes;
pub mod symbols;
pub mod types;

use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            declarations::{
                Declaration, DeclarationId, DeclarationMembers, Declarations, Definition,
            },
            scopes::{Scope, ScopeId, ScopeKind, Scopes},
            symbols::Symbols,
            types::{
                FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId, TypeMembers,
                Types, UnsignedIntegerType,
            },
        },
    },
    dust_type::{DustEnumType, DustFunctionType, DustStructType, DustStructValueType, DustType},
    instruction::OperandType,
    prototype::PrototypeId,
    source::Source,
    syntax::SyntaxId,
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: Symbols,
    pub declarations: Declarations,
    pub scopes: Scopes,
    pub types: Types,
    pub type_parameter_map: HashMap<DeclarationId, TypeId>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    scope_bindings: HashMap<SyntaxId, ScopeId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,
    monomorphization_cache: HashMap<(DeclarationId, SmallVec<[TypeId; 4]>), PrototypeId>,
    constant_item_values: HashMap<DeclarationId, ConstantValue, FxBuildHasher>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            symbols: Symbols::new(),
            declarations: Declarations::new(),
            scopes: Scopes::new(),
            types: Types::new(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
            type_parameter_map: HashMap::new(),
            monomorphization_cache: HashMap::new(),
            constant_item_values: HashMap::default(),
        };

        add_core(&mut resolver);

        resolver
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
            .ok_or(CompileError::MissingDeclarationBinding(*syntax_id))
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, CompileError> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(CompileError::MissingScopeBinding(*syntax_id))
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, CompileError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(CompileError::MissingTypeBinding(*syntax_id))
    }

    pub fn get_cached_prototype(
        &self,
        cache_key: &(DeclarationId, SmallVec<[TypeId; 4]>),
    ) -> Option<PrototypeId> {
        self.monomorphization_cache.get(cache_key).copied()
    }

    pub fn cache_prototype(
        &mut self,
        cache_key: (DeclarationId, SmallVec<[TypeId; 4]>),
        prototype_id: PrototypeId,
    ) {
        self.monomorphization_cache.insert(cache_key, prototype_id);
    }

    pub fn get_concrete_type_arguments(
        &self,
        prototype_id: PrototypeId,
    ) -> Option<&SmallVec<[TypeId; 4]>> {
        self.monomorphization_cache
            .iter()
            .find_map(|((_, type_arguments), id)| {
                if *id == prototype_id {
                    Some(type_arguments)
                } else {
                    None
                }
            })
    }

    pub fn add_constant_item_value(&mut self, declaration_id: DeclarationId, value: ConstantValue) {
        self.constant_item_values.insert(declaration_id, value);
    }

    pub fn get_constant_item_value(&self, declaration_id: &DeclarationId) -> Option<ConstantValue> {
        self.constant_item_values.get(declaration_id).copied()
    }

    pub fn resolve_type(&mut self, type_id: TypeId) -> Result<TypeId, CompileError> {
        let resolved_type = *self.types.get_type(type_id)?;

        let start_type_id = match resolved_type {
            Type::Generic { declaration_id } | Type::Slice { declaration_id, .. } => {
                let concrete_type_id = self
                    .type_parameter_map
                    .get(&declaration_id)
                    .ok_or(CompileError::ExpectedConcreteType)?;

                *concrete_type_id
            }
            Type::Inferred { .. } => type_id,
            _ => return Ok(type_id),
        };

        let mut current_type_id = start_type_id;

        loop {
            match *self.types.get_type(current_type_id)? {
                Type::Inferred {
                    resolved: Some(resolved),
                    ..
                } => {
                    current_type_id = resolved;
                }
                Type::Inferred {
                    inferred_id,
                    constraint: Some(constraint),
                    resolved: None,
                } => {
                    let default_type_id = match constraint {
                        InferredTypeConstraint::Integer => TypeId::I_32,
                        InferredTypeConstraint::Float => TypeId::F_64,
                    };

                    let node = self.types.get_type_mut(current_type_id)?;

                    *node = Type::Inferred {
                        inferred_id,
                        constraint: Some(constraint),
                        resolved: Some(default_type_id),
                    };

                    return Ok(default_type_id);
                }
                _ => return Ok(current_type_id),
            }
        }
    }

    pub fn get_operand_types(&self, type_id: TypeId) -> Result<Vec<OperandType>, CompileError> {
        let r#type = self.types.get_type(type_id)?;

        match r#type {
            Type::Boolean => Ok(vec![OperandType::BOOLEAN]),
            Type::Character => Ok(vec![OperandType::CHARACTER]),
            Type::SignedInteger(SignedIntegerType::I8) => Ok(vec![OperandType::I_8]),
            Type::SignedInteger(SignedIntegerType::I16) => Ok(vec![OperandType::I_16]),
            Type::SignedInteger(SignedIntegerType::I32) => Ok(vec![OperandType::I_32]),
            Type::SignedInteger(SignedIntegerType::I64) => Ok(vec![OperandType::I_64]),
            Type::SignedInteger(SignedIntegerType::I128) => Ok(vec![OperandType::I_128]),
            Type::SignedInteger(SignedIntegerType::ISize) => {
                #[cfg(target_pointer_width = "64")]
                {
                    Ok(vec![OperandType::I_64])
                }

                #[cfg(target_pointer_width = "32")]
                {
                    Ok(vec![OperandType::I_32])
                }
            }
            Type::UnsignedInteger(UnsignedIntegerType::U8) => Ok(vec![OperandType::U_8]),
            Type::UnsignedInteger(UnsignedIntegerType::U16) => Ok(vec![OperandType::U_16]),
            Type::UnsignedInteger(UnsignedIntegerType::U32) => Ok(vec![OperandType::U_32]),
            Type::UnsignedInteger(UnsignedIntegerType::U64) => Ok(vec![OperandType::U_64]),
            Type::UnsignedInteger(UnsignedIntegerType::U128) => Ok(vec![OperandType::U_128]),
            Type::UnsignedInteger(UnsignedIntegerType::USize) => {
                #[cfg(target_pointer_width = "64")]
                {
                    Ok(vec![OperandType::U_64])
                }

                #[cfg(target_pointer_width = "32")]
                {
                    Ok(vec![OperandType::U_32])
                }
            }
            Type::Float(FloatType::F32) => Ok(vec![OperandType::F_32]),
            Type::Float(FloatType::F64) => Ok(vec![OperandType::F_64]),
            Type::Never => Ok(vec![]),
            Type::Tuple { element_type_ids } => {
                let element_type_ids = self.types.get_type_members(*element_type_ids)?;
                let mut operand_types = Vec::with_capacity(element_type_ids.len());

                for element_type_id in element_type_ids {
                    let element_operand_types = self.get_operand_types(*element_type_id)?;

                    operand_types.extend(element_operand_types);
                }

                Ok(operand_types)
            }
            Type::Array {
                element_type_id,
                length,
            } => {
                let element_operand_types = self.get_operand_types(*element_type_id)?;
                let mut operand_types = Vec::with_capacity(element_operand_types.len() * length);

                for _ in 0..*length {
                    operand_types.extend(&element_operand_types);
                }

                Ok(operand_types)
            }
            Type::Slice { .. } | Type::Pointer { .. } => Ok(vec![OperandType::POINTER]),
            Type::FunctionDefinition { .. } | Type::Closure { .. } | Type::Function { .. } => {
                Ok(vec![OperandType::FUNCTION])
            }
            Type::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id)?;

                match &declaration.definition {
                    Definition::EnumType {
                        variants,
                        type_parameters,
                        ..
                    } => {
                        let mut operand_types = vec![OperandType::U_32];

                        let type_parameter_argument_pairs = type_parameters
                            .as_range()
                            .zip(type_arguments.as_range())
                            .filter_map(|(parameter_index, argument_index)| {
                                let parameter_declaration_id = self
                                    .declarations
                                    .get_declaration_member(parameter_index)
                                    .ok()?;
                                let argument_type_id =
                                    self.types.get_type_member(argument_index).ok()?;

                                Some((*parameter_declaration_id, *argument_type_id))
                            });

                        let variant_declaration_ids =
                            self.declarations.get_declaration_members(variants)?;

                        for variant_declaration_id in variant_declaration_ids {
                            let variant_declaration =
                                self.declarations.get_declaration(*variant_declaration_id)?;
                            let Definition::Variant { fields, .. } =
                                &variant_declaration.definition
                            else {
                                return Err(CompileError::ExpectedVariantDeclaration(
                                    *variant_declaration_id,
                                ));
                            };

                            let field_declaration_ids =
                                self.declarations.get_declaration_members(fields)?;

                            for field_declaration_id in field_declaration_ids {
                                let field_declaration =
                                    self.declarations.get_declaration(*field_declaration_id)?;
                                let Definition::Field {
                                    type_id: field_type_id,
                                    ..
                                } = field_declaration.definition
                                else {
                                    return Err(CompileError::ExpectedFieldDeclaration(
                                        *field_declaration_id,
                                    ));
                                };

                                let resolved_field_type = *self.types.get_type(field_type_id)?;
                                let type_id = if let Type::Generic {
                                    declaration_id: parameter_declaration_id,
                                } = resolved_field_type
                                {
                                    type_parameter_argument_pairs
                                        .clone()
                                        .find_map(|(declaration_id, type_id)| {
                                            if declaration_id == parameter_declaration_id {
                                                Some(type_id)
                                            } else {
                                                None
                                            }
                                        })
                                        .ok_or(CompileError::MissingTypeArgument(
                                            *field_declaration_id,
                                        ))?
                                } else {
                                    field_type_id
                                };

                                let resolved_type_id = match self.types.get_type(type_id)? {
                                    Type::Inferred {
                                        resolved: Some(resolved),
                                        ..
                                    } => *resolved,
                                    Type::Inferred {
                                        constraint: Some(InferredTypeConstraint::Integer),
                                        resolved: None,
                                        ..
                                    } => TypeId::I_32,
                                    Type::Inferred {
                                        constraint: Some(InferredTypeConstraint::Float),
                                        resolved: None,
                                        ..
                                    } => TypeId::F_64,
                                    Type::Inferred { resolved: None, .. } => {
                                        return Err(CompileError::CannotInferType { type_id });
                                    }
                                    _ => type_id,
                                };
                                let field_operand_types =
                                    self.get_operand_types(resolved_type_id)?;

                                operand_types.extend(field_operand_types);
                            }
                        }

                        Ok(operand_types)
                    }
                    Definition::StructType {
                        fields,
                        type_parameters,
                        ..
                    } => {
                        let mut operand_types = Vec::new();

                        let type_parameter_argument_pairs = type_parameters
                            .as_range()
                            .zip(type_arguments.as_range())
                            .filter_map(|(parameter_index, argument_index)| {
                                let parameter_declaration_id = self
                                    .declarations
                                    .get_declaration_member(parameter_index)
                                    .ok()?;
                                let argument_type_id =
                                    self.types.get_type_member(argument_index).ok()?;

                                Some((*parameter_declaration_id, *argument_type_id))
                            });

                        let field_declaration_ids =
                            self.declarations.get_declaration_members(fields)?;

                        for field_declaration_id in field_declaration_ids {
                            let field_declaration =
                                self.declarations.get_declaration(*field_declaration_id)?;
                            let Definition::Field {
                                type_id: field_type_id,
                                ..
                            } = field_declaration.definition
                            else {
                                continue;
                            };

                            let resolved_field_type = *self.types.get_type(field_type_id)?;
                            let type_id = if let Type::Generic {
                                declaration_id: parameter_declaration_id,
                            } = resolved_field_type
                            {
                                type_parameter_argument_pairs
                                    .clone()
                                    .find_map(|(declaration_id, type_id)| {
                                        if declaration_id == parameter_declaration_id {
                                            Some(type_id)
                                        } else {
                                            None
                                        }
                                    })
                                    .ok_or(CompileError::MissingTypeArgument(
                                        *field_declaration_id,
                                    ))?
                            } else {
                                field_type_id
                            };

                            let resolved_type_id = match self.types.get_type(type_id)? {
                                Type::Inferred {
                                    resolved: Some(resolved),
                                    ..
                                } => *resolved,
                                Type::Inferred {
                                    constraint: Some(InferredTypeConstraint::Integer),
                                    resolved: None,
                                    ..
                                } => TypeId::I_32,
                                Type::Inferred {
                                    constraint: Some(InferredTypeConstraint::Float),
                                    resolved: None,
                                    ..
                                } => TypeId::F_64,
                                Type::Inferred { resolved: None, .. } => continue,
                                _ => type_id,
                            };

                            match self.get_operand_types(resolved_type_id) {
                                Ok(field_operand_types) => {
                                    operand_types.extend(field_operand_types)
                                }
                                error => return error,
                            }
                        }

                        Ok(operand_types)
                    }
                    _ => Err(CompileError::ExpectedConcreteType),
                }
            }
            Type::Generic { .. } => Err(CompileError::ExpectedConcreteType),
            Type::Inferred {
                resolved: Some(resolved),
                ..
            } => self.get_operand_types(*resolved),
            Type::Inferred {
                constraint: Some(InferredTypeConstraint::Integer),
                resolved: None,
                ..
            } => Ok(vec![OperandType::I_32]),
            Type::Inferred {
                constraint: Some(InferredTypeConstraint::Float),
                resolved: None,
                ..
            } => Ok(vec![OperandType::F_64]),
            Type::Inferred { .. } => Err(CompileError::ExpectedConcreteType),
        }
    }

    pub fn add_external_type(&mut self, new_type: &DustType) -> TypeId {
        match new_type {
            DustType::Unit => TypeId::UNIT,
            DustType::Boolean => TypeId::BOOLEAN,
            DustType::Character => TypeId::CHARACTER,
            DustType::I8 => TypeId::I_8,
            DustType::I16 => TypeId::I_16,
            DustType::I32 => TypeId::I_32,
            DustType::I64 => TypeId::I_64,
            DustType::I128 => TypeId::I_128,
            DustType::ISize => TypeId::I_SIZE,
            DustType::U8 => TypeId::U_8,
            DustType::U16 => TypeId::U_16,
            DustType::U32 => TypeId::U_32,
            DustType::U64 => TypeId::U_64,
            DustType::U128 => TypeId::U_128,
            DustType::USize => TypeId::U_SIZE,
            DustType::F32 => TypeId::F_32,
            DustType::F64 => TypeId::F_64,
            DustType::Tuple(element_types) => {
                let element_type_ids: SmallVec<[TypeId; 4]> = element_types
                    .iter()
                    .map(|element_type| self.add_external_type(element_type))
                    .collect();
                let element_type_ids = self.types.add_type_members(element_type_ids);

                self.types.add_type(Type::Tuple { element_type_ids })
            }
            DustType::Array(element_type, length) => {
                let element_type_id = self.add_external_type(element_type);

                self.types.add_type(Type::Array {
                    element_type_id,
                    length: *length,
                })
            }
            DustType::Function(function_type) => {
                let parameter_type_ids: SmallVec<[TypeId; 4]> = function_type
                    .value_parameters
                    .iter()
                    .map(|parameter_type| self.add_external_type(parameter_type))
                    .collect();
                let value_parameters = self.types.add_type_members(parameter_type_ids);
                let return_type = self.add_external_type(&function_type.return_type);

                self.types.add_type(Type::Function {
                    value_parameters,
                    return_type,
                })
            }
            DustType::Slice(element_type) => {
                let element_type_id = self.add_external_type(element_type);
                let declaration_id = self.declarations.reserve_declaration_id();

                self.types.add_type(Type::Slice {
                    declaration_id,
                    element_type_id,
                })
            }
            DustType::Struct(struct_type) => {
                let DustStructType { name, value_type } = struct_type.as_ref();
                let struct_symbol_id = self.symbols.add_symbol(name);
                let struct_declaration_id = self.declarations.reserve_declaration_id();

                let field_declaration_ids: Vec<DeclarationId> = match value_type {
                    DustStructValueType::Unit => Vec::new(),
                    DustStructValueType::Tuple(types) => types
                        .iter()
                        .enumerate()
                        .map(|(index, field_type)| {
                            let field_type_id = self.add_external_type(field_type);
                            let field_symbol_id = self.symbols.add_symbol(&index.to_string());

                            self.declarations.add_declaration(Declaration {
                                symbol_id: field_symbol_id,
                                definition: Definition::Field {
                                    public: true,
                                    parent_struct: struct_declaration_id,
                                    type_id: field_type_id,
                                },
                                scope_id: ScopeId::NONE,
                                syntax: None,
                            })
                        })
                        .collect(),
                    DustStructValueType::Struct(fields) => fields
                        .iter()
                        .map(|(field_name, field_type)| {
                            let field_type_id = self.add_external_type(field_type);
                            let field_symbol_id = self.symbols.add_symbol(field_name);

                            self.declarations.add_declaration(Declaration {
                                symbol_id: field_symbol_id,
                                definition: Definition::Field {
                                    public: true,
                                    parent_struct: struct_declaration_id,
                                    type_id: field_type_id,
                                },
                                scope_id: ScopeId::NONE,
                                syntax: None,
                            })
                        })
                        .collect(),
                };

                let fields = self
                    .declarations
                    .add_declaration_members(field_declaration_ids);

                self.declarations.set_declaration(
                    struct_declaration_id,
                    Declaration {
                        symbol_id: struct_symbol_id,
                        definition: Definition::StructType {
                            public: true,
                            type_parameters: DeclarationMembers::default(),
                            fields,
                        },
                        scope_id: ScopeId::NONE,
                        syntax: None,
                    },
                );

                self.types.add_type(Type::Algebraic {
                    declaration_id: struct_declaration_id,
                    type_arguments: TypeMembers::default(),
                })
            }
            DustType::Enum(enum_type) => {
                let DustEnumType { name, variants } = enum_type.as_ref();
                let enum_symbol_id = self.symbols.add_symbol(name);
                let enum_declaration_id = self.declarations.reserve_declaration_id();

                let variant_declaration_ids: Vec<DeclarationId> = variants
                    .iter()
                    .enumerate()
                    .map(|(discriminant, (variant_name, variant_value_type))| {
                        let variant_symbol_id = self.symbols.add_symbol(variant_name);

                        let field_declaration_ids: Vec<DeclarationId> = match variant_value_type {
                            DustStructValueType::Unit => Vec::new(),
                            DustStructValueType::Tuple(types) => types
                                .iter()
                                .enumerate()
                                .map(|(index, field_type)| {
                                    let field_type_id = self.add_external_type(field_type);
                                    let field_symbol_id =
                                        self.symbols.add_symbol(&index.to_string());

                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: enum_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: ScopeId::NONE,
                                        syntax: None,
                                    })
                                })
                                .collect(),
                            DustStructValueType::Struct(fields) => fields
                                .iter()
                                .map(|(field_name, field_type)| {
                                    let field_type_id = self.add_external_type(field_type);
                                    let field_symbol_id = self.symbols.add_symbol(field_name);

                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: enum_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: ScopeId::NONE,
                                        syntax: None,
                                    })
                                })
                                .collect(),
                        };

                        let fields = self
                            .declarations
                            .add_declaration_members(field_declaration_ids);

                        self.declarations.add_declaration(Declaration {
                            symbol_id: variant_symbol_id,
                            definition: Definition::Variant {
                                discriminant: discriminant as u32,
                                parent_enum: enum_declaration_id,
                                type_parameters: DeclarationMembers::default(),
                                fields,
                            },
                            scope_id: ScopeId::NONE,
                            syntax: None,
                        })
                    })
                    .collect();

                let variants = self
                    .declarations
                    .add_declaration_members(variant_declaration_ids);

                self.declarations.set_declaration(
                    enum_declaration_id,
                    Declaration {
                        symbol_id: enum_symbol_id,
                        definition: Definition::EnumType {
                            public: true,
                            type_parameters: DeclarationMembers::default(),
                            variants,
                        },
                        scope_id: ScopeId::NONE,
                        syntax: None,
                    },
                );

                self.types.add_type(Type::Algebraic {
                    declaration_id: enum_declaration_id,
                    type_arguments: TypeMembers::default(),
                })
            }
        }
    }

    pub fn get_external_type(
        &self,
        id: TypeId,
        _source: &Source,
    ) -> Result<DustType, CompileError> {
        let r#type = self.types.get_type(id)?;

        match r#type {
            Type::Never => Ok(DustType::Unit),
            Type::Boolean => Ok(DustType::Boolean),
            Type::Character => Ok(DustType::Character),
            Type::SignedInteger(signed) => match signed {
                SignedIntegerType::I8 => Ok(DustType::I8),
                SignedIntegerType::I16 => Ok(DustType::I16),
                SignedIntegerType::I32 => Ok(DustType::I32),
                SignedIntegerType::I64 => Ok(DustType::I64),
                SignedIntegerType::I128 => Ok(DustType::I128),
                SignedIntegerType::ISize => Ok(DustType::ISize),
            },
            Type::UnsignedInteger(unsigned) => match unsigned {
                UnsignedIntegerType::U8 => Ok(DustType::U8),
                UnsignedIntegerType::U16 => Ok(DustType::U16),
                UnsignedIntegerType::U32 => Ok(DustType::U32),
                UnsignedIntegerType::U64 => Ok(DustType::U64),
                UnsignedIntegerType::U128 => Ok(DustType::U128),
                UnsignedIntegerType::USize => Ok(DustType::USize),
            },
            Type::Float(float) => match float {
                FloatType::F32 => Ok(DustType::F32),
                FloatType::F64 => Ok(DustType::F64),
            },
            Type::Tuple { element_type_ids } => {
                if element_type_ids.is_empty() {
                    Ok(DustType::Unit)
                } else {
                    let element_types: Vec<DustType> = self
                        .get_type_members_as_full_types(*element_type_ids, _source)
                        .collect::<Result<_, _>>()?;

                    Ok(DustType::Tuple(element_types))
                }
            }
            Type::Inferred {
                resolved: Some(resolved),
                ..
            } => self.get_external_type(*resolved, _source),
            Type::Inferred {
                resolved: None,
                constraint: Some(InferredTypeConstraint::Integer),
                ..
            } => Ok(DustType::I32),
            Type::Inferred {
                resolved: None,
                constraint: Some(InferredTypeConstraint::Float),
                ..
            } => Ok(DustType::F64),
            Type::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id)?;

                match &declaration.definition {
                    Definition::EnumType {
                        variants,
                        type_parameters,
                        ..
                    } => {
                        let enum_name =
                            self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

                        let type_parameter_map: SmallVec<[(DeclarationId, TypeId); 4]> =
                            type_parameters
                                .as_range()
                                .zip(type_arguments.as_range())
                                .filter_map(|(parameter_index, argument_index)| {
                                    let parameter_declaration_id = self
                                        .declarations
                                        .get_declaration_member(parameter_index)
                                        .ok()?;
                                    let argument_type_id =
                                        self.types.get_type_member(argument_index).ok()?;
                                    Some((*parameter_declaration_id, *argument_type_id))
                                })
                                .collect();

                        let variant_declaration_ids =
                            self.declarations.get_declaration_members(variants)?;
                        let mut variants = Vec::with_capacity(variant_declaration_ids.len());

                        for variant_declaration_id in variant_declaration_ids {
                            let variant_declaration =
                                self.declarations.get_declaration(*variant_declaration_id)?;
                            let Definition::Variant { fields, .. } =
                                &variant_declaration.definition
                            else {
                                continue;
                            };

                            let variant_name = self
                                .symbols
                                .get_symbol(&variant_declaration.symbol_id)?
                                .to_string();

                            let field_declaration_ids =
                                self.declarations.get_declaration_members(fields)?;
                            let mut field_types = Vec::new();

                            for (field_index, field_declaration_id) in
                                field_declaration_ids.iter().enumerate()
                            {
                                let field_declaration =
                                    self.declarations.get_declaration(*field_declaration_id)?;
                                let Definition::Field {
                                    type_id: field_type_id,
                                    ..
                                } = field_declaration.definition
                                else {
                                    continue;
                                };

                                let resolved_type = self.types.get_type(field_type_id)?;
                                let concrete_type_id = if let Type::Generic {
                                    declaration_id: parameter_declaration,
                                } = resolved_type
                                {
                                    type_parameter_map
                                        .iter()
                                        .find(|(declaration, _)| {
                                            declaration == parameter_declaration
                                        })
                                        .map(|(_, type_id)| *type_id)
                                        .unwrap_or(field_type_id)
                                } else {
                                    field_type_id
                                };

                                let resolved_type_id =
                                    match self.types.get_type(concrete_type_id)? {
                                        Type::Inferred {
                                            resolved: Some(resolved),
                                            ..
                                        } => *resolved,
                                        Type::Inferred { resolved: None, .. } => continue,
                                        _ => concrete_type_id,
                                    };

                                let field_dust_type =
                                    self.get_external_type(resolved_type_id, _source)?;
                                let field_name = self
                                    .symbols
                                    .get_symbol(&field_declaration.symbol_id)
                                    .map(|symbol| symbol.to_string())
                                    .unwrap_or_else(|_| field_index.to_string());

                                field_types.push((field_name, field_dust_type));
                            }

                            let value_type = if field_types.is_empty() {
                                DustStructValueType::Unit
                            } else if field_types.iter().all(|(name, _)| name == "0") {
                                DustStructValueType::Tuple(
                                    field_types
                                        .into_iter()
                                        .map(|(_, field_type)| field_type)
                                        .collect(),
                                )
                            } else {
                                DustStructValueType::Struct(field_types)
                            };

                            variants.push((variant_name, value_type));
                        }

                        Ok(DustType::Enum(Box::new(DustEnumType {
                            name: enum_name,
                            variants,
                        })))
                    }
                    Definition::StructType { fields, .. } => {
                        let struct_name =
                            self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

                        let field_declaration_ids =
                            self.declarations.get_declaration_members(fields)?;
                        let mut field_types = Vec::new();

                        for field_declaration_id in field_declaration_ids {
                            let field_declaration =
                                self.declarations.get_declaration(*field_declaration_id)?;
                            let Definition::Field {
                                type_id: field_type_id,
                                ..
                            } = field_declaration.definition
                            else {
                                continue;
                            };

                            let field_dust_type = self.get_external_type(field_type_id, _source)?;
                            let field_name = self
                                .symbols
                                .get_symbol(&field_declaration.symbol_id)?
                                .to_string();

                            field_types.push((field_name, field_dust_type));
                        }

                        let value_type = if field_types.is_empty() {
                            DustStructValueType::Unit
                        } else if field_types.iter().all(|(name, _)| name == "0") {
                            DustStructValueType::Tuple(
                                field_types
                                    .into_iter()
                                    .map(|(_, field_type)| field_type)
                                    .collect(),
                            )
                        } else {
                            DustStructValueType::Struct(field_types)
                        };

                        Ok(DustType::Struct(Box::new(DustStructType {
                            name: struct_name,
                            value_type,
                        })))
                    }
                    Definition::TypeAlias {
                        aliased_type_id, ..
                    } => self.get_external_type(*aliased_type_id, _source),
                    _ => Err(CompileError::MissingAlgebraicTypeDeclaration(
                        *declaration_id,
                    )),
                }
            }
            Type::Array {
                element_type_id,
                length,
            } => {
                let element_dust_type = self.get_external_type(*element_type_id, _source)?;

                Ok(DustType::Array(Box::new(element_dust_type), *length))
            }
            Type::Generic { .. } => Err(CompileError::ExpectedConcreteType),
            Type::Function {
                value_parameters,
                return_type,
            } => {
                let value_parameter_types: Vec<DustType> = self
                    .get_type_members_as_full_types(*value_parameters, _source)
                    .collect::<Result<_, _>>()?;
                let return_dust_type = self.get_external_type(*return_type, _source)?;

                Ok(DustType::Function(Box::new(DustFunctionType {
                    type_parameters: Vec::new(),
                    value_parameters: value_parameter_types,
                    return_type: return_dust_type,
                })))
            }
            Type::Closure {
                value_parameters,
                return_type_id,
            } => {
                let value_parameter_types: Vec<DustType> = self
                    .get_type_members_as_full_types(*value_parameters, _source)
                    .collect::<Result<_, _>>()?;
                let return_dust_type = self.get_external_type(*return_type_id, _source)?;

                Ok(DustType::Function(Box::new(DustFunctionType {
                    type_parameters: Vec::new(),
                    value_parameters: value_parameter_types,
                    return_type: return_dust_type,
                })))
            }
            Type::Slice {
                element_type_id, ..
            } => {
                let element_dust_type = self.get_external_type(*element_type_id, _source)?;

                Ok(DustType::Slice(Box::new(element_dust_type)))
            }
            Type::FunctionDefinition {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id)?;

                match &declaration.definition {
                    Definition::Function {
                        value_parameters,
                        return_type_id,
                        type_parameters,
                        ..
                    } => {
                        let type_parameter_names: Vec<String> = self
                            .get_declaration_member_names(*type_parameters)
                            .collect::<Result<_, _>>()?;

                        let type_parameter_map: SmallVec<[(DeclarationId, TypeId); 4]> =
                            type_parameters
                                .as_range()
                                .zip(type_arguments.as_range())
                                .filter_map(|(parameter_index, argument_index)| {
                                    let parameter_declaration_id = self
                                        .declarations
                                        .get_declaration_member(parameter_index)
                                        .ok()?;
                                    let argument_type_id =
                                        self.types.get_type_member(argument_index).ok()?;
                                    Some((*parameter_declaration_id, *argument_type_id))
                                })
                                .collect();

                        let parameter_type_ids = self.types.get_type_members(*value_parameters)?;
                        let mut value_parameter_types =
                            Vec::with_capacity(parameter_type_ids.len());

                        for &parameter_type_id in parameter_type_ids {
                            let resolved_type = self.types.get_type(parameter_type_id)?;
                            let concrete_type_id = if let Type::Generic {
                                declaration_id: parameter_declaration,
                            } = resolved_type
                            {
                                type_parameter_map
                                    .iter()
                                    .find(|(declaration, _)| declaration == parameter_declaration)
                                    .map(|(_, type_id)| *type_id)
                                    .unwrap_or(parameter_type_id)
                            } else {
                                parameter_type_id
                            };

                            value_parameter_types
                                .push(self.get_external_type(concrete_type_id, _source)?);
                        }

                        let return_dust_type = self.get_external_type(*return_type_id, _source)?;

                        Ok(DustType::Function(Box::new(DustFunctionType {
                            type_parameters: type_parameter_names,
                            value_parameters: value_parameter_types,
                            return_type: return_dust_type,
                        })))
                    }
                    Definition::NativeFunction {
                        value_parameters,
                        return_type_id,
                        type_parameters,
                        ..
                    } => {
                        let type_parameter_names: Vec<String> = self
                            .get_declaration_member_names(*type_parameters)
                            .collect::<Result<_, _>>()?;

                        let type_parameter_map: SmallVec<[(DeclarationId, TypeId); 4]> =
                            type_parameters
                                .as_range()
                                .zip(type_arguments.as_range())
                                .filter_map(|(parameter_index, argument_index)| {
                                    let parameter_declaration_id = self
                                        .declarations
                                        .get_declaration_member(parameter_index)
                                        .ok()?;
                                    let argument_type_id =
                                        self.types.get_type_member(argument_index).ok()?;
                                    Some((*parameter_declaration_id, *argument_type_id))
                                })
                                .collect();

                        let parameter_type_ids = self.types.get_type_members(*value_parameters)?;
                        let mut value_parameter_types =
                            Vec::with_capacity(parameter_type_ids.len());

                        for &parameter_type_id in parameter_type_ids {
                            let resolved_type = self.types.get_type(parameter_type_id)?;
                            let concrete_type_id = if let Type::Generic {
                                declaration_id: parameter_declaration,
                            } = resolved_type
                            {
                                type_parameter_map
                                    .iter()
                                    .find(|(declaration, _)| declaration == parameter_declaration)
                                    .map(|(_, type_id)| *type_id)
                                    .unwrap_or(parameter_type_id)
                            } else {
                                parameter_type_id
                            };

                            value_parameter_types
                                .push(self.get_external_type(concrete_type_id, _source)?);
                        }

                        let return_dust_type = self.get_external_type(*return_type_id, _source)?;

                        Ok(DustType::Function(Box::new(DustFunctionType {
                            type_parameters: type_parameter_names,
                            value_parameters: value_parameter_types,
                            return_type: return_dust_type,
                        })))
                    }
                    _ => Err(CompileError::MissingFunctionDeclaration(*declaration_id)),
                }
            }
            Type::Pointer { .. } => Ok(DustType::Unit),
            Type::Inferred {
                resolved: None,
                constraint: None,
                ..
            } => Err(CompileError::ExpectedConcreteType),
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

            self.get_external_type(type_id, source)
        })
    }

    pub fn definition_display_iterator(
        &self,
    ) -> impl Iterator<Item = Result<(&str, String), CompileError>> {
        self.declarations.iter().map(|(_, declaration)| {
            let symbol = self.symbols.get_symbol(&declaration.symbol_id)?;

            Ok((symbol, format!("{:#?}", declaration.definition)))
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

fn add_core(resolver: &mut Resolver) {
    let _core_scope_id = resolver.scopes.add_scope(Scope {
        kind: ScopeKind::Module,
        parent: ScopeId::NONE,
        modules: SmallVec::new(),
        imports: SmallVec::new(),
    });

    debug_assert_eq!(_core_scope_id, ScopeId::CORE);

    let t_symbol = resolver.symbols.add_symbol("T");
    let field_0_symbol = resolver.symbols.add_index_symbol(0);

    {
        let option_symbol = resolver.symbols.add_symbol("Option");
        let some_symbol = resolver.symbols.add_symbol("Some");
        let none_symbol = resolver.symbols.add_symbol("None");

        let option_declaration_id = resolver.declarations.reserve_declaration_id();

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });

        let some_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: field_0_symbol,
            definition: Definition::Field {
                public: false,
                parent_struct: option_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let some_fields = resolver
            .declarations
            .add_declaration_members([some_field_declaration_id]);

        let some_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: some_symbol,
            definition: Definition::Variant {
                discriminant: 0,
                parent_enum: option_declaration_id,
                type_parameters: DeclarationMembers::default(),
                fields: some_fields,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let none_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: none_symbol,
            definition: Definition::Variant {
                discriminant: 1,
                parent_enum: option_declaration_id,
                type_parameters: DeclarationMembers::default(),
                fields: DeclarationMembers::default(),
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let type_parameters = resolver
            .declarations
            .add_declaration_members([t_declaration_id]);
        let variants = resolver
            .declarations
            .add_declaration_members([some_declaration_id, none_declaration_id]);

        resolver.declarations.set_declaration(
            option_declaration_id,
            Declaration {
                symbol_id: option_symbol,
                definition: Definition::EnumType {
                    public: true,
                    type_parameters,
                    variants,
                },
                scope_id: ScopeId::CORE,
                syntax: None,
            },
        );
    }

    {
        let result_symbol = resolver.symbols.add_symbol("Result");
        let ok_symbol = resolver.symbols.add_symbol("Ok");
        let err_symbol = resolver.symbols.add_symbol("Err");
        let e_symbol = resolver.symbols.add_symbol("E");

        let result_declaration_id = resolver.declarations.reserve_declaration_id();

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let e_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: e_symbol,
            definition: Definition::TypeParameter,
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });
        let e_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: e_declaration_id,
        });

        let ok_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: field_0_symbol,
            definition: Definition::Field {
                public: false,
                parent_struct: result_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let ok_fields = resolver
            .declarations
            .add_declaration_members([ok_field_declaration_id]);

        let ok_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: ok_symbol,
            definition: Definition::Variant {
                discriminant: 0,
                parent_enum: result_declaration_id,
                type_parameters: DeclarationMembers::default(),
                fields: ok_fields,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let err_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: field_0_symbol,
            definition: Definition::Field {
                public: false,
                parent_struct: result_declaration_id,
                type_id: e_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let err_fields = resolver
            .declarations
            .add_declaration_members([err_field_declaration_id]);

        let err_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: err_symbol,
            definition: Definition::Variant {
                discriminant: 1,
                parent_enum: result_declaration_id,
                type_parameters: DeclarationMembers::default(),
                fields: err_fields,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let type_parameters = resolver
            .declarations
            .add_declaration_members([t_declaration_id, e_declaration_id]);
        let variants = resolver
            .declarations
            .add_declaration_members([ok_declaration_id, err_declaration_id]);

        resolver.declarations.set_declaration(
            result_declaration_id,
            Declaration {
                symbol_id: result_symbol,
                definition: Definition::EnumType {
                    public: true,
                    type_parameters,
                    variants,
                },
                scope_id: ScopeId::CORE,
                syntax: None,
            },
        );
    }

    let start_symbol = resolver.symbols.add_symbol("start");
    let end_symbol = resolver.symbols.add_symbol("end");
    let last_symbol = resolver.symbols.add_symbol("last");

    {
        let range_symbol = resolver.symbols.add_symbol("Range");

        let range_declaration_id = resolver.declarations.reserve_declaration_id();

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });

        let start_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: start_symbol,
            definition: Definition::Field {
                public: true,
                parent_struct: range_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let end_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: end_symbol,
            definition: Definition::Field {
                public: true,
                parent_struct: range_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let type_parameters = resolver
            .declarations
            .add_declaration_members([t_declaration_id]);
        let fields = resolver
            .declarations
            .add_declaration_members([start_field_declaration_id, end_field_declaration_id]);

        resolver.declarations.set_declaration(
            range_declaration_id,
            Declaration {
                symbol_id: range_symbol,
                definition: Definition::StructType {
                    public: true,
                    type_parameters,
                    fields,
                },
                scope_id: ScopeId::CORE,
                syntax: None,
            },
        );
    }

    {
        let range_inclusive_symbol = resolver.symbols.add_symbol("RangeInclusive");

        let range_inclusive_declaration_id = resolver.declarations.reserve_declaration_id();

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });

        let start_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: start_symbol,
            definition: Definition::Field {
                public: true,
                parent_struct: range_inclusive_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let last_field_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: last_symbol,
            definition: Definition::Field {
                public: true,
                parent_struct: range_inclusive_declaration_id,
                type_id: t_type_id,
            },
            scope_id: ScopeId::CORE,
            syntax: None,
        });

        let type_parameters = resolver
            .declarations
            .add_declaration_members([t_declaration_id]);
        let fields = resolver
            .declarations
            .add_declaration_members([start_field_declaration_id, last_field_declaration_id]);

        resolver.declarations.set_declaration(
            range_inclusive_declaration_id,
            Declaration {
                symbol_id: range_inclusive_symbol,
                definition: Definition::StructType {
                    public: true,
                    type_parameters,
                    fields,
                },
                scope_id: ScopeId::CORE,
                syntax: None,
            },
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConstantValue {
    Boolean(bool),
    Character(char),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
}

impl ConstantValue {
    pub fn operand_type(&self) -> OperandType {
        match self {
            ConstantValue::Boolean(_) => OperandType::BOOLEAN,
            ConstantValue::Character(_) => OperandType::CHARACTER,
            ConstantValue::U8(_) => OperandType::U_8,
            ConstantValue::I8(_) => OperandType::I_8,
            ConstantValue::U16(_) => OperandType::U_16,
            ConstantValue::I16(_) => OperandType::I_16,
            ConstantValue::U32(_) => OperandType::U_32,
            ConstantValue::I32(_) => OperandType::I_32,
            ConstantValue::U64(_) => OperandType::U_64,
            ConstantValue::I64(_) => OperandType::I_64,
            ConstantValue::U128(_) => OperandType::U_128,
            ConstantValue::I128(_) => OperandType::I_128,
            ConstantValue::F32(_) => OperandType::F_32,
            ConstantValue::F64(_) => OperandType::F_64,
        }
    }

    pub fn type_id(&self) -> TypeId {
        match self {
            ConstantValue::Boolean(_) => TypeId::BOOLEAN,
            ConstantValue::Character(_) => TypeId::CHARACTER,
            ConstantValue::U8(_) => TypeId::U_8,
            ConstantValue::I8(_) => TypeId::I_8,
            ConstantValue::U16(_) => TypeId::U_16,
            ConstantValue::I16(_) => TypeId::I_16,
            ConstantValue::U32(_) => TypeId::U_32,
            ConstantValue::I32(_) => TypeId::I_32,
            ConstantValue::U64(_) => TypeId::U_64,
            ConstantValue::I64(_) => TypeId::I_64,
            ConstantValue::U128(_) => TypeId::U_128,
            ConstantValue::I128(_) => TypeId::I_128,
            ConstantValue::F32(_) => TypeId::F_32,
            ConstantValue::F64(_) => TypeId::F_64,
        }
    }

    pub fn as_encoded_u16(self) -> Option<u16> {
        match self {
            ConstantValue::Boolean(boolean) => Some(boolean as u16),
            ConstantValue::Character(character) => Some(character as u16),
            ConstantValue::I8(integer) => Some(integer as i16 as u16),
            ConstantValue::I16(integer) => Some(integer as u16),
            ConstantValue::I32(integer) if integer <= u16::MAX as i32 => {
                Some(integer as i16 as u16)
            }
            ConstantValue::I64(integer) if integer <= u16::MAX as i64 => {
                Some(integer as i16 as u16)
            }
            ConstantValue::I128(integer) if integer <= u16::MAX as i128 => {
                Some(integer as i16 as u16)
            }
            ConstantValue::U8(integer) => Some(integer as u16),
            ConstantValue::U16(integer) => Some(integer),
            ConstantValue::U32(integer) if integer <= u16::MAX as u32 => Some(integer as u16),
            ConstantValue::U64(integer) if integer <= u16::MAX as u64 => Some(integer as u16),
            ConstantValue::U128(integer) if integer <= u16::MAX as u128 => Some(integer as u16),
            _ => None,
        }
    }

    pub fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left + right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left + right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left + right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left + right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left + right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left + right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::U64(left + right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left + right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left + right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left + right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left + right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left + right))
            }
            _ => None,
        }
    }

    pub fn subtract(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left - right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left - right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left - right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left - right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left - right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left - right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::U64(left - right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left - right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left - right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left - right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left - right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left - right))
            }
            _ => None,
        }
    }

    pub fn multiply(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left * right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left * right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left * right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left * right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left * right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left * right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::U64(left * right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left * right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left * right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left * right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left * right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left * right))
            }
            _ => None,
        }
    }

    pub fn divide(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left / right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left / right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left / right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left / right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left / right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left / right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::U64(left / right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left / right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left / right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left / right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left / right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left / right))
            }
            _ => None,
        }
    }

    pub fn modulo(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left % right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left % right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left % right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left % right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left % right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left % right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::U64(left % right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left % right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left % right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left % right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left % right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left % right))
            }
            _ => None,
        }
    }

    pub fn power(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::U8(left.pow(right as u32)))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::I8(left.pow(right as u32)))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::U16(left.pow(right as u32)))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::I16(left.pow(right as u32)))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U32(left.pow(right)))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::I32(left.pow(right as u32)))
            }
            (ConstantValue::U64(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::U64(left.pow(right)))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::I64(left.pow(right as u32)))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::U128(left.pow(right as u32)))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::I128(left.pow(right as u32)))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::F32(left.powf(right)))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::F64(left.powf(right)))
            }
            _ => None,
        }
    }

    pub fn equal(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::Character(left), ConstantValue::Character(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            _ => None,
        }
    }

    pub fn not_equal(self, other: Self) -> Option<Self> {
        self.equal(other).and_then(|constant| constant.negate())
    }

    pub fn less(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::Character(left), ConstantValue::Character(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::Boolean(left < right))
            }
            _ => None,
        }
    }

    pub fn greater(self, other: Self) -> Option<Self> {
        self.less(other).and_then(|less| less.negate())
    }

    pub fn less_equal(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::Character(left), ConstantValue::Character(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::U8(left), ConstantValue::U8(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::I8(left), ConstantValue::I8(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::U16(left), ConstantValue::U16(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::I16(left), ConstantValue::I16(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::U32(left), ConstantValue::U32(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::I32(left), ConstantValue::I32(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Some(ConstantValue::Boolean(left <= right))
            }
            _ => None,
        }
    }

    pub fn greater_equal(self, other: Self) -> Option<Self> {
        self.less(other).and_then(|less| less.negate())
    }

    pub fn and(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Some(ConstantValue::Boolean(left && right))
            }
            _ => None,
        }
    }

    pub fn or(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Some(ConstantValue::Boolean(left || right))
            }
            _ => None,
        }
    }

    pub fn negate(self) -> Option<Self> {
        match self {
            ConstantValue::Boolean(boolean) => Some(ConstantValue::Boolean(!boolean)),
            ConstantValue::U8(integer) => Some(ConstantValue::U8(integer.wrapping_neg())),
            ConstantValue::I8(integer) => Some(ConstantValue::I8(integer.wrapping_neg())),
            ConstantValue::U16(integer) => Some(ConstantValue::U16(integer.wrapping_neg())),
            ConstantValue::I16(integer) => Some(ConstantValue::I16(integer.wrapping_neg())),
            ConstantValue::U32(integer) => Some(ConstantValue::U32(integer.wrapping_neg())),
            ConstantValue::I32(integer) => Some(ConstantValue::I32(integer.wrapping_neg())),
            ConstantValue::U64(integer) => Some(ConstantValue::U64(integer.wrapping_neg())),
            ConstantValue::I64(integer) => Some(ConstantValue::I64(integer.wrapping_neg())),
            ConstantValue::U128(integer) => Some(ConstantValue::U128(integer.wrapping_neg())),
            ConstantValue::I128(integer) => Some(ConstantValue::I128(integer.wrapping_neg())),
            _ => None,
        }
    }
}
