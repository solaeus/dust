pub mod declaration_graph;
pub mod scope_graph;
pub mod symbol_table;
pub mod type_graph;

use std::collections::{HashMap, HashSet};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    dust_error::{ErrorKind, InternalError},
    dust_type::{DustFunctionType, DustStructType, DustType},
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
    small_type::SmallType,
    source::Source,
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
        let mut resolver = Self {
            symbols: SymbolTable::new(),
            declarations: DeclarationGraph::new(),
            scopes: ScopeGraph::new(),
            types: TypeGraph::new(),
            scope_search: HashSet::default(),
            declaration_bindings: HashMap::default(),
            scope_bindings: HashMap::default(),
            type_bindings: HashMap::default(),
        };

        resolver.add_core();

        resolver
    }

    fn add_core(&mut self) {
        let mut core_items =
            SmallVec::<[DeclarationId; 4]>::with_capacity(NativeFunction::ALL.len());

        let option_declaration_id = self.declarations.next_declaration_id();
        let none_declaration_id = option_declaration_id.offset(1);
        let some_field_declaration_id = option_declaration_id.offset(2);
        let some_declaration_id = option_declaration_id.offset(3);

        let option_symbol = self.symbols.add_symbol("Option");
        let none_symbol = self.symbols.add_symbol("None");
        let some_field_symbol = self.symbols.add_symbol("0");
        let some_symbol = self.symbols.add_symbol("Some");

        let _option_declaration_id = {
            let members = self
                .declarations
                .add_declaration_members(&[none_declaration_id, some_declaration_id]);

            self.declarations.add_declaration(Declaration {
                symbol_id: option_symbol,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: None,
                    type_parameters: DeclarationMembers::default(),
                    members,
                },
                scope_id: ScopeId::CORE,
                is_public: true,
            })
        };
        let _none_declaration_id = self.declarations.add_declaration(Declaration {
            symbol_id: none_symbol,
            syntax: None,
            kind: DeclarationKind::Type {
                parent: Some(option_declaration_id),
                type_parameters: DeclarationMembers::default(),
                members: DeclarationMembers::default(),
            },
            scope_id: ScopeId::CORE,
            is_public: true,
        });
        let _some_field_declaration_id = self.declarations.add_declaration(Declaration {
            symbol_id: some_field_symbol,
            syntax: None,
            kind: DeclarationKind::Type {
                parent: Some(some_declaration_id),
                type_parameters: DeclarationMembers::default(),
                members: DeclarationMembers::default(),
            },
            scope_id: ScopeId::CORE,
            is_public: true,
        });
        let _some_declaration_id = {
            let members = self
                .declarations
                .add_declaration_members(&[some_field_declaration_id]);

            self.declarations.add_declaration(Declaration {
                symbol_id: some_symbol,
                syntax: None,
                kind: DeclarationKind::Type {
                    parent: Some(option_declaration_id),
                    type_parameters: DeclarationMembers::default(),
                    members,
                },
                scope_id: ScopeId::CORE,
                is_public: true,
            })
        };

        debug_assert_eq!(option_declaration_id, _option_declaration_id);
        debug_assert_eq!(none_declaration_id, _none_declaration_id);
        debug_assert_eq!(some_field_declaration_id, _some_field_declaration_id);
        debug_assert_eq!(some_declaration_id, _some_declaration_id);

        core_items.push(option_declaration_id);
        core_items.push(none_declaration_id);
        core_items.push(some_declaration_id);

        for native_function in NativeFunction::ALL {
            let function_symbol = self.symbols.add_symbol(native_function.name());
            let declaration_id = self.declarations.add_declaration(Declaration {
                symbol_id: function_symbol,
                syntax: None,
                kind: DeclarationKind::NativeFunction(native_function),
                scope_id: ScopeId::CORE,
                is_public: true,
            });
            let type_id = native_function.signature(&mut self.types);

            self.declarations
                .set_declaration_type(declaration_id, type_id);
            core_items.push(declaration_id);
        }

        let core_scope_id = self.scopes.add_scope(Scope {
            kind: ScopeKind::Module,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: core_items,
        });

        debug_assert_eq!(core_scope_id, ScopeId::CORE);

        let core_symbol_id = self.symbols.add_symbol("core");

        self.declarations.add_declaration(Declaration {
            symbol_id: core_symbol_id,
            syntax: None,
            kind: DeclarationKind::Module {
                kind: ModuleKind::Inline,
                inner_scope_id: core_scope_id,
            },
            scope_id: ScopeId::NONE,
            is_public: true,
        });
    }

    pub fn add_declaration_binding(&mut self, syntax_id: SyntaxId, declaration_id: DeclarationId) {
        self.declaration_bindings.insert(syntax_id, declaration_id);
    }

    pub fn get_declaration_binding(
        &self,
        syntax_id: &SyntaxId,
    ) -> Result<&DeclarationId, ErrorKind> {
        self.declaration_bindings
            .get(syntax_id)
            .ok_or(ErrorKind::Internal(
                InternalError::MissingDeclarationBinding(*syntax_id),
            ))
    }

    pub fn add_scope_binding(&mut self, syntax_id: SyntaxId, scope_id: ScopeId) {
        self.scope_bindings.insert(syntax_id, scope_id);
    }

    pub fn get_scope_binding(&self, syntax_id: &SyntaxId) -> Result<&ScopeId, ErrorKind> {
        self.scope_bindings
            .get(syntax_id)
            .ok_or(ErrorKind::Internal(InternalError::MissingScopeBinding(
                *syntax_id,
            )))
    }

    pub fn add_type_binding(&mut self, syntax_id: SyntaxId, type_id: TypeId) {
        self.type_bindings.insert(syntax_id, type_id);
    }

    pub fn get_type_binding(&self, syntax_id: &SyntaxId) -> Result<&TypeId, ErrorKind> {
        self.type_bindings.get(syntax_id).ok_or(ErrorKind::Internal(
            InternalError::MissingTypeBinding(*syntax_id),
        ))
    }

    pub fn find_declaration_in_scope(
        &mut self,
        symbol_id: SymbolId,
        target_scope_id: ScopeId,
        path_segment: &SyntaxReader,
    ) -> Result<(DeclarationId, Declaration), ErrorKind> {
        let mut current_scope_id = target_scope_id;

        loop {
            if current_scope_id == ScopeId::NONE || !self.scope_search.insert(current_scope_id) {
                break;
            }

            if let Some((declaration_id, declaration)) = self
                .declarations
                .find_declaration(symbol_id, current_scope_id)
            {
                self.scope_search.clear();

                return Ok((declaration_id, declaration));
            }

            let current_scope = self.scopes.get_scope(current_scope_id)?;

            for module_scope_id in &current_scope.modules {
                if let Some((declaration_id, declaration)) = self
                    .declarations
                    .find_declaration(symbol_id, *module_scope_id)
                {
                    self.scope_search.clear();

                    return Ok((declaration_id, declaration));
                }
            }

            for import_declaration_id in &current_scope.imports {
                let import_declaration =
                    self.declarations.get_declaration(*import_declaration_id)?;

                if import_declaration.symbol_id == symbol_id {
                    self.scope_search.clear();

                    return Ok((*import_declaration_id, import_declaration));
                }
            }

            current_scope_id = current_scope.parent;
        }

        self.scope_search.clear();

        Err(ErrorKind::Compile(CompileError::Undeclared {
            symbol_id,
            usage_position: path_segment.position(),
        }))
    }

    pub fn infer_type(&self, type_id: TypeId) -> Result<TypeId, ErrorKind> {
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
        left_syntax: Option<SyntaxReader>,
        right: TypeId,
        right_syntax: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        let left_inferred = self.infer_type(left)?;
        let right_inferred = self.infer_type(right)?;

        self.unify_inferred_types(left_inferred, left_syntax, right_inferred, right_syntax)
    }

    pub fn unify_inferred_types<'a>(
        &'a mut self,
        left: TypeId,
        left_syntax: Option<SyntaxReader<'a>>,
        right: TypeId,
        right_syntax: SyntaxReader<'a>,
    ) -> Result<(), ErrorKind> {
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
                left_syntax,
                right_element_type,
                right_syntax,
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
                    self.unify_types(left_type_id, left_syntax, right_type_id, right_syntax)?;
                }

                self.unify_types(
                    left_return_type,
                    left_syntax,
                    right_return_type,
                    right_syntax,
                )?;

                Ok(())
            }
            (
                TypeNode::Struct {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                    ..
                },
                TypeNode::Struct {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                    ..
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children()?.next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()?
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    return Err(ErrorKind::Compile(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    }));
                }

                let left_args = self
                    .types
                    .get_type_members(left_type_arguments)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_args = self
                    .types
                    .get_type_members(right_type_arguments)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();

                for (left_arg, right_arg) in left_args.iter().zip(right_args.iter()) {
                    self.unify_types(*left_arg, left_syntax, *right_arg, right_syntax)?;
                }

                Ok(())
            }
            (left_type_node, right_type_node) => {
                if left_type_node == right_type_node {
                    Ok(())
                } else {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children()?.next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()?
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    Err(ErrorKind::Compile(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    }))
                }
            }
        }
    }

    pub fn add_external_type(&mut self, new_type: &DustType) -> TypeId {
        let node = match new_type {
            DustType::Unit => TypeNode::Unit,
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
                    let symbol = self.symbols.add_symbol(type_parameter_name);
                    let type_parameter_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type {
                            parent: None,
                            type_parameters: DeclarationMembers::default(),
                            members: DeclarationMembers::default(),
                        },
                        scope_id: ScopeId::NONE,
                        is_public: false,
                        syntax: None,
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

                let symbol = self.symbols.add_symbol(name);
                let struct_declaration_id = self.declarations.next_declaration_id();

                let mut field_declaration_ids =
                    SmallVec::<[DeclarationId; 8]>::with_capacity(fields.len());

                for (field_name, field_type) in fields {
                    let symbol = self.symbols.add_symbol(field_name);
                    let declaration_id = self.declarations.add_declaration(Declaration {
                        symbol_id: symbol,
                        kind: DeclarationKind::Type {
                            parent: Some(struct_declaration_id),
                            type_parameters: DeclarationMembers::default(),
                            members: DeclarationMembers::default(),
                        },
                        scope_id: ScopeId::NONE,
                        is_public: false,
                        syntax: None,
                    });
                    let type_id = self.add_external_type(field_type);

                    field_declaration_ids.push(declaration_id);
                    self.declarations
                        .set_declaration_type(declaration_id, type_id);
                }

                let members = self
                    .declarations
                    .add_declaration_members(&field_declaration_ids);
                let declared_id = self.declarations.add_declaration(Declaration {
                    symbol_id: symbol,
                    kind: DeclarationKind::Type {
                        parent: None,
                        type_parameters: DeclarationMembers::default(),
                        members,
                    },
                    scope_id: ScopeId::NONE,
                    is_public: false,
                    syntax: None,
                });

                debug_assert_eq!(declared_id, struct_declaration_id);

                TypeNode::Struct {
                    declaration_id: struct_declaration_id,
                    type_arguments: TypeMembers::default(),
                }
            }
        };

        self.types.add_type(node)
    }

    pub fn get_full_type(&self, id: TypeId, source: &Source) -> Result<DustType, ErrorKind> {
        let type_node = self.types.get_type(id)?;

        match type_node {
            TypeNode::Unit => Ok(DustType::Unit),
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
                    Err(ErrorKind::Compile(CompileError::CannotInferType {
                        type_id: id,
                        position: None,
                    }))
                }
            }
            TypeNode::Struct { declaration_id, .. } => {
                let struct_declaration = self.declarations.get_declaration(*declaration_id)?;
                let struct_name = self
                    .symbols
                    .get_symbol(&struct_declaration.symbol_id)?
                    .to_string();

                let DeclarationKind::Type { members, .. } = struct_declaration.kind else {
                    return Err(ErrorKind::Internal(InternalError::MissingDeclaration(
                        *declaration_id,
                    )));
                };
                let field_ids = self.declarations.get_declaration_members(members)?;
                let mut field_types = Vec::with_capacity(field_ids.len());

                for field_id in field_ids {
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
    ) -> impl Iterator<Item = Result<String, ErrorKind>> {
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
    ) -> impl Iterator<Item = Result<DustType, ErrorKind>> {
        members.as_range().map(|member_index| {
            let type_id = *self.types.get_type_member(member_index)?;

            self.get_full_type(type_id, source)
        })
    }

    pub fn get_small_type(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<SmallType, ErrorKind> {
        match self.types.get_type(type_id)? {
            TypeNode::Unit => Ok(SmallType::UNIT),
            TypeNode::Boolean => Ok(SmallType::BOOLEAN),
            TypeNode::Byte => Ok(SmallType::BYTE),
            TypeNode::Character => Ok(SmallType::CHARACTER),
            TypeNode::Float => Ok(SmallType::FLOAT),
            TypeNode::Integer => Ok(SmallType::INTEGER),
            TypeNode::String => Ok(SmallType::STRING),
            TypeNode::List { element_type } => match *element_type {
                TypeId::BOOLEAN => Ok(SmallType::LIST_BOOLEAN),
                TypeId::BYTE => Ok(SmallType::LIST_BYTE),
                TypeId::CHARACTER => Ok(SmallType::LIST_CHARACTER),
                TypeId::FLOAT => Ok(SmallType::LIST_FLOAT),
                TypeId::INTEGER => Ok(SmallType::LIST_INTEGER),
                TypeId::STRING => Ok(SmallType::LIST_STRING),
                _ => {
                    let element_operand_type = self.get_small_type(*element_type, node)?;

                    match element_operand_type {
                        SmallType::LIST_BOOLEAN
                        | SmallType::LIST_BYTE
                        | SmallType::LIST_CHARACTER
                        | SmallType::LIST_FLOAT
                        | SmallType::LIST_INTEGER
                        | SmallType::LIST_STRING
                        | SmallType::LIST_LIST
                        | SmallType::LIST_FUNCTION => Ok(SmallType::LIST_LIST),
                        _ => Err(ErrorKind::Compile(CompileError::CannotInferType {
                            type_id,
                            position: Some(node.position()),
                        })),
                    }
                }
            },
            TypeNode::Function { .. } => Ok(SmallType::FUNCTION),
            TypeNode::Struct { .. } => Ok(SmallType::STRUCT),
            TypeNode::Inferred {
                resolved: Some(inferred),
                ..
            } => self.get_small_type(*inferred, node),
            TypeNode::Inferred { resolved: None, .. } | TypeNode::Enum { .. } => {
                Err(ErrorKind::Compile(CompileError::CannotInferType {
                    type_id,
                    position: Some(node.position()),
                }))
            }
        }
    }

    pub fn get_register_size(
        &self,
        type_id: TypeId,
        node: &SyntaxReader,
    ) -> Result<u16, ErrorKind> {
        match self.types.get_type(type_id)? {
            TypeNode::Unit => Ok(0),
            TypeNode::Struct { declaration_id, .. } => {
                let struct_declaration = self.declarations.get_declaration(*declaration_id)?;
                let DeclarationKind::Type { members, .. } = struct_declaration.kind else {
                    return Err(ErrorKind::Internal(InternalError::MissingDeclaration(
                        *declaration_id,
                    )));
                };
                let mut leaf_count: u16 = 0;

                for index in members.start..(members.start + members.count) {
                    let field_declaration_id = self.declarations.get_declaration_member(index)?;
                    let field_type_id = *self
                        .declarations
                        .get_declaration_type(field_declaration_id)?;
                    let field_register_size = self.get_register_size(field_type_id, node)?;

                    let mut resolved_field_type_id = field_type_id;

                    while let TypeNode::Inferred {
                        resolved: Some(resolved),
                        ..
                    } = self.types.get_type(resolved_field_type_id)?
                    {
                        resolved_field_type_id = *resolved;
                    }

                    let field_leaf_count = match self.types.get_type(resolved_field_type_id)? {
                        TypeNode::Unit => 0,
                        TypeNode::Struct { .. } => field_register_size.saturating_sub(1),
                        _ => 1,
                    };

                    leaf_count = leaf_count.saturating_add(field_leaf_count);
                }

                if leaf_count > 0 {
                    leaf_count += 1;
                }

                Ok(leaf_count)
            }
            TypeNode::Inferred { resolved, .. } => match resolved {
                Some(resolved) => self.get_register_size(*resolved, node),
                None => Err(ErrorKind::Compile(CompileError::CannotInferType {
                    type_id,
                    position: Some(node.position()),
                })),
            },
            _ => Ok(1),
        }
    }

    pub fn declaration_display_iterator<'a>(
        &'a self,
    ) -> impl Iterator<Item = Result<String, ErrorKind>> + 'a {
        self.declarations.iter().map(|(id, declaration)| {
            let symbol = self.symbols.get_symbol(&declaration.symbol_id)?;
            let kind_str = match declaration.kind {
                DeclarationKind::Module { .. } => "module",
                DeclarationKind::Type { parent, .. } => {
                    if let Some(parent) = parent {
                        let parent_declaration = self.declarations.get_declaration(parent)?;
                        let parent_symbol =
                            self.symbols.get_symbol(&parent_declaration.symbol_id)?;

                        return Ok(format!(
                            "ID {}: {symbol} type (parent: {parent_symbol})",
                            id.inner()
                        ));
                    } else {
                        "type"
                    }
                }
                DeclarationKind::NativeFunction(_) => "native function",
                DeclarationKind::Function => "function",
                DeclarationKind::Local { shadowed } => {
                    if shadowed.is_some() {
                        "local (shadow)"
                    } else {
                        "local"
                    }
                }
            };

            Ok(format!("ID {}: {symbol} {kind_str}", id.inner()))
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
