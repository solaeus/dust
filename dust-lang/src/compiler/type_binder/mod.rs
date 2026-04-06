use smallvec::{SmallVec, smallvec};

use crate::{
    compiler::{error::CompileError, value_creation::create_usize_from_decimal},
    resolver::{
        Resolver,
        declarations::{Declaration, DeclarationId, Definition, Visibility},
        error::ResolverError,
        scopes::ScopeId,
        types::{InferredTypeConstraint, Type, TypeId, TypeMembers},
    },
    source::Source,
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, CallExpression,
            ComparisonExpression, CompoundAssignmentExpression, ConstItem, ExpressionStatement,
            FieldAccessExpression, FunctionType, GroupedExpression, IfExpression, ImplItem,
            ImplTraitItem, IndexExpression, LetStatement, LogicExpression, MathExpression,
            NegationExpression, NotExpression, PathSegment, RangeExpression, StructExpression,
            StructExpressionStructFields, TraitConst, TraitItem, WhileExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
        visitor::SyntaxVisitor,
    },
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    resolver: &'a mut Resolver,
    source: &'a Source<'a>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(resolver: &'a mut Resolver, source: &'a Source<'a>) -> Self {
        Self { resolver, source }
    }

    pub fn bind_function_body(
        &mut self,
        body: SyntaxReader,
        return_type_id: TypeId,
    ) -> Result<(), CompileError> {
        assert_eq!(body.node.kind, SyntaxKind::BlockExpression);

        let resolved_return_type_id = self.resolver.resolve_type(return_type_id)?;

        let mut body_type_id = TypeId::UNIT;

        for child in body.children() {
            if child.is_expression() {
                body_type_id = self.visit_expression(child, None)?;
            } else {
                self.visit_statement(child)?;

                body_type_id = TypeId::UNIT;
            }
        }

        self.unify_types(resolved_return_type_id, None, body_type_id, body)?;

        Ok(())
    }

    fn infer_type(&self, type_id: TypeId) -> Result<TypeId, CompileError> {
        if let Type::Inferred {
            resolved: Some(resolved),
            ..
        } = self.resolver.types.get_type(type_id)?
        {
            self.infer_type(*resolved)
        } else {
            Ok(type_id)
        }
    }

    fn unify_types(
        &mut self,
        left: TypeId,
        left_syntax: Option<SyntaxReader>,
        right: TypeId,
        right_syntax: SyntaxReader,
    ) -> Result<(), CompileError> {
        if left == right {
            return Ok(());
        }

        let left_inferred = self.infer_type(left)?;
        let right_inferred = self.infer_type(right)?;

        if left_inferred == right_inferred {
            return Ok(());
        }

        self.unify_inferred_types(left_inferred, left_syntax, right_inferred, right_syntax)
    }

    fn unify_inferred_types<'b>(
        &'b mut self,
        left: TypeId,
        left_syntax: Option<SyntaxReader<'b>>,
        right: TypeId,
        right_syntax: SyntaxReader<'b>,
    ) -> Result<(), CompileError> {
        let left_type_node = *self.resolver.types.get_type(left)?;
        let right_type_node = *self.resolver.types.get_type(right)?;

        match (left_type_node, right_type_node) {
            (
                Type::Inferred {
                    inferred_id: left_inferred_id,
                    constraint: left_constraint,
                    resolved: None,
                },
                Type::Inferred {
                    constraint: right_constraint,
                    resolved: None,
                    ..
                },
            ) => {
                match (left_constraint, right_constraint) {
                    (Some(left_bound), Some(right_bound)) if left_bound != right_bound => {
                        let expected_position = if let Some(left) = left_syntax {
                            left.children().next_back().map(|child| child.position())
                        } else {
                            None
                        };
                        let found_position = right_syntax
                            .children()
                            .next_back()
                            .unwrap_or(right_syntax)
                            .position();

                        return Err(CompileError::TypeConflict {
                            expected_type: left,
                            expected_position,
                            found_type: right,
                            found_position,
                        });
                    }
                    (Some(inherited_constraint), None) => {
                        let right_node = self.resolver.types.get_type_mut(right)?;

                        if let Type::Inferred { constraint, .. } = right_node {
                            *constraint = Some(inherited_constraint);
                        }
                    }
                    _ => {}
                }

                let left_node = self.resolver.types.get_type_mut(left)?;

                *left_node = Type::Inferred {
                    inferred_id: left_inferred_id,
                    constraint: left_constraint,
                    resolved: Some(right),
                };

                Ok(())
            }
            (
                Type::Inferred {
                    inferred_id,
                    constraint,
                    resolved: None,
                },
                _,
            ) => {
                if let Some(constraint) = constraint {
                    let satisfied = match constraint {
                        InferredTypeConstraint::Integer => matches!(
                            right_type_node,
                            Type::SignedInteger(_) | Type::UnsignedInteger(_)
                        ),
                        InferredTypeConstraint::Float => {
                            matches!(right_type_node, Type::Float(_))
                        }
                    };

                    if !satisfied {
                        let expected_position = if let Some(left) = left_syntax {
                            left.children().next_back().map(|child| child.position())
                        } else {
                            None
                        };
                        let found_position = right_syntax
                            .children()
                            .next_back()
                            .unwrap_or(right_syntax)
                            .position();

                        return Err(CompileError::TypeConflict {
                            expected_type: left,
                            expected_position,
                            found_type: right,
                            found_position,
                        });
                    }
                }

                let left_node = self.resolver.types.get_type_mut(left)?;

                *left_node = Type::Inferred {
                    inferred_id,
                    constraint,
                    resolved: Some(right),
                };

                Ok(())
            }
            (
                _,
                Type::Inferred {
                    inferred_id,
                    constraint,
                    resolved: None,
                },
            ) => {
                if let Some(constraint) = constraint {
                    let satisfied = match constraint {
                        InferredTypeConstraint::Integer => matches!(
                            left_type_node,
                            Type::SignedInteger(_) | Type::UnsignedInteger(_)
                        ),
                        InferredTypeConstraint::Float => {
                            matches!(left_type_node, Type::Float(_))
                        }
                    };

                    if !satisfied {
                        let expected_position = if let Some(left) = left_syntax {
                            left.children().next_back().map(|child| child.position())
                        } else {
                            None
                        };
                        let found_position = right_syntax
                            .children()
                            .next_back()
                            .unwrap_or(right_syntax)
                            .position();

                        return Err(CompileError::TypeConflict {
                            expected_type: left,
                            expected_position,
                            found_type: right,
                            found_position,
                        });
                    }
                }

                let right_node = self.resolver.types.get_type_mut(right)?;

                *right_node = Type::Inferred {
                    inferred_id,
                    constraint,
                    resolved: Some(left),
                };

                Ok(())
            }
            (
                Type::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    return Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    });
                }

                for (left_index, right_index) in left_type_arguments
                    .as_range()
                    .zip(right_type_arguments.as_range())
                {
                    let left_arg = *self.resolver.types.get_type_member(left_index)?;
                    let right_arg = *self.resolver.types.get_type_member(right_index)?;

                    self.unify_types(left_arg, left_syntax, right_arg, right_syntax)?;
                }

                Ok(())
            }
            (
                Type::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    return Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    });
                }

                if left_type_arguments.is_empty() && !right_type_arguments.is_empty() {
                    let left_node = self.resolver.types.get_type_mut(left)?;

                    *left_node = Type::Algebraic {
                        declaration_id: left_declaration_id,
                        type_arguments: right_type_arguments,
                    };
                } else if right_type_arguments.is_empty() && !left_type_arguments.is_empty() {
                    let right_node = self.resolver.types.get_type_mut(right)?;

                    *right_node = Type::Algebraic {
                        declaration_id: right_declaration_id,
                        type_arguments: left_type_arguments,
                    };
                } else {
                    for (left_index, right_index) in left_type_arguments
                        .as_range()
                        .zip(right_type_arguments.as_range())
                    {
                        let left_arg = *self.resolver.types.get_type_member(left_index)?;
                        let right_arg = *self.resolver.types.get_type_member(right_index)?;

                        self.unify_types(left_arg, left_syntax, right_arg, right_syntax)?;
                    }
                }

                Ok(())
            }
            (
                Type::Array {
                    element_type_id: left_element_type_id,
                    length: left_length,
                },
                Type::Array {
                    element_type_id: right_element_type_id,
                    length: right_length,
                },
            ) => {
                if left_length != right_length {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    return Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    });
                }

                self.unify_types(
                    left_element_type_id,
                    left_syntax,
                    right_element_type_id,
                    right_syntax,
                )
            }
            (
                Type::Slice {
                    element_type_id: slice_element_type_id,
                    ..
                },
                Type::Array {
                    element_type_id: array_element_type_id,
                    ..
                },
            ) => self.unify_types(
                slice_element_type_id,
                left_syntax,
                array_element_type_id,
                right_syntax,
            ),
            (left_type_node, right_type_node) => {
                if left_type_node == right_type_node {
                    Ok(())
                } else {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    })
                }
            }
        }
    }
}

