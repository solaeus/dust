pub mod declarations;
pub mod scopes;
pub mod symbols;
pub mod types;

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            declarations::{Declaration, DeclarationId, Declarations, Definition, VariantKind},
            scopes::{Barrier, ScopeId, Scopes},
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
    syntax::{SyntaxId, reader::SyntaxReader},
};

#[derive(Debug)]
pub struct Resolver {
    pub symbols: Symbols,
    pub declarations: Declarations,
    pub scopes: Scopes,
    pub types: Types,
    pub type_parameter_map: HashMap<DeclarationId, TypeId, FxBuildHasher>,
    pub implementations: HashMap<DeclarationId, DeclarationId::SmallVec, FxBuildHasher>,

    prototypes: Vec<Prototype>,
    compilation_stack: Vec<PrototypeId>,
    monomorphization_cache: IndexSet<(DeclarationId, TypeId::SmallVec), FxBuildHasher>,

    declaration_bindings: HashMap<SyntaxId, DeclarationId, FxBuildHasher>,
    type_bindings: HashMap<SyntaxId, TypeId, FxBuildHasher>,

    constant_item_values: HashMap<DeclarationId, ConstantValue, FxBuildHasher>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            symbols: Symbols::new(),
            declarations: Declarations::new(),
            scopes: Scopes::new(),
            types: Types::new(),
            type_parameter_map: HashMap::default(),
            implementations: HashMap::default(),
            compilation_stack: Vec::new(),
            prototypes: Vec::new(),
            monomorphization_cache: IndexSet::default(),
            declaration_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
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

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, CompileError> {
        self.type_bindings
            .get(syntax_id)
            .ok_or(CompileError::MissingTypeBinding(*syntax_id))
    }

    pub fn add_monomorphized_function(
        &mut self,
        declaration_id: DeclarationId,
        type_arguments: TypeId::SmallVec,
    ) -> PrototypeId {
        let cache_key = (declaration_id, type_arguments);

        if let Some(index) = self.monomorphization_cache.get_index_of(&cache_key) {
            PrototypeId(index as u16)
        } else {
            let prototype_id = PrototypeId(self.prototypes.len() as u16);

            self.prototypes.push(Prototype::placeholder());
            self.monomorphization_cache.insert(cache_key);
            self.compilation_stack.push(prototype_id);

            prototype_id
        }
    }

    pub fn pop_from_compilation_stack(&mut self) -> Option<PrototypeId> {
        self.compilation_stack.pop()
    }

    pub fn get_monomorphized_function(
        &self,
        prototype_id: PrototypeId,
    ) -> &(DeclarationId, TypeId::SmallVec) {
        &self.monomorphization_cache[prototype_id.0 as usize]
    }

    pub fn set_prototype(&mut self, prototype_id: PrototypeId, prototype: Prototype) {
        self.prototypes[prototype_id.0 as usize] = prototype;
    }

    pub fn add_constant_item_value(&mut self, declaration_id: DeclarationId, value: ConstantValue) {
        self.constant_item_values.insert(declaration_id, value);
    }

    pub fn get_constant_item_value(&self, declaration_id: &DeclarationId) -> Option<ConstantValue> {
        self.constant_item_values.get(declaration_id).copied()
    }

    pub fn get_signature(
        &mut self,
        function_declaration_id: DeclarationId,
        type_arguments: TypeMembers,
        self_type_id: Option<TypeId>,
    ) -> Result<(TypeId::SmallVec, TypeId), CompileError> {
        fn collect_value_parameter_type_ids(
            resolver: &mut Resolver,
            value_parameters: ScopeId,
            value_parameter_type_ids: &mut TypeId::SmallVec,
        ) -> Result<(), CompileError> {
            let value_parameter_ids = resolver.scopes.get_members(value_parameters);

            for value_parameter_id in value_parameter_ids {
                let value_parameter_declaration =
                    resolver.declarations.get_declaration(*value_parameter_id);
                let (Definition::Local { type_id, .. } | Definition::Field { type_id, .. }) =
                    value_parameter_declaration.definition
                else {
                    return Err(CompileError::ExpectedLocalDefinition(*value_parameter_id));
                };

                value_parameter_type_ids.push(type_id);
            }

            Ok(())
        }

        let mut value_parameter_type_ids = TypeId::SmallVec::new();

        let declaration = self.declarations.get_declaration(function_declaration_id);
        let return_type_id = match declaration.definition {
            Definition::Function {
                parent_impl_or_trait,
                type_parameters,
                value_parameters,
                return_type_id,
                ..
            } => {
                if let (Some(parent_impl_or_trait_id), Some(self_type_id)) =
                    (parent_impl_or_trait, self_type_id)
                {
                    let parent_declaration =
                        self.declarations.get_declaration(parent_impl_or_trait_id);

                    if let Definition::InherentImplementation {
                        declarations: Some(declarations),
                        ..
                    }
                    | Definition::Trait {
                        declarations: Some(declarations),
                        ..
                    }
                    | Definition::TraitImplementation {
                        declarations: Some(declarations),
                        ..
                    } = parent_declaration.definition
                    {
                        for &member_id in self.scopes.get_members(declarations) {
                            let member_declaration = self.declarations.get_declaration(member_id);

                            if matches!(
                                member_declaration.definition,
                                Definition::TypeParameter { is_self: true }
                            ) {
                                self.type_parameter_map.insert(member_id, self_type_id);

                                break;
                            }
                        }
                    }
                }

                if let Some(value_parameters) = value_parameters {
                    collect_value_parameter_type_ids(
                        self,
                        value_parameters,
                        &mut value_parameter_type_ids,
                    )?;
                }

                let type_parameter_ids = self
                    .get_type_parameter_ids(parent_impl_or_trait, type_parameters)?
                    .into_iter();
                let mut explicit_type_argument_ids = self
                    .types
                    .get_type_members(type_arguments)
                    .iter()
                    .copied()
                    .collect::<TypeId::SmallVec>()
                    .into_iter();

                for parameter_declaration_id in type_parameter_ids {
                    let argument_type_id = if let Some(explicit_type_argument_id) =
                        explicit_type_argument_ids.next()
                    {
                        explicit_type_argument_id
                    } else {
                        self.types.create_inferred_type(None)
                    };

                    self.type_parameter_map
                        .insert(parameter_declaration_id, argument_type_id);
                }

                self.infer_concrete_type_id(return_type_id)?
            }
            Definition::Variant {
                enum_declaration_id,
                fields: Some(fields),
                kind: VariantKind::TupleFields,
                ..
            } => {
                collect_value_parameter_type_ids(self, fields, &mut value_parameter_type_ids)?;

                self.types.add_type(Type::Algebraic {
                    declaration_id: enum_declaration_id,
                    type_arguments: TypeMembers::default(),
                })
            }
            _ => todo!("Error: {:?}", declaration.definition),
        };

        for type_id in value_parameter_type_ids.iter_mut() {
            *type_id = self.infer_concrete_type_id(*type_id)?;
        }

        Ok((value_parameter_type_ids, return_type_id))
    }

