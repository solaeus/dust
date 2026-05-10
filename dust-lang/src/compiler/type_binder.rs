use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            Resolver,
            declarations::{DeclarationId, Definition, VariantKind},
            types::{InferredTypeConstraint, Type, TypeId, TypeMembers},
        },
        value_creation::create_usize_from_decimal,
    },
    error::ErrorKind,
    source::Source,
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, CallExpression,
            ComparisonExpression, ConstItem, ExpressionStatement, FieldAccessExpression,
            GroupedExpression, IfExpression, ImplItem, IndexExpression, LetStatement,
            LogicExpression, MathExpression, MethodCallExpression, NegationExpression,
            NotExpression, PathExpression, PathSegment, RangeExpression, StructExpression,
            StructExpressionStructFields, TraitItem, WhileExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
    },
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    resolver: &'a mut Resolver,

    source: &'a Source<'a>,

    errors: &'a mut Vec<ErrorKind>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        resolver: &'a mut Resolver,
        source: &'a Source<'a>,
        errors: &'a mut Vec<ErrorKind>,
    ) -> Self {
        Self {
            resolver,
            source,
            errors,
        }
    }

    pub fn bind_function_body(&mut self, body: SyntaxReader, return_type_id: TypeId) {
        assert_eq!(body.node.kind, SyntaxKind::BlockExpression);

        let mut body_type_id = TypeId::UNIT;

        for child in body.children() {
            if child.node.kind.is_expression() {
                body_type_id = match self.bind_expression(child) {
                    Ok(type_id) => type_id,
                    Err(error) => {
                        self.errors.push(ErrorKind::Compile(error));

                        TypeId::UNIT
                    }
                };
            } else {
                match self.bind_statement(child) {
                    Ok(()) => {}
                    Err(error) => {
                        self.errors.push(ErrorKind::Compile(error));
                    }
                }

                body_type_id = TypeId::UNIT;
            }
        }

        if let Err(error) = self.unify_types(return_type_id, None, body_type_id, body) {
            self.errors.push(ErrorKind::Compile(error))
        }

        let _ = self.resolver.infer_concrete_type_id(return_type_id);
    }

    fn unify_types<'b>(
        &'b mut self,
        left_id: TypeId,
        left_syntax: Option<SyntaxReader<'b>>,
        right_id: TypeId,
        right_syntax: SyntaxReader<'b>,
    ) -> Result<(), CompileError> {
        if left_id == right_id {
            return Ok(());
        }

        let (left_id, left) = self
            .resolver
            .get_concrete_type(left_id)
            .map(|(id, r#type)| (id, *r#type))?;
        let (right_id, right) = self
            .resolver
            .get_concrete_type(right_id)
            .map(|(id, r#type)| (id, *r#type))?;

        if left == right {
            return Ok(());
        }

        match (left, right) {
            (
                Type::Inferred {
                    constraint: left_constraint,
                    resolved: None,
                    ..
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
                            expected_type: left_id,
                            expected_position,
                            found_type: right_id,
                            found_position,
                        });
                    }
                    (Some(bound), None) => self
                        .resolver
                        .types
                        .constrain_inferred_type(left_id, bound)?,
                    (None, Some(bound)) => self
                        .resolver
                        .types
                        .constrain_inferred_type(right_id, bound)?,

                    _ => {}
                }

                self.resolver.types.resolve_type(left_id, right_id)?;

                Ok(())
            }
            (
                Type::Inferred {
                    constraint,
                    resolved: None,
                    ..
                },
                _,
            ) => {
                if let Some(constraint) = constraint {
                    let satisfied = match constraint {
                        InferredTypeConstraint::Integer => {
                            matches!(right, Type::SignedInteger(_) | Type::UnsignedInteger(_))
                        }
                        InferredTypeConstraint::Float => {
                            matches!(right, Type::Float(_))
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
                            expected_type: left_id,
                            expected_position,
                            found_type: right_id,
                            found_position,
                        });
                    }
                }

                self.resolver.types.resolve_type(left_id, right_id)?;

                Ok(())
            }
            (
                _,
                Type::Inferred {
                    inferred_id: _,
                    constraint,
                    resolved: None,
                },
            ) => {
                if let Some(constraint) = constraint {
                    let satisfied = match constraint {
                        InferredTypeConstraint::Integer => {
                            matches!(left, Type::SignedInteger(_) | Type::UnsignedInteger(_))
                        }
                        InferredTypeConstraint::Float => {
                            matches!(left, Type::Float(_))
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
                            expected_type: left_id,
                            expected_position,
                            found_type: right_id,
                            found_position,
                        });
                    }
                }

                self.resolver.types.resolve_type(right_id, left_id)?;

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
                        expected_type: left_id,
                        expected_position,
                        found_type: right_id,
                        found_position,
                    });
                }

                for (left_index, right_index) in left_type_arguments
                    .as_usize_range()
                    .zip(right_type_arguments.as_usize_range())
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
                    type_arguments: left_arguments,
                },
                Type::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_arguments,
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
                        expected_type: left_id,
                        expected_position,
                        found_type: right_id,
                        found_position,
                    });
                }

                if left_arguments.is_empty() && !right_arguments.is_empty() {
                    self.resolver
                        .types
                        .resolve_type_arguments(left_id, right_arguments)?;
                } else if right_arguments.is_empty() && !left_arguments.is_empty() {
                    self.resolver
                        .types
                        .resolve_type_arguments(right_id, left_arguments)?;
                } else {
                    for (left_index, right_index) in left_arguments
                        .as_usize_range()
                        .zip(right_arguments.as_usize_range())
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
                        expected_type: left_id,
                        expected_position,
                        found_type: right_id,
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
                        expected_type: left_id,
                        expected_position,
                        found_type: right_id,
                        found_position,
                    })
                }
            }
        }
    }

    fn bind_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::ConstItem => self.bind_const_item(reader),
            SyntaxKind::ImplItem => self.bind_impl_item(reader),
            SyntaxKind::TraitItem => self.bind_trait_item(reader),
            SyntaxKind::LetStatement => self.bind_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(reader),
            SyntaxKind::ModItem
            | SyntaxKind::UseItem
            | SyntaxKind::FunctionItem
            | SyntaxKind::StructItem
            | SyntaxKind::EnumItem
            | SyntaxKind::TypeItem => Ok(()),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::ConstItem,
                    SyntaxKind::EnumItem,
                    SyntaxKind::ExpressionStatement,
                    SyntaxKind::FunctionItem,
                    SyntaxKind::ImplItem,
                    SyntaxKind::LetStatement,
                    SyntaxKind::ModItem,
                    SyntaxKind::StructItem,
                    SyntaxKind::TraitItem,
                    SyntaxKind::TypeItem,
                    SyntaxKind::UseItem,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn bind_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        match reader.node.kind {
            SyntaxKind::AssignmentExpression => self.bind_assignment_expression(reader),
            SyntaxKind::BooleanExpression => self.bind_boolean_expression(reader),
            SyntaxKind::HexadecimalExpression => self.bind_hexadecimal_expression(reader),
            SyntaxKind::CharacterExpression => self.bind_character_expression(reader),
            SyntaxKind::FloatExpression => self.bind_float_expression(reader),
            SyntaxKind::IntegerExpression => self.bind_integer_expression(reader),
            SyntaxKind::StringExpression => self.bind_string_expression(reader),
            SyntaxKind::ArrayExpression => self.bind_array_expression(reader),
            SyntaxKind::ArrayRepeatExpression => self.bind_array_repeat_expression(reader),
            SyntaxKind::IndexExpression => self.bind_index_expression(reader),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.bind_range_expression(reader)
            }
            SyntaxKind::PathExpression => self.bind_path_expression(reader),
            SyntaxKind::StructExpression => self.bind_struct_expression(reader),
            SyntaxKind::GroupedExpression => self.bind_grouped_expression(reader),
            SyntaxKind::BlockExpression => self.bind_block_expression(reader),
            SyntaxKind::IfExpression => self.bind_if_expression(reader),
            SyntaxKind::NegationExpression => self.bind_negation_expression(reader),
            SyntaxKind::NotExpression => self.bind_not_expression(reader),
            SyntaxKind::WhileExpression => self.bind_while_expression(reader),
            SyntaxKind::BreakExpression => self.bind_break_expression(reader),
            SyntaxKind::CallExpression => self.bind_call_expression(reader),
            SyntaxKind::MethodCallExpression => self.bind_method_call_expression(reader),
            SyntaxKind::FieldAccessExpression => self.bind_field_access_expression(reader),
            SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentAssignmentExpression => self.bind_math_expression(reader),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.bind_math_expression(reader),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.bind_comparison_expression(reader),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.bind_logic_expression(reader)
            }
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[
                    SyntaxKind::AdditionAssignmentExpression,
                    SyntaxKind::AdditionExpression,
                    SyntaxKind::AndExpression,
                    SyntaxKind::ArrayExpression,
                    SyntaxKind::ArrayRepeatExpression,
                    SyntaxKind::AssignmentExpression,
                    SyntaxKind::BlockExpression,
                    SyntaxKind::BooleanExpression,
                    SyntaxKind::BreakExpression,
                    SyntaxKind::CallExpression,
                    SyntaxKind::CharacterExpression,
                    SyntaxKind::DivisionAssignmentExpression,
                    SyntaxKind::DivisionExpression,
                    SyntaxKind::EqualExpression,
                    SyntaxKind::ExponentAssignmentExpression,
                    SyntaxKind::ExponentExpression,
                    SyntaxKind::FieldAccessExpression,
                    SyntaxKind::FloatExpression,
                    SyntaxKind::GreaterThanExpression,
                    SyntaxKind::GreaterThanOrEqualExpression,
                    SyntaxKind::GroupedExpression,
                    SyntaxKind::HexadecimalExpression,
                    SyntaxKind::IfExpression,
                    SyntaxKind::IndexExpression,
                    SyntaxKind::IntegerExpression,
                    SyntaxKind::LessThanExpression,
                    SyntaxKind::LessThanOrEqualExpression,
                    SyntaxKind::ModuloAssignmentExpression,
                    SyntaxKind::ModuloExpression,
                    SyntaxKind::MultiplicationAssignmentExpression,
                    SyntaxKind::MultiplicationExpression,
                    SyntaxKind::NegationExpression,
                    SyntaxKind::NotEqualExpression,
                    SyntaxKind::NotExpression,
                    SyntaxKind::OrExpression,
                    SyntaxKind::PathExpression,
                    SyntaxKind::RangeExpression,
                    SyntaxKind::RangeInclusiveExpression,
                    SyntaxKind::StringExpression,
                    SyntaxKind::StructExpression,
                    SyntaxKind::SubtractionAssignmentExpression,
                    SyntaxKind::SubtractionExpression,
                    SyntaxKind::WhileExpression,
                ],
                found: reader.node.kind,
            }),
        }
    }

    fn bind_const_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ConstItem {
            name,
            type_notation: _,
            value,
            ..
        } = reader.as_component()?;
        let Some(value) = value else {
            return Ok(());
        };

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id);
        let Definition::Constant {
            type_id: declared_type_id,
            ..
        } = declaration.definition
        else {
            return Err(CompileError::ExpectedConstantDefinition(declaration_id));
        };

        let value_type_id = self.bind_expression(value)?;

        self.unify_types(declared_type_id, Some(reader), value_type_id, value)?;

        Ok(())
    }

    fn bind_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::ConstItem => self.bind_const_item(child)?,
                SyntaxKind::FunctionItem | SyntaxKind::TypeItem => {}
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::ConstItem,
                            SyntaxKind::FunctionItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn bind_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::ConstItem => self.bind_const_item(child)?,
                SyntaxKind::FunctionItem | SyntaxKind::TypeItem => {}
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::ConstItem,
                            SyntaxKind::FunctionItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn bind_let_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LetStatement {
            name, expression, ..
        } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id);
        let Definition::Local {
            type_id: declared_type_id,
            ..
        } = declaration.definition
        else {
            return Err(CompileError::ExpectedLocalDefinition(declaration_id));
        };
        let expression_type_id = self.bind_expression(expression)?;

        self.unify_types(
            declared_type_id,
            Some(reader),
            expression_type_id,
            expression,
        )?;

        Ok(())
    }

    fn bind_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.bind_expression(expression)?;

        Ok(())
    }

    fn bind_assignment_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let AssignmentExpression { target, source } = reader.as_component()?;

        let target_type_id = self.bind_expression(target)?;
        let value_type_id = self.bind_expression(source)?;

        self.unify_types(target_type_id, Some(target), value_type_id, source)?;
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn bind_boolean_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn bind_hexadecimal_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<TypeId, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Integer));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_character_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn bind_float_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Float));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_integer_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Integer));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_string_expression(&mut self, _: SyntaxReader) -> Result<TypeId, CompileError> {
        todo!()
    }

    fn bind_array_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let ArrayExpression { mut elements } = reader.as_component()?;

        let first_element = elements.expect_next()?;
        let element_type_id = self.bind_expression(first_element)?;

        let mut length = 1;

        for element in elements {
            let element_type = self.bind_expression(element)?;

            self.unify_types(element_type_id, Some(first_element), element_type, element)?;

            length += 1;
        }

        let array_type_id = self.resolver.types.add_type(Type::Array {
            element_type_id,
            length,
        });

        self.resolver.add_type_binding(reader.id, array_type_id);

        Ok(array_type_id)
    }

    fn bind_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<TypeId, CompileError> {
        let ArrayRepeatExpression { element, length } = reader.as_component()?;

        let length_str = self.source.get_content(length.position())?;
        let length = create_usize_from_decimal(length_str)?;

        let element_type_id = self.bind_expression(element)?;
        let array_type_id = self.resolver.types.add_type(Type::Array {
            element_type_id,
            length,
        });

        self.resolver.add_type_binding(reader.id, array_type_id);

        Ok(array_type_id)
    }

    fn bind_index_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let IndexExpression { collection, index } = reader.as_component()?;

        let collection_type_id = self.bind_expression(collection)?;
        let index_type_id = self.bind_expression(index)?;

        let collection_type = *self.resolver.types.get_type(collection_type_id);
        let element_type_id = match collection_type {
            Type::Array {
                element_type_id, ..
            } => element_type_id,
            _ => {
                return Err(CompileError::ExpectedIndexableType {
                    type_id: collection_type_id,
                });
            }
        };
        let index_type = *self.resolver.types.get_type(index_type_id);
        let result_type_id = if let Type::Algebraic { declaration_id, .. } = index_type
            && matches!(
                declaration_id,
                DeclarationId::RANGE | DeclarationId::RANGE_INCLUSIVE
            ) {
            collection_type_id
        } else {
            element_type_id
        };

        self.resolver.add_type_binding(reader.id, result_type_id);

        Ok(result_type_id)
    }

    fn bind_range_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let start_type_id = self.bind_expression(start)?;

        self.bind_expression(end)?;

        let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
        let type_arguments = self.resolver.types.add_type_members([start_type_id]);
        let range_type_id = self.resolver.types.add_type(Type::Algebraic {
            declaration_id,
            type_arguments,
        });

        self.resolver.add_type_binding(reader.id, range_type_id);

        Ok(range_type_id)
    }

    fn bind_path_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        fn collect_turbofish_arguments(
            resolver: &mut Resolver,
            reader: SyntaxReader,
        ) -> Result<TypeMembers, CompileError> {
            let PathExpression { segments } = reader.as_component()?;

            let mut turbofish_arguments: TypeId::SmallVec = SmallVec::new();

            for segment in segments {
                let PathSegment { type_arguments } = segment.as_component()?;
                let Some(type_arguments) = type_arguments else {
                    continue;
                };

                for type_argument in type_arguments.children() {
                    let type_id = *resolver.get_type_binding(&type_argument.id)?;

                    turbofish_arguments.push(type_id);
                }
            }

            Ok(resolver.types.add_type_members(turbofish_arguments))
        }

        let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id);
        let type_id = match declaration.definition {
            Definition::Local { type_id, .. }
            | Definition::Constant { type_id, .. }
            | Definition::InherentAssociatedConstant { type_id, .. }
            | Definition::TraitAssociatedConstant { type_id, .. } => {
                self.resolver.get_concrete_type(type_id)?.0
            }
            Definition::Function { .. } => {
                let type_arguments = collect_turbofish_arguments(self.resolver, reader)?;

                self.resolver.types.add_type(Type::FunctionDefinition {
                    declaration_id,
                    type_arguments,
                })
            }
            Definition::Variant {
                enum_declaration_id,
                kind,
                ..
            } => match kind {
                VariantKind::Unit => {
                    let type_arguments = collect_turbofish_arguments(self.resolver, reader)?;

                    self.resolver.types.add_type(Type::Algebraic {
                        declaration_id: enum_declaration_id,
                        type_arguments,
                    })
                }
                VariantKind::TupleFields => {
                    let type_arguments = collect_turbofish_arguments(self.resolver, reader)?;

                    self.resolver.types.add_type(Type::FunctionDefinition {
                        declaration_id,
                        type_arguments,
                    })
                }
                VariantKind::NamedFields => todo!("Determine if this is reachable"),
            },
            Definition::Use {
                source_declaration_id,
                ..
            } => {
                let target_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(source_declaration_id);

                match target_declaration.definition {
                    Definition::Local { type_id, .. }
                    | Definition::Constant { type_id, .. }
                    | Definition::InherentAssociatedConstant { type_id, .. }
                    | Definition::TraitAssociatedConstant { type_id, .. } => {
                        self.resolver.get_concrete_type(type_id)?.0
                    }
                    _ => {
                        return Err(CompileError::ExpectedValue {
                            source_id: reader.source_id(),
                            syntax_id: reader.id,
                        });
                    }
                }
            }
            Definition::ForwardReference {
                resolved: Some(resolved),
            } => {
                let resolved_declaration = self.resolver.declarations.get_declaration(resolved);

                match resolved_declaration.definition {
                    Definition::Local { type_id, .. }
                    | Definition::Constant { type_id, .. }
                    | Definition::InherentAssociatedConstant { type_id, .. }
                    | Definition::TraitAssociatedConstant { type_id, .. } => {
                        self.resolver.get_concrete_type(type_id)?.0
                    }
                    _ => {
                        let type_arguments = collect_turbofish_arguments(self.resolver, reader)?;
                        let algebraic_type = Type::Algebraic {
                            declaration_id: resolved,
                            type_arguments,
                        };

                        self.resolver.types.add_type(algebraic_type)
                    }
                }
            }
            _ => {
                todo!("Handle {:?} {:?}", declaration.definition, declaration_id)
            }
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_struct_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let StructExpression { path, fields } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = fields.as_component()?;

        for (field_name, field_value) in name_expression_pairs {
            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_declaration = self
                .resolver
                .declarations
                .get_declaration(field_declaration_id);

            let Definition::Field { type_id, .. } = field_declaration.definition else {
                return Err(CompileError::ExpectedFieldDefinition(field_declaration_id));
            };

            let field_type_id = self.resolver.get_concrete_type(type_id)?.0;
            let value_type_id = self.bind_expression(field_value)?;

            self.unify_types(field_type_id, Some(field_name), value_type_id, field_value)?;
        }

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;
        let struct_type = Type::Algebraic {
            declaration_id,
            type_arguments: TypeMembers::default(),
        };
        let type_id = self.resolver.types.add_type(struct_type);

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_grouped_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        let type_id = if let Some(inner) = expression {
            self.bind_expression(inner)?
        } else {
            TypeId::UNIT
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn bind_block_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let mut block_type_id = TypeId::UNIT;

        for child in reader.children() {
            if child.node.kind.is_expression() {
                block_type_id = self.bind_expression(child)?;
            } else {
                self.bind_statement(child)?;
                block_type_id = TypeId::UNIT;
            }
        }

        self.resolver.add_type_binding(reader.id, block_type_id);

        Ok(block_type_id)
    }

    fn bind_if_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        let condition_type_id = self.bind_expression(condition)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;

        let then_type_id = self.bind_expression(then_branch)?;

        if let Some(else_branch) = else_branch {
            let else_type_id = self.bind_expression(else_branch)?;

            self.unify_types(then_type_id, Some(then_branch), else_type_id, else_branch)?;
            self.resolver.add_type_binding(reader.id, then_type_id);

            Ok(then_type_id)
        } else {
            self.resolver.add_type_binding(reader.id, TypeId::UNIT);

            Ok(TypeId::UNIT)
        }
    }

    fn bind_math_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        let left_type_id = self.bind_expression(left)?;
        let right_type_id = self.bind_expression(right)?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, left_type_id);

        if matches!(reader.node.kind, SyntaxKind::AdditionAssignmentExpression) {
            Ok(TypeId::UNIT)
        } else {
            Ok(left_type_id)
        }
    }

    fn bind_comparison_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_type_id = self.bind_expression(left)?;
        let right_type_id = self.bind_expression(right)?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn bind_logic_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        let left_type_id = self.bind_expression(left)?;
        let right_type_id = self.bind_expression(right)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), left_type_id, left)?;
        self.unify_types(TypeId::BOOLEAN, Some(reader), right_type_id, right)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn bind_negation_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        let operand_type_id = self.bind_expression(operand)?;

        self.resolver.add_type_binding(reader.id, operand_type_id);

        Ok(operand_type_id)
    }

    fn bind_not_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        let operand_type_id = self.bind_expression(operand)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), operand_type_id, operand)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn bind_while_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        let condition_type_id = self.bind_expression(condition)?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;
        self.bind_expression(body)?;
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn bind_break_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn bind_call_expression(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        let callee_type_id = self.bind_expression(callee)?;
        let callee_type = *self.resolver.types.get_type(callee_type_id);
        let Type::FunctionDefinition {
            declaration_id,
            type_arguments,
        } = callee_type
        else {
            return Err(CompileError::ExpectedFunctionType {
                found: callee_type_id,
                position: callee.position(),
            });
        };
        let (value_parameter_type_ids, return_type_id) =
            self.resolver
                .get_signature(declaration_id, type_arguments, None)?;

        for (argument, expected_type_id) in arguments.children().zip(value_parameter_type_ids) {
            let actual_type_id = self.bind_expression(argument)?;

            self.unify_types(expected_type_id, Some(argument), actual_type_id, argument)?;
        }

        self.resolver.add_type_binding(reader.id, return_type_id);

        Ok(return_type_id)
    }

    fn bind_method_call_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<TypeId, CompileError> {
        let MethodCallExpression {
            method_parent,
            method,
            type_arguments,
            value_arguments,
        } = reader.as_component()?;

        let parent_type_id = self.bind_expression(method_parent)?;
        let Type::Algebraic {
            declaration_id: parent_declaration_id,
            ..
        } = *self.resolver.types.get_type(parent_type_id)
        else {
            return Err(CompileError::ExpectedAlgebraicType(parent_type_id));
        };

        let method_symbol_str = self.source.get_content(method.position())?;
        let method_symbol_id = self.resolver.symbols.add_symbol(method_symbol_str);

        let method_declaration_id = self
            .resolver
            .find_method(method_symbol_id, parent_declaration_id)
            .ok_or_else(|| CompileError::Undeclared {
                symbol_id: method_symbol_id,
                usage_position: method.position(),
            })?;

        let method_type_arguments = if let Some(type_arguments) = type_arguments {
            let mut type_argument_ids = TypeId::SmallVec::new();

            for type_argument in type_arguments.children() {
                let type_argument_id = *self.resolver.get_type_binding(&type_argument.id)?;

                type_argument_ids.push(type_argument_id);
            }

            self.resolver.types.add_type_members(type_argument_ids)
        } else {
            TypeMembers::default()
        };
        let method_type_id = self.resolver.types.add_type(Type::FunctionDefinition {
            declaration_id: method_declaration_id,
            type_arguments: method_type_arguments,
        });

        let (value_parameter_type_ids, return_type_id) = self.resolver.get_signature(
            method_declaration_id,
            method_type_arguments,
            Some(parent_type_id),
        )?;
        let value_parameter_ids_without_self = value_parameter_type_ids.into_iter().skip(1);

        if let Some(value_arguments) = value_arguments {
            for (argument, expected_type_id) in value_arguments
                .children()
                .zip(value_parameter_ids_without_self)
            {
                let actual_type_id = self.bind_expression(argument)?;

                self.unify_types(expected_type_id, Some(argument), actual_type_id, argument)?;
            }
        }

        self.resolver
            .add_declaration_binding(method.id, method_declaration_id);
        self.resolver.add_type_binding(method.id, method_type_id);
        self.resolver.add_type_binding(reader.id, return_type_id);

        Ok(return_type_id)
    }

    fn bind_field_access_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<TypeId, CompileError> {
        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = reader.as_component()?;

        let parent_type_id = self.bind_expression(struct_expression)?;
        let Type::Algebraic {
            declaration_id: parent_declaration_id,
            ..
        } = *self.resolver.types.get_type(parent_type_id)
        else {
            return Err(CompileError::ExpectedAlgebraicType(parent_type_id));
        };

        let field_symbol_str = self.source.get_content(field_name.position())?;
        let field_symbol_id = self.resolver.symbols.add_symbol(field_symbol_str);
        let field_declaration_id = self.resolver.find_member_declaration(
            field_symbol_id,
            parent_declaration_id,
            field_name,
        )?;
        let Definition::Field {
            type_id: field_raw_type_id,
            ..
        } = self
            .resolver
            .declarations
            .get_declaration(field_declaration_id)
            .definition
        else {
            return Err(CompileError::ExpectedFieldDefinition(field_declaration_id));
        };
        let field_type_id = self.resolver.get_concrete_type(field_raw_type_id)?.0;

        self.resolver
            .add_declaration_binding(field_name.id, field_declaration_id);
        self.resolver
            .add_declaration_binding(reader.id, field_declaration_id);

        self.resolver.add_type_binding(reader.id, field_type_id);

        Ok(field_raw_type_id)
    }
}