impl SyntaxVisitor for TypeBinder<'_> {
    type RootOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = TypeId;
    type TypeOutput = TypeId;
    type PathInput = ();
    type PathOutput = ();

    fn visit_root(&mut self, _: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        Ok(())
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_function_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_struct_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_enum_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_const_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ConstItem {
            name,
            type_annotation: _,
            value,
            ..
        } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let Definition::Constant { type_id, .. } = declaration.definition else {
            return Err(CompileError::ExpectedValue {
                node_kind: reader.node.kind,
                position: reader.position(),
            });
        };

        let value_type_id = self.visit_expression(value, None)?;

        self.unify_types(type_id, Some(reader), value_type_id, value)?;

        Ok(())
    }

    fn visit_type_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            self.visit_item(child)?;
        }

        Ok(())
    }

    fn visit_impl_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplTraitItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            self.visit_item(child)?;
        }

        Ok(())
    }

    fn visit_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::TraitMethod => {}
                SyntaxKind::TraitConst => {
                    let TraitConst {
                        name,
                        type_annotation: _,
                        value,
                    } = child.as_component()?;

                    if let Some(value) = value {
                        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
                        let declaration =
                            self.resolver.declarations.get_declaration(declaration_id)?;
                        let Definition::AssociatedConstant { type_id, .. } = declaration.definition
                        else {
                            return Err(CompileError::ExpectedValue {
                                node_kind: child.node.kind,
                                position: child.position(),
                            });
                        };

                        let value_type_id = self.visit_expression(value, None)?;

                        self.unify_types(type_id, Some(child), value_type_id, value)?;
                    }
                }
                SyntaxKind::TraitType => {}
                _ => {
                    return Err(CompileError::ExpectedSyntaxKinds {
                        expected: &[
                            SyntaxKind::TraitMethod,
                            SyntaxKind::TraitConst,
                            SyntaxKind::TraitType,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        let LetStatement {
            name, expression, ..
        } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let Definition::Local { type_id, .. } = declaration.definition else {
            return Err(CompileError::ExpectedLocalDefinition);
        };
        let expression_type_id = self.visit_expression(expression, None)?;

        self.unify_types(type_id, Some(reader), expression_type_id, expression)?;

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.visit_expression(expression, None)?;

        Ok(())
    }

    fn visit_compound_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let CompoundAssignmentExpression { target, value } = reader.as_component()?;

        let target_type_id = self.visit_expression(target, None)?;
        let value_type_id = self.visit_expression(value, None)?;

        self.unify_types(target_type_id, Some(target), value_type_id, value)?;

        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let AssignmentExpression { target, value } = reader.as_component()?;

        let target_type_id = self.visit_expression(target, None)?;
        let value_type_id = self.visit_expression(value, None)?;

        self.unify_types(target_type_id, Some(target), value_type_id, value)?;

        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_boolean_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Integer));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_character_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Float));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_integer_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Integer));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_array_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayExpression { mut elements } = reader.as_component()?;

        let first_element = elements.next().ok_or(CompileError::ExpectedValue {
            node_kind: reader.node.kind,
            position: reader.position(),
        })?;
        let element_type_id = self.visit_expression(first_element, None)?;

        let mut length = 1;

        for element in elements {
            let element_type = self.visit_expression(element, None)?;

            self.unify_types(element_type_id, Some(first_element), element_type, element)?;

            length += 1;
        }

        let array_type = Type::Array {
            element_type_id,
            length,
        };
        let type_id = self.resolver.types.add_type(array_type);

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayRepeatExpression {
            element,
            length: length_reader,
        } = reader.as_component()?;

        let element_type_id = self.visit_expression(element, None)?;

        let length_str = self.source.get_file_content(&length_reader.position())?;
        let length = create_usize_from_decimal(length_str)?;

        let array_type = Type::Array {
            element_type_id,
            length,
        };
        let type_id = self.resolver.types.add_type(array_type);

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let IndexExpression { list, index } = reader.as_component()?;

        let list_type_id = self.visit_expression(list, None)?;
        let index_type_id = self.visit_expression(index, None)?;

        let list_type = *self.resolver.types.get_type(list_type_id)?;

        let element_type_id = match list_type {
            Type::Array {
                element_type_id, ..
            } => element_type_id,
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: reader.node.kind,
                    position: reader.position(),
                });
            }
        };

        let index_type = *self.resolver.types.get_type(index_type_id)?;

        let result_type_id = if let Type::Algebraic { declaration_id, .. } = index_type {
            let range_symbol = self.resolver.symbols.add_symbol("Range");
            let range_inclusive_symbol = self.resolver.symbols.add_symbol("RangeInclusive");

            let is_range = self
                .resolver
                .declarations
                .find_declaration(range_symbol, ScopeId::CORE, Visibility::Module)
                .is_some_and(|(id, _)| id == declaration_id);
            let is_range_inclusive = self
                .resolver
                .declarations
                .find_declaration(range_inclusive_symbol, ScopeId::CORE, Visibility::Module)
                .is_some_and(|(id, _)| id == declaration_id);

            if is_range || is_range_inclusive {
                list_type_id
            } else {
                element_type_id
            }
        } else {
            element_type_id
        };

        self.resolver.add_type_binding(reader.id, result_type_id);

        Ok(result_type_id)
    }

    fn visit_range_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let start_type_id = self.visit_expression(start, None)?;
        self.visit_expression(end, None)?;

        let symbol_name = if reader.node.kind == SyntaxKind::RangeInclusiveExpression {
            "RangeInclusive"
        } else {
            "Range"
        };

        let symbol_id = self.resolver.symbols.add_symbol(symbol_name);
        let (declaration_id, _) = self
            .resolver
            .declarations
            .find_declaration(symbol_id, ScopeId::CORE, Visibility::Module)
            .ok_or(ResolverError::ExpectedConcreteType)?;

        let type_arguments = self
            .resolver
            .types
            .add_type_members(smallvec![start_type_id]);
        let range_type_id = self.resolver.types.add_type(Type::Algebraic {
            declaration_id,
            type_arguments,
        });

        self.resolver.add_type_binding(reader.id, range_type_id);

        Ok(range_type_id)
    }

    fn visit_path_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let type_id = match declaration.definition {
            Definition::Local { type_id, .. }
            | Definition::Constant { type_id, .. }
            | Definition::AssociatedConstant { type_id, .. } => {
                self.resolver.resolve_type(type_id)?
            }
            Definition::Function {
                type_parameters, ..
            } => {
                let mut turbofish_type_arguments = None;

                if let Some(last_segment) = reader.last_child()? {
                    let PathSegment { type_arguments } = last_segment.as_component()?;

                    if let Some(type_arguments_node) = type_arguments {
                        let types: SmallVec<[TypeId; 4]> = type_arguments_node
                            .children()
                            .map(|type_argument| self.visit_type(type_argument))
                            .try_collect()?;

                        turbofish_type_arguments = Some(types);
                    }
                }

                let type_argument_types: SmallVec<[TypeId; 4]> =
                    if let Some(types) = turbofish_type_arguments {
                        types
                    } else {
                        type_parameters
                            .as_range()
                            .map(|index| {
                                let type_parameter_declaration_id = self
                                    .resolver
                                    .declarations
                                    .get_declaration_member(index)
                                    .copied();

                                match type_parameter_declaration_id {
                                    Ok(type_parameter_declaration_id) => self
                                        .resolver
                                        .type_parameter_map
                                        .get(&type_parameter_declaration_id)
                                        .copied()
                                        .unwrap_or_else(|| {
                                            self.resolver.types.create_inferred_type(None)
                                        }),
                                    Err(_) => self.resolver.types.create_inferred_type(None),
                                }
                            })
                            .collect()
                    };

                let type_arguments = self.resolver.types.add_type_members(type_argument_types);
                let function_definition_type = Type::FunctionDefinition {
                    declaration_id,
                    type_arguments,
                };

                self.resolver.types.add_type(function_definition_type)
            }
            Definition::StructType { .. } | Definition::EnumType { .. } => {
                let type_arguments = self.resolver.types.add_type_members(SmallVec::new());
                let algebraic_type = Type::Algebraic {
                    declaration_id,
                    type_arguments,
                };

                self.resolver.types.add_type(algebraic_type)
            }
            Definition::Variant { parent_enum, .. } => {
                let parent_declaration = self.resolver.declarations.get_declaration(parent_enum)?;
                let Definition::EnumType {
                    type_parameters, ..
                } = parent_declaration.definition
                else {
                    return Err(CompileError::ExpectedValue {
                        node_kind: reader.node.kind,
                        position: reader.position(),
                    });
                };

                let type_argument_types: SmallVec<[TypeId; 4]> = type_parameters
                    .as_range()
                    .map(|index| {
                        let type_parameter_declaration_id = self
                            .resolver
                            .declarations
                            .get_declaration_member(index)
                            .copied();

                        match type_parameter_declaration_id {
                            Ok(type_parameter_declaration_id) => self
                                .resolver
                                .type_parameter_map
                                .get(&type_parameter_declaration_id)
                                .copied()
                                .unwrap_or_else(|| self.resolver.types.create_inferred_type(None)),
                            Err(_) => self.resolver.types.create_inferred_type(None),
                        }
                    })
                    .collect();

                let type_arguments = self.resolver.types.add_type_members(type_argument_types);
                let algebraic_type = Type::Algebraic {
                    declaration_id: parent_enum,
                    type_arguments,
                };

                self.resolver.types.add_type(algebraic_type)
            }
            Definition::Use { item, .. } => {
                let target_declaration = self.resolver.declarations.get_declaration(item)?;

                match target_declaration.definition {
                    Definition::Local { type_id, .. }
                    | Definition::Constant { type_id, .. }
                    | Definition::AssociatedConstant { type_id, .. } => {
                        self.resolver.resolve_type(type_id)?
                    }
                    _ => {
                        let type_arguments = self.resolver.types.add_type_members(SmallVec::new());
                        let algebraic_type = Type::Algebraic {
                            declaration_id: item,
                            type_arguments,
                        };

                        self.resolver.types.add_type(algebraic_type)
                    }
                }
            }
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: reader.node.kind,
                    position: reader.position(),
                });
            }
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let StructExpression { path, fields } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = fields.as_component()?;

        for [field_name, field_value] in name_expression_pairs {
            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_declaration = self
                .resolver
                .declarations
                .get_declaration(field_declaration_id)?;

            let Definition::Field { type_id, .. } = field_declaration.definition else {
                return Err(CompileError::ExpectedValue {
                    node_kind: field_name.node.kind,
                    position: field_name.position(),
                });
            };

            let field_type_id = self.resolver.resolve_type(type_id)?;
            let value_type_id = self.visit_expression(field_value, None)?;

            self.unify_types(field_type_id, Some(field_name), value_type_id, field_value)?;
        }

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;
        let type_arguments = self.resolver.types.add_type_members(SmallVec::new());
        let struct_type = Type::Algebraic {
            declaration_id,
            type_arguments,
        };
        let type_id = self.resolver.types.add_type(struct_type);

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        let type_id = if let Some(inner) = expression {
            self.visit_expression(inner, None)?
        } else {
            TypeId::UNIT
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_block_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let mut block_type_id = TypeId::UNIT;

        for child in reader.children() {
            if child.is_expression() {
                block_type_id = self.visit_expression(child, None)?;
            } else {
                self.visit_statement(child)?;
                block_type_id = TypeId::UNIT;
            }
        }

        self.resolver.add_type_binding(reader.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        let condition_type_id = self.visit_expression(condition, None)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;

        let then_type_id = self.visit_expression(then_branch, None)?;

        if let Some(else_branch) = else_branch {
            let else_type_id = self.visit_expression(else_branch, None)?;

            self.unify_types(then_type_id, Some(then_branch), else_type_id, else_branch)?;
            self.resolver.add_type_binding(reader.id, then_type_id);

            Ok(then_type_id)
        } else {
            self.resolver.add_type_binding(reader.id, TypeId::UNIT);

            Ok(TypeId::UNIT)
        }
    }

    fn visit_math_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, None)?;
        let right_type_id = self.visit_expression(right, None)?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, left_type_id);

        Ok(left_type_id)
    }

    fn visit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, None)?;
        let right_type_id = self.visit_expression(right, None)?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logic_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, None)?;
        let right_type_id = self.visit_expression(right, None)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), left_type_id, left)?;
        self.unify_types(TypeId::BOOLEAN, Some(reader), right_type_id, right)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_negation_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        let operand_type_id = self.visit_expression(operand, None)?;

        self.resolver.add_type_binding(reader.id, operand_type_id);

        Ok(operand_type_id)
    }

    fn visit_not_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        let operand_type_id = self.visit_expression(operand, None)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), operand_type_id, operand)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_while_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        let condition_type_id = self.visit_expression(condition, None)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;
        self.visit_expression(body, None)?;
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_break_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        let callee_type_id = self.visit_expression(callee, None)?;
        let resolved_callee_type_id = self.infer_type(callee_type_id)?;
        let callee_type = *self.resolver.types.get_type(resolved_callee_type_id)?;

        match callee_type {
            Type::FunctionDefinition {
                declaration_id,
                type_arguments,
            } => {
                let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

                let (type_parameters, value_parameters, return_type_id) =
                    match declaration.definition {
                        Definition::Function {
                            type_parameters,
                            value_parameters,
                            return_type_id,
                            ..
                        } => (type_parameters, value_parameters, return_type_id),
                        Definition::NativeFunction {
                            type_parameters,
                            value_parameters,
                            return_type_id,
                            ..
                        } => (type_parameters, value_parameters, return_type_id),
                        _ => {
                            return Err(CompileError::ExpectedFunctionType {
                                found: resolved_callee_type_id,
                                position: callee.position(),
                            });
                        }
                    };

                let mut inserted_type_parameters = SmallVec::<[DeclarationId; 4]>::new();

                if !type_arguments.is_empty() {
                    let type_parameter_declaration_ids = self
                        .resolver
                        .declarations
                        .get_declaration_members(&type_parameters)?;

                    let type_argument_ids = self.resolver.types.get_type_members(type_arguments)?;

                    for (&type_parameter_declaration_id, &type_argument_id) in
                        type_parameter_declaration_ids
                            .iter()
                            .zip(type_argument_ids.iter())
                    {
                        self.resolver
                            .type_parameter_map
                            .insert(type_parameter_declaration_id, type_argument_id);
                        inserted_type_parameters.push(type_parameter_declaration_id);
                    }
                }

                let mut argument_count = 0;
                let mut parameter_range = value_parameters.as_range();

                if callee.node.kind == SyntaxKind::FieldAccessExpression {
                    parameter_range.next(); // For method calls, skip the `self` parameter
                }

                for argument in arguments.children() {
                    let Some(parameter_index) = parameter_range.next() else {
                        return Err(CompileError::ExpectedArguments {
                            function_type: resolved_callee_type_id,
                            expected_count: value_parameters.len() as usize,
                            found_count: arguments.child_count(),
                            found_position: arguments.position(),
                        });
                    };
                    let parameter_type_id =
                        *self.resolver.types.get_type_member(parameter_index)?;
                    let resolved_parameter_type_id =
                        self.resolver.resolve_type(parameter_type_id)?;
                    let argument_type_id = self.visit_expression(argument, None)?;

                    self.unify_types(resolved_parameter_type_id, None, argument_type_id, argument)?;

                    argument_count += 1;
                }

                if parameter_range.next().is_some() {
                    return Err(CompileError::ExpectedArguments {
                        function_type: resolved_callee_type_id,
                        expected_count: value_parameters.len() as usize,
                        found_count: argument_count,
                        found_position: arguments.position(),
                    });
                }

                let resolved_return_type_id = self.resolver.resolve_type(return_type_id)?;

                for type_parameter_declaration_id in inserted_type_parameters {
                    self.resolver
                        .type_parameter_map
                        .remove(&type_parameter_declaration_id);
                }

                self.resolver
                    .add_type_binding(reader.id, resolved_return_type_id);

                Ok(resolved_return_type_id)
            }
            Type::Algebraic { type_arguments, .. } => {
                let callee_declaration_id = *self.resolver.get_declaration_binding(&callee.id)?;
                let callee_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(callee_declaration_id)?;

                let Definition::Variant {
                    parent_enum,
                    fields,
                    ..
                } = callee_declaration.definition
                else {
                    return Err(CompileError::ExpectedFunctionType {
                        found: resolved_callee_type_id,
                        position: callee.position(),
                    });
                };

                let parent_declaration = self.resolver.declarations.get_declaration(parent_enum)?;
                let Definition::EnumType {
                    type_parameters, ..
                } = parent_declaration.definition
                else {
                    return Err(CompileError::ExpectedFunctionType {
                        found: resolved_callee_type_id,
                        position: callee.position(),
                    });
                };

                for (parameter_index, argument_index) in
                    type_parameters.as_range().zip(type_arguments.as_range())
                {
                    let parameter_declaration_id = *self
                        .resolver
                        .declarations
                        .get_declaration_member(parameter_index)?;
                    let argument_type_id = *self.resolver.types.get_type_member(argument_index)?;

                    self.resolver
                        .type_parameter_map
                        .insert(parameter_declaration_id, argument_type_id);
                }

                let mut argument_count = 0;
                let mut field_range = fields.as_range();

                for argument in arguments.children() {
                    let Some(field_index) = field_range.next() else {
                        argument_count += 1;
                        continue;
                    };

                    let field_declaration_id = *self
                        .resolver
                        .declarations
                        .get_declaration_member(field_index)?;
                    let field_declaration = self
                        .resolver
                        .declarations
                        .get_declaration(field_declaration_id)?;

                    let Definition::Field {
                        type_id: field_type_id,
                        ..
                    } = field_declaration.definition
                    else {
                        return Err(CompileError::ExpectedValue {
                            node_kind: callee.node.kind,
                            position: callee.position(),
                        });
                    };

                    let resolved_field_type_id = self.resolver.resolve_type(field_type_id)?;
                    let argument_type_id = self.visit_expression(argument, None)?;

                    self.unify_types(resolved_field_type_id, None, argument_type_id, argument)?;

                    argument_count += 1;
                }

                if argument_count != fields.len() {
                    return Err(CompileError::ExpectedArguments {
                        function_type: resolved_callee_type_id,
                        expected_count: fields.len() as usize,
                        found_count: argument_count as usize,
                        found_position: arguments.position(),
                    });
                }

                self.resolver
                    .add_type_binding(reader.id, resolved_callee_type_id);

                Ok(resolved_callee_type_id)
            }
            _ => Err(CompileError::ExpectedFunctionType {
                found: resolved_callee_type_id,
                position: callee.position(),
            }),
        }
    }

    fn visit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let FieldAccessExpression {
            operand,
            field_name,
        } = reader.as_component()?;

        self.visit_expression(operand, None)?;

        let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
        let field_declaration = self
            .resolver
            .declarations
            .get_declaration(field_declaration_id)?;

        let type_id = match field_declaration.definition {
            Definition::Field { type_id, .. } => self.resolver.resolve_type(type_id)?,
            Definition::Function {
                type_parameters, ..
            } => {
                let type_argument_types: SmallVec<[TypeId; 4]> = type_parameters
                    .as_range()
                    .map(|index| {
                        let type_parameter_declaration_id = self
                            .resolver
                            .declarations
                            .get_declaration_member(index)
                            .copied();

                        match type_parameter_declaration_id {
                            Ok(type_parameter_declaration_id) => self
                                .resolver
                                .type_parameter_map
                                .get(&type_parameter_declaration_id)
                                .copied()
                                .unwrap_or_else(|| self.resolver.types.create_inferred_type(None)),
                            Err(_) => self.resolver.types.create_inferred_type(None),
                        }
                    })
                    .collect();

                let type_arguments = self.resolver.types.add_type_members(type_argument_types);

                self.resolver.types.add_type(Type::FunctionDefinition {
                    declaration_id: field_declaration_id,
                    type_arguments,
                })
            }
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: field_name.node.kind,
                    position: field_name.position(),
                });
            }
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        match reader.node.kind {
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::I8Type => Ok(TypeId::I_8),
            SyntaxKind::I16Type => Ok(TypeId::I_16),
            SyntaxKind::I32Type => Ok(TypeId::I_32),
            SyntaxKind::I64Type => Ok(TypeId::I_64),
            SyntaxKind::I128Type => Ok(TypeId::I_128),
            SyntaxKind::ISizeType => Ok(TypeId::I_SIZE),
            SyntaxKind::U8Type => Ok(TypeId::U_8),
            SyntaxKind::U16Type => Ok(TypeId::U_16),
            SyntaxKind::U32Type => Ok(TypeId::U_32),
            SyntaxKind::U64Type => Ok(TypeId::U_64),
            SyntaxKind::U128Type => Ok(TypeId::U_128),
            SyntaxKind::USizeType => Ok(TypeId::U_SIZE),
            SyntaxKind::F32Type => Ok(TypeId::F_32),
            SyntaxKind::F64Type => Ok(TypeId::F_64),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::SliceType => {
                let element_type = reader.single_child()?;

                let slice_symbol_id = self.resolver.symbols.add_slice_symbol();
                let element_type_id = self.visit_type(element_type)?;
                let declaration_id = self.resolver.declarations.add_declaration(Declaration {
                    symbol_id: slice_symbol_id,
                    definition: Definition::TypeParameter,
                    scope_id: ScopeId::CORE,
                    syntax: None,
                });

                Ok(self.resolver.types.add_type(Type::Slice {
                    declaration_id,
                    element_type_id,
                }))
            }
            SyntaxKind::TupleType => {
                let element_type_ids = reader
                    .children()
                    .map(|element_type| self.visit_type(element_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let element_type_ids = self.resolver.types.add_type_members(element_type_ids);

                Ok(self
                    .resolver
                    .types
                    .add_type(Type::Tuple { element_type_ids }))
            }
            SyntaxKind::FunctionType => {
                let FunctionType {
                    value_parameter_types,
                    return_type,
                } = reader.as_component()?;

                let value_parameter_ids = value_parameter_types
                    .children()
                    .map(|parameter_type| self.visit_type(parameter_type))
                    .try_collect::<SmallVec<[TypeId; 4]>>()?;
                let value_parameters = self.resolver.types.add_type_members(value_parameter_ids);
                let return_type_id = if let Some(return_type) = return_type {
                    self.visit_type(return_type)?
                } else {
                    TypeId::UNIT
                };

                Ok(self.resolver.types.add_type(Type::Function {
                    value_parameters,
                    return_type: return_type_id,
                }))
            }
            SyntaxKind::TypePath => {
                let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
                let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

                match declaration.definition {
                    Definition::TypeParameter => {
                        let type_id = self
                            .resolver
                            .type_parameter_map
                            .get(&declaration_id)
                            .copied()
                            .unwrap_or_else(|| self.resolver.types.create_inferred_type(None));

                        Ok(type_id)
                    }
                    Definition::StructType { .. } | Definition::EnumType { .. } => {
                        let type_arguments = if let Some(last_segment) = reader.last_child()? {
                            let PathSegment { type_arguments } = last_segment.as_component()?;

                            if let Some(type_arguments_node) = type_arguments {
                                let type_argument_types: SmallVec<[TypeId; 4]> =
                                    type_arguments_node
                                        .children()
                                        .map(|type_argument| self.visit_type(type_argument))
                                        .try_collect()?;

                                self.resolver.types.add_type_members(type_argument_types)
                            } else {
                                TypeMembers::default()
                            }
                        } else {
                            TypeMembers::default()
                        };

                        Ok(self.resolver.types.add_type(Type::Algebraic {
                            declaration_id,
                            type_arguments,
                        }))
                    }
                    _ => Err(CompileError::ExpectedTypeDeclaration(declaration_id)),
                }
            }
            _ => Err(CompileError::ExpectedSyntaxKinds {
                expected: &[
                    SyntaxKind::BooleanType,
                    SyntaxKind::I8Type,
                    SyntaxKind::I16Type,
                    SyntaxKind::I32Type,
                    SyntaxKind::I64Type,
                    SyntaxKind::I128Type,
                    SyntaxKind::ISizeType,
                    SyntaxKind::U8Type,
                    SyntaxKind::U16Type,
                    SyntaxKind::U32Type,
                    SyntaxKind::U64Type,
                    SyntaxKind::U128Type,
                    SyntaxKind::USizeType,
                    SyntaxKind::F32Type,
                    SyntaxKind::F64Type,
                    SyntaxKind::CharacterType,
                    SyntaxKind::SliceType,
                    SyntaxKind::TupleType,
                    SyntaxKind::FunctionType,
                    SyntaxKind::TypePath,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_path(&mut self, _: SyntaxReader, _: ()) -> Result<Self::PathOutput, CompileError> {
        Ok(())
    }

    fn visit_simple_path(
        &mut self,
        _: SyntaxReader,
        _: (),
    ) -> Result<Self::PathOutput, CompileError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