    pub fn get_type_parameter_ids(
        &mut self,
        parent_impl_or_trait: Option<DeclarationId>,
        type_parameters: Option<ScopeId>,
    ) -> Result<DeclarationId::SmallVec, CompileError> {
        if parent_impl_or_trait.is_none() && type_parameters.is_none() {
            return Ok(DeclarationId::SmallVec::new());
        }

        let mut type_parameter_ids = DeclarationId::SmallVec::new();

        if let Some(parent_declaration_id) = parent_impl_or_trait {
            let parent_declaration = self.declarations.get_declaration(parent_declaration_id);

            match &parent_declaration.definition {
                Definition::Trait {
                    type_parameters,
                    declarations: Some(declarations),
                    ..
                } => {
                    for &member_id in self.scopes.get_members(*declarations) {
                        let member_declaration = self.declarations.get_declaration(member_id);

                        if matches!(
                            member_declaration.definition,
                            Definition::TypeParameter { is_self: true }
                        ) {
                            type_parameter_ids.push(member_id);

                            break;
                        }
                    }

                    if let Some(type_parameters) = type_parameters {
                        type_parameter_ids
                            .extend(self.scopes.get_members(*type_parameters).iter().copied());
                    }
                }
                Definition::TraitImplementation {
                    type_parameters, ..
                }
                | Definition::InherentImplementation {
                    type_parameters, ..
                } => {
                    if let Some(type_parameters) = type_parameters {
                        type_parameter_ids
                            .extend(self.scopes.get_members(*type_parameters).iter().copied());
                    }
                }
                _ => {}
            };
        }

        if let Some(type_parameters) = type_parameters {
            type_parameter_ids.extend(self.scopes.get_members(type_parameters).iter().copied());
        }

        Ok(type_parameter_ids)
    }

    pub fn find_method(
        &self,
        symbol_id: SymbolId,
        parent_declaration_id: DeclarationId,
    ) -> Option<DeclarationId> {
        let implementation_declaration_ids = self.implementations.get(&parent_declaration_id)?;
        let mut found_in_trait_implementation = None;

        for implementation_declaration_id in implementation_declaration_ids {
            let implementation_declaration = self
                .declarations
                .get_declaration(*implementation_declaration_id);

            match implementation_declaration.definition {
                Definition::InherentImplementation {
                    declarations: Some(scope_id),
                    ..
                } => {
                    if let Some(declaration_id) =
                        self.declarations.find_declaration_id(symbol_id, scope_id)
                    {
                        return Some(*declaration_id);
                    }
                }
                Definition::TraitImplementation {
                    trait_declaration_id,
                    declarations: Some(scope_id),
                    ..
                } => {
                    if let Some(declaration_id) =
                        self.declarations.find_declaration_id(symbol_id, scope_id)
                    {
                        return Some(*declaration_id);
                    }

                    if found_in_trait_implementation.is_none() {
                        let trait_declaration =
                            self.declarations.get_declaration(trait_declaration_id);
                        let Definition::Trait {
                            declarations: Some(trait_scope_id),
                            ..
                        } = trait_declaration.definition
                        else {
                            continue;
                        };

                        found_in_trait_implementation = self
                            .declarations
                            .find_declaration_id(symbol_id, trait_scope_id)
                            .copied();
                    }
                }
                _ => continue,
            };
        }

        found_in_trait_implementation
    }

