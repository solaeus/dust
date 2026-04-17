pub mod declarations;
pub mod scopes;
pub mod symbols;
pub mod types;

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            declarations::{
                Declaration, DeclarationId, DeclarationMembers, Declarations, Definition,
            },
            scopes::{Scope, ScopeFrame, ScopeId, ScopeKind, Scopes},
            symbols::{SymbolId, Symbols},
            types::{
                FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId, TypeMembers,
                Types, UnsignedIntegerType,
            },
        },
    },
    constants::value::ConstantValue,
    dust_type::{DustEnumType, DustFunctionType, DustStructType, DustStructTypeFields, DustType},
    instruction::OperandType,
    prototype::Prototype,
    source::Source,
    syntax::{SyntaxId, components::FunctionType},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: Symbols,
    pub declarations: Declarations,
    pub scopes: Scopes,
    pub types: Types,
    pub type_parameter_map: HashMap<DeclarationId, TypeId>,

    prototypes: Vec<Prototype>,
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
            prototypes: Vec::new(),
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

    pub fn into_prototypes(self) -> Vec<Prototype> {
        self.prototypes
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

    pub fn reserve_prototype_id(&mut self) -> PrototypeId {
        let id = PrototypeId(self.prototypes.len() as u16);

        self.prototypes.push(Prototype::placeholder());

        id
    }

    pub fn set_prototype(&mut self, prototype_id: PrototypeId, prototype: Prototype) {
        self.prototypes[prototype_id.0 as usize] = prototype;
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

    pub fn get_operand_types(
        &self,
        type_id: TypeId,
    ) -> Result<OperandType::SmallVec, CompileError> {
        let r#type = self.types.get_type(type_id)?;

        match r#type {
            Type::Never => Ok(SmallVec::new()),
            Type::Boolean => Ok(smallvec![OperandType::BOOLEAN]),
            Type::Character => Ok(smallvec![OperandType::CHARACTER]),
            Type::SignedInteger(SignedIntegerType::I8) => Ok(smallvec![OperandType::I_8]),
            Type::SignedInteger(SignedIntegerType::I16) => Ok(smallvec![OperandType::I_16]),
            Type::SignedInteger(SignedIntegerType::I32) => Ok(smallvec![OperandType::I_32]),
            Type::SignedInteger(SignedIntegerType::I64) => Ok(smallvec![OperandType::I_64]),
            Type::SignedInteger(SignedIntegerType::I128) => Ok(smallvec![OperandType::I_128]),
            Type::SignedInteger(SignedIntegerType::ISize) => {
                #[cfg(target_pointer_width = "64")]
                {
                    Ok(smallvec![OperandType::I_64])
                }

                #[cfg(target_pointer_width = "32")]
                {
                    Ok(smallvec![OperandType::I_32])
                }
            }
            Type::UnsignedInteger(UnsignedIntegerType::U8) => Ok(smallvec![OperandType::U_8]),
            Type::UnsignedInteger(UnsignedIntegerType::U16) => Ok(smallvec![OperandType::U_16]),
            Type::UnsignedInteger(UnsignedIntegerType::U32) => Ok(smallvec![OperandType::U_32]),
            Type::UnsignedInteger(UnsignedIntegerType::U64) => Ok(smallvec![OperandType::U_64]),
            Type::UnsignedInteger(UnsignedIntegerType::U128) => Ok(smallvec![OperandType::U_128]),
            Type::UnsignedInteger(UnsignedIntegerType::USize) => {
                #[cfg(target_pointer_width = "64")]
                {
                    Ok(smallvec![OperandType::U_64])
                }

                #[cfg(target_pointer_width = "32")]
                {
                    Ok(smallvec![OperandType::U_32])
                }
            }
            Type::Float(FloatType::F32) => Ok(smallvec![OperandType::F_32]),
            Type::Float(FloatType::F64) => Ok(smallvec![OperandType::F_64]),
            Type::Tuple { element_type_ids } => {
                let element_type_ids = self.types.get_type_members(*element_type_ids)?;
                let mut operand_types = SmallVec::with_capacity(element_type_ids.len());

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
                let mut operand_types =
                    SmallVec::with_capacity(element_operand_types.len() * length);

                for _ in 0..*length {
                    operand_types.extend(element_operand_types.iter().copied());
                }

                Ok(operand_types)
            }
            Type::Slice { .. } | Type::Pointer { .. } => Ok(smallvec![OperandType::POINTER]),
            Type::FunctionDefinition { .. } | Type::Closure { .. } | Type::Function { .. } => {
                Ok(smallvec![OperandType::FUNCTION])
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

                        let mut largest_variant_operand_types: Vec<OperandType> = Vec::new();
                        let mut largest_variant_register_count: u16 = 0;

                        for variant_declaration_id in variant_declaration_ids {
                            let variant_declaration =
                                self.declarations.get_declaration(*variant_declaration_id)?;
                            let Definition::Variant { fields, .. } =
                                &variant_declaration.definition
                            else {
                                return Err(CompileError::ExpectedVariantDefinition(
                                    *variant_declaration_id,
                                ));
                            };

                            let field_declaration_ids =
                                self.declarations.get_declaration_members(fields)?;

                            let mut variant_operand_types: Vec<OperandType> = Vec::new();
                            let mut variant_register_count: u16 = 0;

                            for field_declaration_id in field_declaration_ids {
                                let field_declaration =
                                    self.declarations.get_declaration(*field_declaration_id)?;
                                let Definition::Field {
                                    type_id: field_type_id,
                                    ..
                                } = field_declaration.definition
                                else {
                                    return Err(CompileError::ExpectedFieldDefinition(
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

                                for operand_type in &field_operand_types {
                                    variant_register_count +=
                                        operand_type.register_width().as_u16();
                                }

                                variant_operand_types.extend(field_operand_types);
                            }

                            if variant_register_count > largest_variant_register_count {
                                largest_variant_register_count = variant_register_count;
                                largest_variant_operand_types = variant_operand_types;
                            }
                        }

                        let mut operand_types =
                            SmallVec::with_capacity(largest_variant_operand_types.len() + 1);

                        operand_types.push(OperandType::U_16);
                        operand_types.extend(largest_variant_operand_types);

                        Ok(operand_types)
                    }
                    Definition::StructType {
                        fields,
                        type_parameters,
                        ..
                    } => {
                        let mut operand_types = SmallVec::with_capacity(fields.len() as usize);

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
            } => Ok(smallvec![OperandType::I_32]),
            Type::Inferred {
                constraint: Some(InferredTypeConstraint::Float),
                resolved: None,
                ..
            } => Ok(smallvec![OperandType::F_64]),
            Type::Inferred { .. } => Err(CompileError::ExpectedConcreteType),
        }
    }

    pub fn add_external_types(&mut self, types: &[(String, DustType)]) -> Result<(), CompileError> {
        let external_scope_id = self.scopes.enter_scope(ScopeKind::Module, ScopeId::NONE);

        let mut type_bindings = Vec::with_capacity(types.len());

        for (symbol, r#type) in types {
            let symbol_id = self.symbols.add_symbol(symbol);
            let type_id = self.add_external_type(r#type, external_scope_id);
            let declaration_id = self.declarations.add_declaration(Declaration {
                symbol_id,
                definition: Definition::TypeAlias {
                    public: true,
                    aliased_type_id: type_id,
                    type_parameters: DeclarationMembers::default(),
                },
                scope_id: external_scope_id,
                syntax: None,
            });

            type_bindings.push((symbol_id, declaration_id));
        }

        self.scopes.exit_scope(external_scope_id, type_bindings);

        Ok(())
    }

    fn add_external_type(&mut self, new_type: &DustType, scope_id: ScopeId) -> TypeId {
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
                let type_scope_id = self.scopes.enter_scope(ScopeKind::Type, scope_id);
                let element_type_ids = element_types
                    .iter()
                    .map(|element_type| self.add_external_type(element_type, type_scope_id))
                    .collect::<TypeId::SmallVec>();
                let element_type_ids = self.types.add_type_members(element_type_ids);

                self.types.add_type(Type::Tuple { element_type_ids })
            }
            DustType::Array(element_type, length) => {
                let element_type_id = self.add_external_type(element_type, scope_id);

                self.types.add_type(Type::Array {
                    element_type_id,
                    length: *length,
                })
            }
            DustType::Slice(element_type) => {
                let element_type_id = self.add_external_type(element_type, scope_id);
                let declaration_id = self.declarations.add_declaration(Declaration {
                    symbol_id: self.symbols.add_slice_symbol(),
                    definition: Definition::TypeParameter,
                    scope_id,
                    syntax: None,
                });

                self.types.add_type(Type::Slice {
                    declaration_id,
                    element_type_id,
                })
            }
            DustType::Function(function_type) => {
                let DustFunctionType {
                    value_parameters,
                    return_type,
                } = function_type.as_ref();
                let value_parameter_ids = value_parameters
                    .iter()
                    .map(|parameter_type| self.add_external_type(parameter_type, scope_id))
                    .collect::<TypeId::SmallVec>();
                let value_parameters = self.types.add_type_members(value_parameter_ids);
                let return_type_id = self.add_external_type(return_type, scope_id);

                self.types.add_type(Type::Function {
                    value_parameters,
                    return_type_id,
                })
            }
            DustType::Struct(struct_type) => {
                let DustStructType { name, value_type } = struct_type.as_ref();
                let struct_symbol_id = self.symbols.add_symbol(name);
                let struct_scope_id = self.scopes.enter_scope(ScopeKind::Type, ScopeId::NONE);
                let struct_declaration_id = self.declarations.reserve_declaration_id(
                    struct_symbol_id,
                    struct_scope_id,
                    None,
                );

                let mut type_bindings = Vec::new();

                let fields = match value_type {
                    DustStructTypeFields::Unit => DeclarationMembers::default(),
                    DustStructTypeFields::Tuple(types) => {
                        let field_ids = types
                            .iter()
                            .enumerate()
                            .map(|(index, field_type)| {
                                let field_type_id =
                                    self.add_external_type(field_type, struct_scope_id);
                                let field_symbol_id = self.symbols.add_index_symbol(index as u32);
                                let field_declaration_id =
                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: struct_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: struct_scope_id,
                                        syntax: None,
                                    });

                                type_bindings.push((field_symbol_id, field_declaration_id));

                                field_declaration_id
                            })
                            .collect::<DeclarationId::SmallVec>();

                        self.declarations.add_declaration_members(field_ids)
                    }
                    DustStructTypeFields::Named(fields) => {
                        let field_ids = fields
                            .iter()
                            .map(|(field_name, field_type)| {
                                let field_type_id =
                                    self.add_external_type(field_type, struct_scope_id);
                                let field_symbol_id = self.symbols.add_symbol(field_name);
                                let field_declaration_id =
                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: struct_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: struct_scope_id,
                                        syntax: None,
                                    });

                                type_bindings.push((field_symbol_id, field_declaration_id));

                                field_declaration_id
                            })
                            .collect::<DeclarationId::SmallVec>();

                        self.declarations.add_declaration_members(field_ids)
                    }
                };

                self.scopes.exit_scope(struct_scope_id, type_bindings);
                self.declarations.set_reserved_declaration(
                    struct_declaration_id,
                    Definition::StructType {
                        public: true,
                        type_parameters: DeclarationMembers::default(),
                        fields,
                        inner_scope_id: struct_scope_id,
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
                let enum_scope_id = self.scopes.enter_scope(ScopeKind::Type, ScopeId::NONE);
                let enum_declaration_id =
                    self.declarations
                        .reserve_declaration_id(enum_symbol_id, scope_id, None);
                let variant_fields = variants
                    .iter()
                    .enumerate()
                    .map(|(discriminant, (variant_name, variant_value_type))| {
                        let variant_symbol_id = self.symbols.add_symbol(variant_name);
                        let fields = match variant_value_type {
                            DustStructTypeFields::Unit => DeclarationMembers::default(),
                            DustStructTypeFields::Tuple(types) => {
                                let enum_variant_scope_id =
                                    self.scopes.enter_scope(ScopeKind::Type, enum_scope_id);

                                let mut type_bindings = Vec::new();

                                let field_declaration_ids = types
                                    .iter()
                                    .enumerate()
                                    .map(|(index, field_type)| {
                                        let field_symbol_id =
                                            self.symbols.add_index_symbol(index as u32);
                                        let field_type_id = self
                                            .add_external_type(field_type, enum_variant_scope_id);
                                        let field_declaration_id =
                                            self.declarations.add_declaration(Declaration {
                                                symbol_id: field_symbol_id,
                                                definition: Definition::Field {
                                                    public: true,
                                                    parent_struct: enum_declaration_id,
                                                    type_id: field_type_id,
                                                },
                                                scope_id: ScopeId::NONE,
                                                syntax: None,
                                            });

                                        type_bindings.push((field_symbol_id, field_declaration_id));

                                        field_declaration_id
                                    })
                                    .collect::<DeclarationId::SmallVec>();

                                self.scopes.exit_scope(enum_variant_scope_id, type_bindings);
                                self.declarations
                                    .add_declaration_members(field_declaration_ids)
                            }
                            DustStructTypeFields::Named(fields) => {
                                let enum_variant_scope_id =
                                    self.scopes.enter_scope(ScopeKind::Type, enum_scope_id);

                                let mut type_bindings = Vec::new();

                                let field_declaration_ids = fields
                                    .iter()
                                    .map(|(field_name, field_type)| {
                                        let field_symbol_id = self.symbols.add_symbol(field_name);
                                        let field_type_id = self
                                            .add_external_type(field_type, enum_variant_scope_id);
                                        let field_declaration_id =
                                            self.declarations.add_declaration(Declaration {
                                                symbol_id: field_symbol_id,
                                                definition: Definition::Field {
                                                    public: true,
                                                    parent_struct: enum_declaration_id,
                                                    type_id: field_type_id,
                                                },
                                                scope_id: ScopeId::NONE,
                                                syntax: None,
                                            });

                                        type_bindings.push((field_symbol_id, field_declaration_id));

                                        field_declaration_id
                                    })
                                    .collect::<DeclarationId::SmallVec>();

                                self.scopes.exit_scope(enum_variant_scope_id, type_bindings);
                                self.declarations
                                    .add_declaration_members(field_declaration_ids)
                            }
                        };

                        self.declarations.add_declaration(Declaration {
                            symbol_id: variant_symbol_id,
                            definition: Definition::Variant {
                                discriminant: discriminant as u16,
                                enum_declaration_id,
                                fields,
                            },
                            scope_id: ScopeId::NONE,
                            syntax: None,
                        })
                    })
                    .collect::<DeclarationId::SmallVec>();

                let variants = self.declarations.add_declaration_members(variant_fields);

                self.declarations.set_reserved_declaration(
                    enum_declaration_id,
                    Definition::EnumType {
                        public: true,
                        type_parameters: DeclarationMembers::default(),
                        variants,
                    },
                );

                self.types.add_type(Type::Algebraic {
                    declaration_id: enum_declaration_id,
                    type_arguments: TypeMembers::default(),
                })
            }
        }
    }

    pub fn get_external_type(&self, id: TypeId) -> Result<DustType, CompileError> {
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
                        .get_type_members_as_full_types(*element_type_ids)
                        .collect::<Result<_, _>>()?;

                    Ok(DustType::Tuple(element_types))
                }
            }
            Type::Inferred {
                resolved: Some(resolved),
                ..
            } => self.get_external_type(*resolved),
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

                                let field_dust_type = self.get_external_type(resolved_type_id)?;
                                let field_name = self
                                    .symbols
                                    .get_symbol(&field_declaration.symbol_id)
                                    .map(|symbol| symbol.to_string())
                                    .unwrap_or_else(|_| field_index.to_string());

                                field_types.push((field_name, field_dust_type));
                            }

                            let value_type = if field_types.is_empty() {
                                DustStructTypeFields::Unit
                            } else if field_types.iter().all(|(name, _)| name == "0") {
                                DustStructTypeFields::Tuple(
                                    field_types
                                        .into_iter()
                                        .map(|(_, field_type)| field_type)
                                        .collect(),
                                )
                            } else {
                                DustStructTypeFields::Named(field_types)
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

                            let field_dust_type = self.get_external_type(field_type_id)?;
                            let field_name = self
                                .symbols
                                .get_symbol(&field_declaration.symbol_id)?
                                .to_string();

                            field_types.push((field_name, field_dust_type));
                        }

                        let value_type = if field_types.is_empty() {
                            DustStructTypeFields::Unit
                        } else if field_types.iter().all(|(name, _)| name == "0") {
                            DustStructTypeFields::Tuple(
                                field_types
                                    .into_iter()
                                    .map(|(_, field_type)| field_type)
                                    .collect(),
                            )
                        } else {
                            DustStructTypeFields::Named(field_types)
                        };

                        Ok(DustType::Struct(Box::new(DustStructType {
                            name: struct_name,
                            value_type,
                        })))
                    }
                    Definition::TypeAlias {
                        aliased_type_id, ..
                    } => self.get_external_type(*aliased_type_id),
                    _ => Err(CompileError::ExpectedAlgebraicTypeDefinition(
                        *declaration_id,
                    )),
                }
            }
            Type::Array {
                element_type_id,
                length,
            } => {
                let element_dust_type = self.get_external_type(*element_type_id)?;

                Ok(DustType::Array(Box::new(element_dust_type), *length))
            }
            Type::Generic { .. } => Err(CompileError::ExpectedConcreteType),
            Type::Function {
                value_parameters,
                return_type_id,
            } => {
                let value_parameter_types: Vec<DustType> = self
                    .get_type_members_as_full_types(*value_parameters)
                    .collect::<Result<_, _>>()?;
                let return_dust_type = self.get_external_type(*return_type_id)?;

                Ok(DustType::Function(Box::new(DustFunctionType {
                    value_parameters: value_parameter_types,
                    return_type: return_dust_type,
                })))
            }
            Type::Closure {
                value_parameters,
                return_type_id,
            } => {
                let value_parameter_types = self
                    .get_type_members_as_full_types(*value_parameters)
                    .collect::<Result<Vec<DustType>, CompileError>>()?;
                let return_dust_type = self.get_external_type(*return_type_id)?;

                Ok(DustType::Function(Box::new(DustFunctionType {
                    value_parameters: value_parameter_types,
                    return_type: return_dust_type,
                })))
            }
            Type::Slice {
                element_type_id, ..
            } => {
                let element_dust_type = self.get_external_type(*element_type_id)?;

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
                        let parameter_declaration_ids = self
                            .declarations
                            .get_declaration_members(value_parameters)?;
                        let value_parameters = parameter_declaration_ids
                            .iter()
                            .map(|parameter_declaration_id| {
                                let parameter_declaration = self
                                    .declarations
                                    .get_declaration(*parameter_declaration_id)?;
                                let type_id = if let Definition::Local { type_id, .. } =
                                    parameter_declaration.definition
                                {
                                    type_id
                                } else {
                                    return Err(CompileError::ExpectedLocalDefinition(
                                        *parameter_declaration_id,
                                    ));
                                };

                                self.get_external_type(type_id)
                            })
                            .try_collect::<Vec<DustType>>()?;

                        let return_dust_type = self.get_external_type(*return_type_id)?;

                        Ok(DustType::Function(Box::new(DustFunctionType {
                            value_parameters,
                            return_type: return_dust_type,
                        })))
                    }
                    Definition::NativeFunction {
                        value_parameters,
                        return_type_id,
                        type_parameters,
                        ..
                    } => {
                        todo!()
                    }
                    _ => Err(CompileError::ExpectedFunctionDefinition(*declaration_id)),
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
    ) -> impl Iterator<Item = Result<DustType, CompileError>> {
        members.as_range().map(|member_index| {
            let type_id = *self.types.get_type_member(member_index)?;

            self.get_external_type(type_id)
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PrototypeId(#[cfg(test)] pub(crate) u16, #[cfg(not(test))] u16);

impl PrototypeId {
    pub(crate) const MAIN: Self = Self(0);

    pub fn inner(self) -> u16 {
        self.0
    }

    pub fn index_usize(self) -> usize {
        self.0 as usize
    }
}

impl Display for PrototypeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "proto_{}", self.0)
    }
}

fn add_core(resolver: &mut Resolver) {
    let mut namespace_entries = Vec::new();

    fn enter_scope(
        kind: ScopeKind,
        parent: ScopeId,
        namespace_entries: &Vec<(SymbolId, DeclarationId)>,
        scopes: &mut Scopes,
    ) -> ScopeFrame {
        ScopeFrame {
            scope_id: scopes.enter_scope(kind, parent),
            type_entries_start: namespace_entries.len() as u32,
        }
    }

    let core_scope_frame = enter_scope(
        ScopeKind::Module,
        ScopeId::NONE,
        &namespace_entries,
        &mut resolver.scopes,
    );

    debug_assert_eq!(core_scope_frame.scope_id, ScopeId::CORE);

    let t_symbol = resolver.symbols.add_symbol("T");
    let field_0_symbol = resolver.symbols.add_index_symbol(0);

    {
        let option_symbol_id = resolver.symbols.add_symbol("Option");
        let option_declaration_id = resolver.declarations.reserve_declaration_id(
            option_symbol_id,
            core_scope_frame.scope_id,
            None,
        );

        namespace_entries.push((option_symbol_id, option_declaration_id));

        let option_scope_frame = enter_scope(
            ScopeKind::Type,
            core_scope_frame.scope_id,
            &namespace_entries,
            &mut resolver.scopes,
        );

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: option_scope_frame.scope_id,
            syntax: None,
        });
        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });

        namespace_entries.push((t_symbol, t_declaration_id));

        let none_declaration_id = {
            let none_symbol_id = resolver.symbols.add_symbol("None");
            let none_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: none_symbol_id,
                definition: Definition::Variant {
                    discriminant: 0,
                    enum_declaration_id: option_declaration_id,
                    fields: DeclarationMembers::default(),
                },
                scope_id: option_scope_frame.scope_id,
                syntax: None,
            });

            namespace_entries.push((none_symbol_id, none_declaration_id));

            none_declaration_id
        };
        let some_declaration_id = {
            let some_symbol_id = resolver.symbols.add_symbol("Some");
            let some_declaration_id = resolver.declarations.reserve_declaration_id(
                some_symbol_id,
                option_scope_frame.scope_id,
                None,
            );

            namespace_entries.push((some_symbol_id, some_declaration_id));

            let some_scope_frame = enter_scope(
                ScopeKind::Type,
                option_scope_frame.scope_id,
                &namespace_entries,
                &mut resolver.scopes,
            );
            let some_field_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: field_0_symbol,
                definition: Definition::Field {
                    public: false,
                    parent_struct: some_declaration_id,
                    type_id: t_type_id,
                },
                scope_id: some_scope_frame.scope_id,
                syntax: None,
            });
            let some_fields = resolver
                .declarations
                .add_declaration_members([some_field_declaration_id]);

            resolver.declarations.set_reserved_declaration(
                some_declaration_id,
                Definition::Variant {
                    discriminant: 1,
                    enum_declaration_id: option_declaration_id,
                    fields: some_fields,
                },
            );

            namespace_entries.push((field_0_symbol, some_field_declaration_id));
            resolver.scopes.exit_scope(
                some_scope_frame.scope_id,
                namespace_entries.drain(some_scope_frame.type_entries_start as usize..),
            );

            some_declaration_id
        };

        let type_parameters = resolver
            .declarations
            .add_declaration_members([t_declaration_id]);
        let variants = resolver
            .declarations
            .add_declaration_members([some_declaration_id, none_declaration_id]);

        resolver.declarations.set_reserved_declaration(
            option_declaration_id,
            Definition::EnumType {
                public: true,
                type_parameters,
                variants,
            },
        );
    }

    {
        let result_symbol = resolver.symbols.add_symbol("Result");
        let result_declaration_id = resolver.declarations.reserve_declaration_id(
            result_symbol,
            core_scope_frame.scope_id,
            None,
        );

        namespace_entries.push((result_symbol, result_declaration_id));

        let result_scope_frame = enter_scope(
            ScopeKind::Type,
            core_scope_frame.scope_id,
            &namespace_entries,
            &mut resolver.scopes,
        );

        let t_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: t_symbol,
            definition: Definition::TypeParameter,
            scope_id: result_scope_frame.scope_id,
            syntax: None,
        });
        let t_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: t_declaration_id,
        });

        namespace_entries.push((t_symbol, t_declaration_id));

        let e_symbol = resolver.symbols.add_symbol("E");
        let e_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: e_symbol,
            definition: Definition::TypeParameter,
            scope_id: result_scope_frame.scope_id,
            syntax: None,
        });
        let e_type_id = resolver.types.add_type(Type::Generic {
            declaration_id: e_declaration_id,
        });

        namespace_entries.push((e_symbol, e_declaration_id));

        let ok_declaration_id = {
            let ok_symbol = resolver.symbols.add_symbol("Ok");
            let ok_declaration_id = resolver.declarations.reserve_declaration_id(
                ok_symbol,
                result_scope_frame.scope_id,
                None,
            );

            namespace_entries.push((ok_symbol, ok_declaration_id));

            let ok_scope_frame = enter_scope(
                ScopeKind::Type,
                result_scope_frame.scope_id,
                &namespace_entries,
                &mut resolver.scopes,
            );
            let ok_field_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: field_0_symbol,
                definition: Definition::Field {
                    public: false,
                    parent_struct: ok_declaration_id,
                    type_id: t_type_id,
                },
                scope_id: ok_scope_frame.scope_id,
                syntax: None,
            });

            namespace_entries.push((field_0_symbol, ok_field_declaration_id));

            let ok_fields = resolver
                .declarations
                .add_declaration_members([ok_field_declaration_id]);

            resolver.declarations.set_reserved_declaration(
                ok_declaration_id,
                Definition::Variant {
                    discriminant: 0,
                    enum_declaration_id: result_declaration_id,
                    fields: ok_fields,
                },
            );
            resolver.scopes.exit_scope(
                ok_scope_frame.scope_id,
                namespace_entries.drain(ok_scope_frame.type_entries_start as usize..),
            );

            ok_declaration_id
        };
        let err_declaration_id = {
            let err_symbol = resolver.symbols.add_symbol("Err");
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

            namespace_entries.push((field_0_symbol, err_field_declaration_id));

            let err_fields = resolver
                .declarations
                .add_declaration_members([err_field_declaration_id]);

            let err_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: err_symbol,
                definition: Definition::Variant {
                    discriminant: 1,
                    enum_declaration_id: result_declaration_id,
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

            resolver.declarations.set_reserved_declaration(
                result_declaration_id,
                Definition::EnumType {
                    public: true,
                    type_parameters,
                    variants,
                },
            );
            resolver.scopes.exit_scope(
                result_scope_frame.scope_id,
                namespace_entries.drain(result_scope_frame.type_entries_start as usize..),
            );

            err_declaration_id
        };

        let start_symbol = resolver.symbols.add_symbol("start");
        let end_symbol = resolver.symbols.add_symbol("end");
        let last_symbol = resolver.symbols.add_symbol("last");

        {
            let range_symbol = resolver.symbols.add_symbol("Range");
            let range_declaration_id = resolver.declarations.reserve_declaration_id(
                range_symbol,
                core_scope_frame.scope_id,
                None,
            );

            namespace_entries.push((range_symbol, range_declaration_id));

            let range_scope_frame = enter_scope(
                ScopeKind::Type,
                core_scope_frame.scope_id,
                &namespace_entries,
                &mut resolver.scopes,
            );

            let t_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: t_symbol,
                definition: Definition::TypeParameter,
                scope_id: range_scope_frame.scope_id,
                syntax: None,
            });
            let t_type_id = resolver.types.add_type(Type::Generic {
                declaration_id: t_declaration_id,
            });

            namespace_entries.push((t_symbol, t_declaration_id));

            let start_field_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: start_symbol,
                definition: Definition::Field {
                    public: true,
                    parent_struct: range_declaration_id,
                    type_id: t_type_id,
                },
                scope_id: range_scope_frame.scope_id,
                syntax: None,
            });

            namespace_entries.push((start_symbol, start_field_declaration_id));

            let end_field_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: end_symbol,
                definition: Definition::Field {
                    public: true,
                    parent_struct: range_declaration_id,
                    type_id: t_type_id,
                },
                scope_id: range_scope_frame.scope_id,
                syntax: None,
            });

            namespace_entries.push((end_symbol, end_field_declaration_id));

            let type_parameters = resolver
                .declarations
                .add_declaration_members([t_declaration_id]);
            let fields = resolver
                .declarations
                .add_declaration_members([start_field_declaration_id, end_field_declaration_id]);

            resolver.declarations.set_reserved_declaration(
                range_declaration_id,
                Definition::StructType {
                    public: true,
                    type_parameters,
                    fields,
                    inner_scope_id: range_scope_frame.scope_id,
                },
            );
            resolver.scopes.exit_scope(
                range_scope_frame.scope_id,
                namespace_entries.drain(range_scope_frame.type_entries_start as usize..),
            );
        }

        {
            let range_inclusive_symbol = resolver.symbols.add_symbol("RangeInclusive");
            let range_inclusive_declaration_id = resolver.declarations.reserve_declaration_id(
                range_inclusive_symbol,
                core_scope_frame.scope_id,
                None,
            );

            namespace_entries.push((range_inclusive_symbol, range_inclusive_declaration_id));

            let range_inclusive_scope_frame = enter_scope(
                ScopeKind::Type,
                core_scope_frame.scope_id,
                &namespace_entries,
                &mut resolver.scopes,
            );

            let t_declaration_id = resolver.declarations.add_declaration(Declaration {
                symbol_id: t_symbol,
                definition: Definition::TypeParameter,
                scope_id: range_inclusive_scope_frame.scope_id,
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

            resolver.declarations.set_reserved_declaration(
                range_inclusive_declaration_id,
                Definition::StructType {
                    public: true,
                    type_parameters,
                    fields,
                    inner_scope_id: range_inclusive_scope_frame.scope_id,
                },
            );
            resolver.scopes.exit_scope(
                range_inclusive_scope_frame.scope_id,
                namespace_entries.drain(range_inclusive_scope_frame.type_entries_start as usize..),
            );
        }
    }
}

#[derive(Clone, Copy)]
enum BuiltInType<'a> {
    Struct {
        name: &'a str,
        type_parameters: &'a [&'a str],
        fields: BuiltInStructFields<'a>,
    },
    Enum {
        name: &'a str,
        type_parameters: &'a [&'a str],
        variants: &'a [(&'a str, BuiltInStructFields<'a>)],
    },
    Generic {
        parameter_index: usize,
    },
}

#[derive(Clone, Copy)]
enum BuiltInStructFields<'a> {
    Unit,
    Tuple(&'a [BuiltInType<'a>]),
    Named(&'a [(&'a str, BuiltInType<'a>)]),
}