    pub fn find_visible_declaration(
        &self,
        symbol_id: SymbolId,
        starting_scope_id: ScopeId,
        path_segment: SyntaxReader,
    ) -> Result<Option<DeclarationId>, CompileError> {
        let mut scope_id = Some(starting_scope_id);
        let mut crossed_scope_kinds = SmallVec::<[Barrier; 7]>::new();

        while let Some(current_scope_id) = scope_id {
            if let Some(declaration_id) = self
                .declarations
                .find_declaration_id(symbol_id, current_scope_id)
                .copied()
            {
                let declaration = self.declarations.get_declaration(declaration_id);

                if crossed_scope_kinds
                    .iter()
                    .any(|scope_kind| scope_kind.is_barrier(&declaration.definition))
                {
                    return Err(CompileError::DeclarationOutOfScope {
                        declaration_id,
                        usage_position: path_segment.position(),
                    });
                }

                return Ok(Some(declaration_id));
            }

            let scope = self.scopes.get_scope(current_scope_id);
            scope_id = scope.parent;

            if !crossed_scope_kinds.contains(&scope.barrier) {
                crossed_scope_kinds.push(scope.barrier);
            }
        }

        if let Some(declaration_id) = self
            .declarations
            .find_declaration_id(symbol_id, ScopeId::CORE)
        {
            return Ok(Some(*declaration_id));
        }

        Ok(None)
    }

    pub fn find_member_declaration(
        &self,
        symbol_id: SymbolId,
        parent_declaration_id: DeclarationId,
        path_segment: SyntaxReader,
    ) -> Result<DeclarationId, CompileError> {
        let parent_declaration = self.declarations.get_declaration(parent_declaration_id);
        let primary_member_scope_id = match parent_declaration.definition {
            Definition::Module {
                inner_scope_id: Some(inner_scope_id),
                ..
            }
            | Definition::StructType {
                fields: Some(inner_scope_id),
                ..
            }
            | Definition::EnumType {
                variants: Some(inner_scope_id),
                ..
            }
            | Definition::TraitImplementation {
                declarations: Some(inner_scope_id),
                ..
            }
            | Definition::InherentImplementation {
                declarations: Some(inner_scope_id),
                ..
            } => Some(inner_scope_id),
            Definition::StructType { fields: None, .. }
            | Definition::EnumType { variants: None, .. } => None,
            _ => {
                return Err(CompileError::Undeclared {
                    symbol_id,
                    usage_position: path_segment.position(),
                });
            }
        };

        if let Some(scope_id) = primary_member_scope_id
            && let Some(declaration_id) = self.declarations.find_declaration_id(symbol_id, scope_id)
        {
            return Ok(*declaration_id);
        }

        if matches!(
            parent_declaration.definition,
            Definition::StructType { .. } | Definition::EnumType { .. }
        ) && let Some(implementation_declaration_ids) =
            self.implementations.get(&parent_declaration_id)
        {
            let mut found_in_trait_implementation = None;

            for implementation_declaration_id in implementation_declaration_ids {
                let implementation_declaration = self
                    .declarations
                    .get_declaration(*implementation_declaration_id);

                match implementation_declaration.definition {
                    Definition::InherentImplementation {
                        declarations: Some(scope_id),
                        ..
                    } => {
                        if let Some(declaration_id) =
                            self.declarations.find_declaration_id(symbol_id, scope_id)
                        {
                            return Ok(*declaration_id);
                        }
                    }
                    Definition::TraitImplementation {
                        trait_declaration_id,
                        declarations: Some(scope_id),
                        ..
                    } => {
                        if let Some(declaration_id) =
                            self.declarations.find_declaration_id(symbol_id, scope_id)
                        {
                            return Ok(*declaration_id);
                        }

                        if found_in_trait_implementation.is_none() {
                            let trait_declaration =
                                self.declarations.get_declaration(trait_declaration_id);
                            let Definition::Trait {
                                declarations: Some(trait_scope_id),
                                ..
                            } = trait_declaration.definition
                            else {
                                continue;
                            };

                            found_in_trait_implementation = self
                                .declarations
                                .find_declaration_id(symbol_id, trait_scope_id)
                                .copied();
                        }
                    }
                    _ => continue,
                };
            }

            if let Some(declaration_id) = found_in_trait_implementation {
                return Ok(declaration_id);
            }
        }

        Err(CompileError::Undeclared {
            symbol_id,
            usage_position: path_segment.position(),
        })
    }

    pub fn get_defined_declaration(
        &mut self,
        declaration_id: DeclarationId,
    ) -> Result<Declaration, CompileError> {
        let mut current_id = declaration_id;

        loop {
            let current_declaration = self.declarations.get_declaration(current_id);

            match current_declaration.definition {
                Definition::ForwardReference { resolved } => {
                    if let Some(resolved_id) = resolved {
                        current_id = resolved_id;
                    } else {
                        return Err(CompileError::Undeclared {
                            symbol_id: current_declaration.symbol_id,
                            usage_position: current_declaration.syntax.unwrap().0,
                        });
                    }
                }
                _ => return Ok(*current_declaration),
            }
        }
    }

    pub fn infer_concrete_type_id(&mut self, type_id: TypeId) -> Result<TypeId, CompileError> {
        let (type_id, r#type) = self.get_concrete_type(type_id)?;

        if let Type::Inferred {
            constraint: Some(constraint),
            resolved: None,
            ..
        } = r#type
        {
            let resolved_type_id = match constraint {
                InferredTypeConstraint::Integer => TypeId::I_32,
                InferredTypeConstraint::Float => TypeId::F_64,
            };

            self.types.resolve_type(type_id, resolved_type_id)?;

            Ok(resolved_type_id)
        } else {
            Ok(type_id)
        }
    }

    pub fn get_concrete_type(&self, type_id: TypeId) -> Result<(TypeId, &Type), CompileError> {
        let mut current_id = type_id;

        loop {
            let current_type = self.types.get_type(current_id);

            match current_type {
                Type::Inferred {
                    resolved: Some(resolved_id),
                    ..
                } => {
                    current_id = *resolved_id;
                }
                Type::Generic { declaration_id } => {
                    if let Some(argument_type_id) = self.type_parameter_map.get(declaration_id) {
                        current_id = *argument_type_id;
                    } else {
                        return Err(CompileError::ExpectedConcreteType);
                    }
                }
                _ => return Ok((current_id, current_type)),
            }
        }
    }

    pub fn get_operand_types(
        &self,
        type_id: TypeId,
    ) -> Result<OperandType::SmallVec, CompileError> {
        let r#type = self.types.get_type(type_id);

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
            Type::Tuple { element_types } => {
                let element_type_ids = self.types.get_type_members(*element_types);
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
            Type::Pointer { .. } => Ok(smallvec![OperandType::POINTER]),
            Type::FunctionDefinition { .. } | Type::Closure { .. } | Type::Function { .. } => {
                Ok(smallvec![OperandType::FUNCTION])
            }
            Type::Algebraic {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id);

                match &declaration.definition {
                    Definition::EnumType {
                        variants,
                        type_parameters,
                        ..
                    } => {
                        let type_param_entries = (*type_parameters)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();
                        let type_parameter_argument_pairs = type_param_entries
                            .iter()
                            .zip(type_arguments.as_usize_range())
                            .filter_map(|(parameter_declaration_id, argument_index)| {
                                let argument_type_id =
                                    self.types.get_type_member(argument_index).ok()?;

                                Some((parameter_declaration_id, *argument_type_id))
                            });

                        let variant_entries = (*variants)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();

                        let mut largest_variant_operand_types: Vec<OperandType> = Vec::new();
                        let mut largest_variant_register_count: u16 = 0;

                        for &variant_declaration_id in variant_entries {
                            let variant_declaration =
                                self.declarations.get_declaration(variant_declaration_id);
                            let Definition::Variant { fields, .. } =
                                &variant_declaration.definition
                            else {
                                return Err(CompileError::ExpectedVariantDefinition(
                                    variant_declaration_id,
                                ));
                            };

                            let field_entries = (*fields)
                                .map(|id| self.scopes.get_members(id))
                                .unwrap_or_default();

                            let mut variant_operand_types: Vec<OperandType> = Vec::new();
                            let mut variant_register_count: u16 = 0;

                            for &field_declaration_id in field_entries {
                                let field_declaration =
                                    self.declarations.get_declaration(field_declaration_id);
                                let Definition::Field {
                                    type_id: field_type_id,
                                    ..
                                } = field_declaration.definition
                                else {
                                    return Err(CompileError::ExpectedFieldDefinition(
                                        field_declaration_id,
                                    ));
                                };

                                let resolved_field_type = *self.types.get_type(field_type_id);
                                let type_id = if let Type::Generic {
                                    declaration_id: parameter_declaration_id,
                                } = resolved_field_type
                                {
                                    type_parameter_argument_pairs
                                        .clone()
                                        .find_map(|(declaration_id, type_id)| {
                                            if *declaration_id == parameter_declaration_id {
                                                Some(type_id)
                                            } else {
                                                None
                                            }
                                        })
                                        .ok_or(CompileError::MissingTypeArgument(
                                            field_declaration_id,
                                        ))?
                                } else {
                                    field_type_id
                                };

                                let resolved_type_id = match self.types.get_type(type_id) {
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
                        let type_paramter_declaration_ids =
                            if let Some(type_parameters) = type_parameters {
                                self.scopes.get_members(*type_parameters)
                            } else {
                                &[]
                            };
                        let mut operand_types =
                            SmallVec::with_capacity(type_paramter_declaration_ids.len());

                        let type_parameter_argument_pairs = type_paramter_declaration_ids
                            .iter()
                            .zip(type_arguments.as_usize_range())
                            .filter_map(|(parameter_declaration_id, argument_index)| {
                                let argument_type_id =
                                    self.types.get_type_member(argument_index).ok()?;

                                Some((parameter_declaration_id, *argument_type_id))
                            });

                        let field_entries = (*fields)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();

                        for &field_declaration_id in field_entries {
                            let field_declaration =
                                self.declarations.get_declaration(field_declaration_id);
                            let Definition::Field {
                                type_id: field_type_id,
                                ..
                            } = field_declaration.definition
                            else {
                                continue;
                            };

                            let resolved_field_type = *self.types.get_type(field_type_id);
                            let type_id = if let Type::Generic {
                                declaration_id: parameter_declaration_id,
                            } = resolved_field_type
                            {
                                type_parameter_argument_pairs
                                    .clone()
                                    .find_map(|(declaration_id, type_id)| {
                                        if *declaration_id == parameter_declaration_id {
                                            Some(type_id)
                                        } else {
                                            None
                                        }
                                    })
                                    .ok_or(CompileError::MissingTypeArgument(
                                        field_declaration_id,
                                    ))?
                            } else {
                                field_type_id
                            };

                            let resolved_type_id = match self.types.get_type(type_id) {
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
                let type_scope_id = self.scopes.enter_scope(Barrier::Members, Some(scope_id));
                let element_type_ids = element_types
                    .iter()
                    .map(|element_type| self.add_external_type(element_type, type_scope_id))
                    .collect::<TypeId::SmallVec>();
                let element_types = self.types.add_type_members(element_type_ids);

                self.types.add_type(Type::Tuple { element_types })
            }
            DustType::Array(element_type, length) => {
                let element_type_id = self.add_external_type(element_type, scope_id);

                self.types.add_type(Type::Array {
                    element_type_id,
                    length: *length,
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
                let struct_scope_id = self.scopes.enter_scope(Barrier::Members, None);
                let struct_declaration_id = self.declarations.reserve_declaration_id(
                    struct_symbol_id,
                    struct_scope_id,
                    None,
                );

                let mut type_bindings = Vec::new();

                match value_type {
                    DustStructTypeFields::Unit => {}
                    DustStructTypeFields::Tuple(types) => {
                        for (index, field_type) in types.iter().enumerate() {
                            let field_type_id = self.add_external_type(field_type, struct_scope_id);
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
                        }
                    }
                    DustStructTypeFields::Named(fields) => {
                        for (field_name, field_type) in fields.iter() {
                            let field_type_id = self.add_external_type(field_type, struct_scope_id);
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
                        }
                    }
                };

                self.scopes.exit_scope(struct_scope_id);
                self.declarations.set_reserved_declaration(
                    struct_declaration_id,
                    Definition::StructType {
                        public: true,
                        type_parameters: None,
                        fields: Some(struct_scope_id),
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
                let enum_scope_id = self.scopes.enter_scope(Barrier::Members, None);
                let enum_declaration_id =
                    self.declarations
                        .reserve_declaration_id(enum_symbol_id, enum_scope_id, None);
                let mut enum_type_bindings = Vec::new();

                for (discriminant, (variant_name, variant_value_type)) in
                    variants.iter().enumerate()
                {
                    let variant_symbol_id = self.symbols.add_symbol(variant_name);
                    let (fields, kind) = match variant_value_type {
                        DustStructTypeFields::Unit => (None, VariantKind::Unit),
                        DustStructTypeFields::Tuple(types) => {
                            let enum_variant_scope_id = self
                                .scopes
                                .enter_scope(Barrier::Members, Some(enum_scope_id));

                            let mut type_bindings = Vec::new();

                            for (index, field_type) in types.iter().enumerate() {
                                let field_symbol_id = self.symbols.add_index_symbol(index as u32);
                                let field_type_id =
                                    self.add_external_type(field_type, enum_variant_scope_id);
                                let field_declaration_id =
                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: enum_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: enum_variant_scope_id,
                                        syntax: None,
                                    });

                                type_bindings.push((field_symbol_id, field_declaration_id));
                            }

                            self.scopes.exit_scope(enum_variant_scope_id);

                            (Some(enum_variant_scope_id), VariantKind::TupleFields)
                        }
                        DustStructTypeFields::Named(fields) => {
                            let enum_variant_scope_id = self
                                .scopes
                                .enter_scope(Barrier::Members, Some(enum_scope_id));

                            let mut type_bindings = Vec::new();

                            for (field_name, field_type) in fields.iter() {
                                let field_symbol_id = self.symbols.add_symbol(field_name);
                                let field_type_id =
                                    self.add_external_type(field_type, enum_variant_scope_id);
                                let field_declaration_id =
                                    self.declarations.add_declaration(Declaration {
                                        symbol_id: field_symbol_id,
                                        definition: Definition::Field {
                                            public: true,
                                            parent_struct: enum_declaration_id,
                                            type_id: field_type_id,
                                        },
                                        scope_id: enum_variant_scope_id,
                                        syntax: None,
                                    });

                                type_bindings.push((field_symbol_id, field_declaration_id));
                            }

                            self.scopes.exit_scope(enum_variant_scope_id);

                            (Some(enum_variant_scope_id), VariantKind::NamedFields)
                        }
                    };

                    let variant_declaration_id = self.declarations.add_declaration(Declaration {
                        symbol_id: variant_symbol_id,
                        definition: Definition::Variant {
                            discriminant: discriminant as u16,
                            enum_declaration_id,
                            fields,
                            kind,
                        },
                        scope_id: enum_scope_id,
                        syntax: None,
                    });

                    enum_type_bindings.push((variant_symbol_id, variant_declaration_id));
                }

                self.scopes.exit_scope(enum_scope_id);

                self.declarations.set_reserved_declaration(
                    enum_declaration_id,
                    Definition::EnumType {
                        public: true,
                        type_parameters: None,
                        variants: Some(enum_scope_id),
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
        let r#type = self.types.get_type(id);

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
            Type::Tuple { element_types } => {
                if element_types.is_empty() {
                    Ok(DustType::Unit)
                } else {
                    let element_types: Vec<DustType> = self
                        .get_type_members_as_full_types(*element_types)
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
                let declaration = self.declarations.get_declaration(*declaration_id);

                match &declaration.definition {
                    Definition::EnumType {
                        variants,
                        type_parameters,
                        ..
                    } => {
                        let enum_name =
                            self.symbols.get_symbol(&declaration.symbol_id)?.to_string();

                        let type_param_entries = (*type_parameters)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();
                        let type_parameter_map: SmallVec<[(DeclarationId, TypeId); 4]> =
                            type_param_entries
                                .iter()
                                .zip(type_arguments.as_usize_range())
                                .filter_map(|(parameter_declaration_id, argument_index)| {
                                    let argument_type_id =
                                        self.types.get_type_member(argument_index).ok()?;
                                    Some((*parameter_declaration_id, *argument_type_id))
                                })
                                .collect();

                        let variant_entries = (*variants)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();
                        let mut variants = Vec::with_capacity(variant_entries.len());

                        for variant_declaration_id in variant_entries {
                            let variant_declaration =
                                self.declarations.get_declaration(*variant_declaration_id);
                            let Definition::Variant { fields, .. } =
                                &variant_declaration.definition
                            else {
                                continue;
                            };

                            let variant_name = self
                                .symbols
                                .get_symbol(&variant_declaration.symbol_id)?
                                .to_string();

                            let field_entries = (*fields)
                                .map(|id| self.scopes.get_members(id))
                                .unwrap_or_default();
                            let mut field_types = Vec::new();

                            for (field_index, field_declaration_id) in
                                field_entries.iter().enumerate()
                            {
                                let field_declaration =
                                    self.declarations.get_declaration(*field_declaration_id);
                                let Definition::Field {
                                    type_id: field_type_id,
                                    ..
                                } = field_declaration.definition
                                else {
                                    continue;
                                };

                                let resolved_type = self.types.get_type(field_type_id);
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

                                let resolved_type_id = match self.types.get_type(concrete_type_id) {
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

                        let field_entries = (*fields)
                            .map(|id| self.scopes.get_members(id))
                            .unwrap_or_default();
                        let mut field_types = Vec::new();

                        for field_declaration_id in field_entries {
                            let field_declaration =
                                self.declarations.get_declaration(*field_declaration_id);
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
            Type::FunctionDefinition {
                declaration_id,
                type_arguments: _,
            } => {
                let declaration = self.declarations.get_declaration(*declaration_id);

                match &declaration.definition {
                    Definition::Function {
                        value_parameters,
                        return_type_id,
                        type_parameters: _,
                        ..
                    } => {
                        let value_parameters = if let Some(value_parameters) = *value_parameters {
                            self.scopes
                                .get_members(value_parameters)
                                .iter()
                                .map(|&parameter_declaration_id| {
                                    let parameter_declaration =
                                        self.declarations.get_declaration(parameter_declaration_id);
                                    let type_id = if let Definition::Local { type_id, .. } =
                                        parameter_declaration.definition
                                    {
                                        type_id
                                    } else {
                                        return Err(CompileError::ExpectedLocalDefinition(
                                            parameter_declaration_id,
                                        ));
                                    };

                                    self.get_external_type(type_id)
                                })
                                .try_collect::<Vec<DustType>>()?
                        } else {
                            Vec::new()
                        };

                        let return_dust_type = self.get_external_type(*return_type_id)?;

                        Ok(DustType::Function(Box::new(DustFunctionType {
                            value_parameters,
                            return_type: return_dust_type,
                        })))
                    }
                    Definition::NativeFunction {
                        value_parameters: _,
                        return_type_id: _,
                        type_parameters: _,
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

    fn get_type_members_as_full_types<'a>(
        &'a self,
        type_members: TypeMembers,
    ) -> impl Iterator<Item = Result<DustType, CompileError>> + 'a {
        type_members.as_usize_range().map(move |index| {
            let type_id = *self.types.get_type_member(index)?;

            self.get_external_type(type_id)
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

    pub fn index(self) -> u16 {
        self.0
    }
}

impl Display for PrototypeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "proto_{}", self.0)
    }
}

fn add_core(resolver: &mut Resolver) {
    const OPTION_VARIANTS: &[(&str, BuiltInStructFields)] = &[
        ("None", BuiltInStructFields::Unit),
        (
            "Some",
            BuiltInStructFields::Tuple(&[BuiltInType::Generic("T")]),
        ),
    ];
    const RESULT_VARIANTS: &[(&str, BuiltInStructFields)] = &[
        (
            "Ok",
            BuiltInStructFields::Tuple(&[BuiltInType::Generic("T")]),
        ),
        (
            "Err",
            BuiltInStructFields::Tuple(&[BuiltInType::Generic("E")]),
        ),
    ];
    const RANGE_FIELDS: &[(&str, BuiltInType)] = &[
        ("start", BuiltInType::Generic("T")),
        ("end", BuiltInType::Generic("T")),
    ];
    const BUILT_IN_TYPES: &[BuiltInType] = &[
        BuiltInType::Enum {
            name: "Option",
            type_parameters: &["T"],
            variants: OPTION_VARIANTS,
        },
        BuiltInType::Enum {
            name: "Result",
            type_parameters: &["T", "E"],
            variants: RESULT_VARIANTS,
        },
        BuiltInType::Struct {
            name: "Range",
            type_parameters: &["T"],
            fields: BuiltInStructFields::Named(RANGE_FIELDS),
        },
        BuiltInType::Struct {
            name: "RangeInclusive",
            type_parameters: &["T"],
            fields: BuiltInStructFields::Named(RANGE_FIELDS),
        },
    ];

    let core_scope_id = resolver.scopes.enter_scope(Barrier::Module, None);

    debug_assert_eq!(core_scope_id, ScopeId::CORE);

    let mut type_declaration_ids = Vec::with_capacity(BUILT_IN_TYPES.len());

    for built_in_type in BUILT_IN_TYPES {
        let type_symbol_id = resolver.symbols.add_symbol(built_in_type.name());
        let type_declaration_id =
            resolver
                .declarations
                .reserve_declaration_id(type_symbol_id, core_scope_id, None);

        resolver
            .scopes
            .add_to_current_namespace(type_declaration_id);
        type_declaration_ids.push((*built_in_type, type_declaration_id));
    }

    debug_assert_eq!(type_declaration_ids[0].1, DeclarationId::OPTION);
    debug_assert_eq!(type_declaration_ids[1].1, DeclarationId::RESULT);
    debug_assert_eq!(type_declaration_ids[2].1, DeclarationId::RANGE);
    debug_assert_eq!(type_declaration_ids[3].1, DeclarationId::RANGE_INCLUSIVE);

    for (built_in_type, type_declaration_id) in type_declaration_ids {
        add_built_in_type_definition(resolver, built_in_type, type_declaration_id, core_scope_id);
    }

    resolver.scopes.exit_scope(core_scope_id);
    resolver.declarations.finish_reserved_range();
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
    Generic(&'a str),
}

#[derive(Clone, Copy)]
enum BuiltInStructFields<'a> {
    Unit,
    Tuple(&'a [BuiltInType<'a>]),
    Named(&'a [(&'a str, BuiltInType<'a>)]),
}

impl<'a> BuiltInType<'a> {
    fn name(self) -> &'a str {
        match self {
            BuiltInType::Struct { name, .. } | BuiltInType::Enum { name, .. } => name,
            BuiltInType::Generic(_) => unreachable!("generic built-in types are not top-level"),
        }
    }
}

fn add_built_in_type_definition(
    resolver: &mut Resolver,
    built_in_type: BuiltInType,
    type_declaration_id: DeclarationId,
    core_scope_id: ScopeId,
) {
    let item_scope_id = resolver
        .scopes
        .enter_scope(Barrier::Item, Some(core_scope_id));

    match built_in_type {
        BuiltInType::Struct {
            type_parameters,
            fields,
            ..
        } => {
            let type_parameter_declarations =
                add_built_in_type_parameters(resolver, item_scope_id, type_parameters);
            let type_parameters = if type_parameter_declarations.is_empty() {
                None
            } else {
                Some(item_scope_id)
            };
            let fields = add_built_in_fields(
                resolver,
                fields,
                type_declaration_id,
                item_scope_id,
                &type_parameter_declarations,
            );

            resolver.scopes.exit_scope(item_scope_id);
            resolver.declarations.set_reserved_declaration(
                type_declaration_id,
                Definition::StructType {
                    public: true,
                    type_parameters,
                    fields,
                },
            );
        }
        BuiltInType::Enum {
            type_parameters,
            variants,
            ..
        } => {
            let type_parameter_declarations =
                add_built_in_type_parameters(resolver, item_scope_id, type_parameters);
            let type_parameters = if type_parameter_declarations.is_empty() {
                None
            } else {
                Some(item_scope_id)
            };
            let variants = add_built_in_variants(
                resolver,
                variants,
                type_declaration_id,
                item_scope_id,
                &type_parameter_declarations,
            );

            resolver.scopes.exit_scope(item_scope_id);
            resolver.declarations.set_reserved_declaration(
                type_declaration_id,
                Definition::EnumType {
                    public: true,
                    type_parameters,
                    variants: Some(variants),
                },
            );
        }
        BuiltInType::Generic(_) => unreachable!("generic built-in types are not declarations"),
    }
}

fn add_built_in_type_parameters<'a>(
    resolver: &mut Resolver,
    item_scope_id: ScopeId,
    type_parameters: &'a [&'a str],
) -> Vec<(&'a str, SymbolId, DeclarationId)> {
    let mut type_parameter_declarations = Vec::with_capacity(type_parameters.len());

    for type_parameter_name in type_parameters {
        let type_parameter_symbol_id = resolver.symbols.add_symbol(type_parameter_name);
        let type_parameter_declaration_id = resolver.declarations.add_declaration(Declaration {
            symbol_id: type_parameter_symbol_id,
            definition: Definition::TypeParameter { is_self: false },
            scope_id: item_scope_id,
            syntax: None,
        });

        type_parameter_declarations.push((
            *type_parameter_name,
            type_parameter_symbol_id,
            type_parameter_declaration_id,
        ));
        resolver
            .scopes
            .add_to_current_namespace(type_parameter_declaration_id);
    }

    type_parameter_declarations
}

fn add_built_in_fields(
    resolver: &mut Resolver,
    fields: BuiltInStructFields,
    parent_declaration_id: DeclarationId,
    parent_scope_id: ScopeId,
    type_parameter_declarations: &[(&str, SymbolId, DeclarationId)],
) -> Option<ScopeId> {
    match fields {
        BuiltInStructFields::Unit => None,
        BuiltInStructFields::Tuple(field_types) => {
            let fields_scope_id = resolver
                .scopes
                .enter_scope(Barrier::Members, Some(parent_scope_id));

            for (field_index, field_type) in field_types.iter().enumerate() {
                let field_symbol_id = resolver.symbols.add_index_symbol(field_index as u32);
                let field_type_id =
                    get_built_in_type_id(resolver, *field_type, type_parameter_declarations);
                let field_declaration_id = resolver.declarations.add_declaration(Declaration {
                    symbol_id: field_symbol_id,
                    definition: Definition::Field {
                        public: true,
                        parent_struct: parent_declaration_id,
                        type_id: field_type_id,
                    },
                    scope_id: fields_scope_id,
                    syntax: None,
                });

                resolver
                    .scopes
                    .add_to_current_namespace(field_declaration_id);
            }

            resolver.scopes.exit_scope(fields_scope_id);

            Some(fields_scope_id)
        }
        BuiltInStructFields::Named(field_types) => {
            let fields_scope_id = resolver
                .scopes
                .enter_scope(Barrier::Members, Some(parent_scope_id));

            for (field_name, field_type) in field_types {
                let field_symbol_id = resolver.symbols.add_symbol(field_name);
                let field_type_id =
                    get_built_in_type_id(resolver, *field_type, type_parameter_declarations);
                let field_declaration_id = resolver.declarations.add_declaration(Declaration {
                    symbol_id: field_symbol_id,
                    definition: Definition::Field {
                        public: true,
                        parent_struct: parent_declaration_id,
                        type_id: field_type_id,
                    },
                    scope_id: fields_scope_id,
                    syntax: None,
                });

                resolver
                    .scopes
                    .add_to_current_namespace(field_declaration_id);
            }

            resolver.scopes.exit_scope(fields_scope_id);

            Some(fields_scope_id)
        }
    }
}

fn add_built_in_variants(
    resolver: &mut Resolver,
    variants: &[(&str, BuiltInStructFields)],
    enum_declaration_id: DeclarationId,
    item_scope_id: ScopeId,
    type_parameter_declarations: &[(&str, SymbolId, DeclarationId)],
) -> ScopeId {
    let variants_scope_id = resolver
        .scopes
        .enter_scope(Barrier::Members, Some(item_scope_id));
    let mut variant_namespace_entries = Vec::with_capacity(variants.len());

    for (variant_index, (variant_name, variant_fields)) in variants.iter().enumerate() {
        let variant_symbol_id = resolver.symbols.add_symbol(variant_name);
        let variant_declaration_id = match variant_fields {
            BuiltInStructFields::Unit => resolver.declarations.add_declaration(Declaration {
                symbol_id: variant_symbol_id,
                definition: Definition::Variant {
                    discriminant: variant_index as u16,
                    enum_declaration_id,
                    fields: None,
                    kind: VariantKind::Unit,
                },
                scope_id: variants_scope_id,
                syntax: None,
            }),
            BuiltInStructFields::Tuple(_) => {
                let variant_declaration_id = resolver.declarations.reserve_declaration_id(
                    variant_symbol_id,
                    variants_scope_id,
                    None,
                );
                let fields = add_built_in_fields(
                    resolver,
                    *variant_fields,
                    variant_declaration_id,
                    variants_scope_id,
                    type_parameter_declarations,
                );

                resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant: variant_index as u16,
                        enum_declaration_id,
                        fields,
                        kind: VariantKind::TupleFields,
                    },
                );

                variant_declaration_id
            }
            BuiltInStructFields::Named(_) => {
                let variant_declaration_id = resolver.declarations.reserve_declaration_id(
                    variant_symbol_id,
                    variants_scope_id,
                    None,
                );
                let fields = add_built_in_fields(
                    resolver,
                    *variant_fields,
                    variant_declaration_id,
                    variants_scope_id,
                    type_parameter_declarations,
                );

                resolver.declarations.set_reserved_declaration(
                    variant_declaration_id,
                    Definition::Variant {
                        discriminant: variant_index as u16,
                        enum_declaration_id,
                        fields,
                        kind: VariantKind::NamedFields,
                    },
                );

                variant_declaration_id
            }
        };

        resolver
            .scopes
            .add_to_current_namespace(variant_declaration_id);
        variant_namespace_entries.push((variant_symbol_id, variant_declaration_id));
    }

    resolver.scopes.exit_scope(variants_scope_id);

    variants_scope_id
}

fn get_built_in_type_id(
    resolver: &mut Resolver,
    built_in_type: BuiltInType,
    type_parameter_declarations: &[(&str, SymbolId, DeclarationId)],
) -> TypeId {
    match built_in_type {
        BuiltInType::Generic(type_parameter_name) => {
            for (candidate_name, _, declaration_id) in type_parameter_declarations {
                if *candidate_name == type_parameter_name {
                    return resolver.types.add_type(Type::Generic {
                        declaration_id: *declaration_id,
                    });
                }
            }

            unreachable!("missing core type parameter")
        }
        BuiltInType::Struct { .. } | BuiltInType::Enum { .. } => {
            unreachable!("nested core algebraic types are not used")
        }
    }
}
