#[cfg(test)]
mod tests;

use smallvec::SmallVec;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            Resolver,
            declarations::{Declaration, DeclarationId, Definition},
            scopes::ScopeId,
            types::{InferredTypeConstraint, Type, TypeId, TypeMembers},
        },
        value_creation::create_usize_from_decimal,
    },
    source::Source,
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, CallExpression,
            ComparisonExpression, ConstItem, ExpressionStatement, FieldAccessExpression,
            FunctionType, GroupedExpression, IfExpression, ImplItem, IndexExpression, LetStatement,
            LogicExpression, MathExpression, NegationExpression, NotExpression, PathSegment,
            RangeExpression, StructExpression, StructExpressionStructFields, TraitItem,
            WhileExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
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
            if child.node.kind.is_expression() {
                body_type_id = self.visit_expression(child, ())?;
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

impl TypeBinder<'_> {
    fn visit_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::LetStatement => self.visit_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[SyntaxKind::LetStatement, SyntaxKind::ExpressionStatement],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_expression(
        &mut self,
        reader: SyntaxReader,
        input: (),
    ) -> Result<TypeId, CompileError> {
        match reader.node.kind {
            SyntaxKind::AssignmentExpression => self.visit_assignment_expression(reader, input),
            SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentAssignmentExpression => self.visit_math_expression(reader, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(reader, input),
            SyntaxKind::HexadecimalExpression => self.visit_hexadecimal_expression(reader, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(reader, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(reader, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(reader, input),
            SyntaxKind::StringExpression => self.visit_string_expression(reader, input),
            SyntaxKind::ArrayExpression => self.visit_array_expression(reader, input),
            SyntaxKind::ArrayRepeatExpression => self.visit_array_repeat_expression(reader, input),
            SyntaxKind::IndexExpression => self.visit_index_expression(reader, input),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.visit_range_expression(reader, input)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(reader, input),
            SyntaxKind::StructExpression => self.visit_struct_expression(reader, input),
            SyntaxKind::GroupedExpression => self.visit_grouped_expression(reader, input),
            SyntaxKind::BlockExpression => self.visit_block_expression(reader, input),
            SyntaxKind::IfExpression => self.visit_if_expression(reader, input),
            SyntaxKind::NegationExpression => self.visit_negation_expression(reader, input),
            SyntaxKind::NotExpression => self.visit_not_expression(reader, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(reader, input),
            SyntaxKind::BreakExpression => self.visit_break_expression(reader, input),
            SyntaxKind::CallExpression => self.visit_call_expression(reader, input),
            SyntaxKind::FieldAccessExpression => self.visit_field_access_expression(reader, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_expression(reader, input),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.visit_comparison_expression(reader, input),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logic_expression(reader, input)
            }
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[],
                found: reader.node.kind,
            }),
        }
    }

    fn visit_root(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
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
            type_notation: _,
            value,
            ..
        } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let Definition::Constant {
            type_id: declared_type_id,
            ..
        } = declaration.definition
        else {
            return Err(CompileError::ExpectedConstantDefinition(declaration_id));
        };

        let value_type_id = self.visit_expression(value, ())?;

        self.unify_types(declared_type_id, Some(reader), value_type_id, value)?;

        Ok(())
    }

    fn visit_type_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ImplItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::FnItem | SyntaxKind::ConstItem | SyntaxKind::TypeItem => {}
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FnItem,
                            SyntaxKind::ConstItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let TraitItem { body, .. } = reader.as_component()?;

        for child in body.children() {
            match child.node.kind {
                SyntaxKind::ConstItem => {
                    let ConstItem {
                        name,
                        type_notation: _,
                        value,
                        ..
                    } = child.as_component()?;

                    if let Some(value) = value {
                        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
                        let declaration =
                            self.resolver.declarations.get_declaration(declaration_id)?;
                        let Definition::TraitAssociatedConstant { type_id, .. } =
                            declaration.definition
                        else {
                            return Err(CompileError::ExpectedConstantDefinition(declaration_id));
                        };

                        let value_type_id = self.visit_expression(value, ())?;

                        self.unify_types(type_id, Some(child), value_type_id, value)?;
                    }
                }
                SyntaxKind::FnItem | SyntaxKind::TypeItem => {}
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[
                            SyntaxKind::FnItem,
                            SyntaxKind::ConstItem,
                            SyntaxKind::TypeItem,
                        ],
                        found: child.node.kind,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_let_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let LetStatement {
            name, expression, ..
        } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let Definition::Local { type_id, .. } = declaration.definition else {
            return Err(CompileError::ExpectedLocalDefinition);
        };
        let expression_type_id = self.visit_expression(expression, ())?;

        self.unify_types(type_id, Some(reader), expression_type_id, expression)?;

        Ok(())
    }

    fn visit_expression_statement(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_assignment_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let AssignmentExpression { target, source } = reader.as_component()?;

        let target_type_id = self.visit_expression(target, ())?;
        let value_type_id = self.visit_expression(source, ())?;

        self.unify_types(target_type_id, Some(target), value_type_id, source)?;

        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_boolean_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_hexadecimal_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
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
        _: (),
    ) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
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
        _: (),
    ) -> Result<TypeId, CompileError> {
        let type_id = self
            .resolver
            .types
            .create_inferred_type(Some(InferredTypeConstraint::Integer));

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_string_expression(&mut self, _: SyntaxReader, _: ()) -> Result<TypeId, CompileError> {
        todo!()
    }

    fn visit_array_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let ArrayExpression { mut elements } = reader.as_component()?;

        let first_element = elements.expect_next()?;
        let element_type_id = self.visit_expression(first_element, ())?;

        let mut length = 1;

        for element in elements {
            let element_type = self.visit_expression(element, ())?;

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
        _: (),
    ) -> Result<TypeId, CompileError> {
        let ArrayRepeatExpression { element, length } = reader.as_component()?;

        let element_type_id = self.visit_expression(element, ())?;

        let length_str = self.source.get_content(length.position())?;
        let length_value = create_usize_from_decimal(length_str)?;

        let array_type = Type::Array {
            element_type_id,
            length: length_value,
        };
        let type_id = self.resolver.types.add_type(array_type);

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let IndexExpression { collection, index } = reader.as_component()?;

        let list_type_id = self.visit_expression(collection, ())?;
        let index_type_id = self.visit_expression(index, ())?;

        let list_type = *self.resolver.types.get_type(list_type_id)?;

        let element_type_id = match list_type {
            Type::Array {
                element_type_id, ..
            } => element_type_id,
            _ => {
                return Err(CompileError::ExpectedIndexableType {
                    type_id: list_type_id,
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
                .find_declaration(range_symbol, ScopeId::CORE)
                .is_some_and(|(id, _)| id == declaration_id);
            let is_range_inclusive = self
                .resolver
                .declarations
                .find_declaration(range_inclusive_symbol, ScopeId::CORE)
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
        _: (),
    ) -> Result<TypeId, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let start_type_id = self.visit_expression(start, ())?;
        self.visit_expression(end, ())?;

        let symbol_name = if reader.node.kind == SyntaxKind::RangeInclusiveExpression {
            "RangeInclusive"
        } else {
            "Range"
        };

        let symbol_id = self.resolver.symbols.add_symbol(symbol_name);
        let (declaration_id, _) = self
            .resolver
            .declarations
            .find_declaration(symbol_id, ScopeId::CORE)
            .ok_or(CompileError::ExpectedConcreteType)?;

        let type_arguments = self.resolver.types.add_type_members([start_type_id]);
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
        _: (),
    ) -> Result<TypeId, CompileError> {
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

                let type_argument_types: SmallVec<[TypeId; 4]> = if let Some(types) =
                    turbofish_type_arguments
                {
                    types
                } else {
                    self.resolver
                        .scopes
                        .get_namespace_entries(type_parameters)
                        .iter()
                        .map(|&(_, type_parameter_declaration_id)| {
                            self.resolver
                                .type_parameter_map
                                .get(&type_parameter_declaration_id)
                                .copied()
                                .unwrap_or_else(|| self.resolver.types.create_inferred_type(None))
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
                let algebraic_type = Type::Algebraic {
                    declaration_id,
                    type_arguments: TypeMembers::default(),
                };

                self.resolver.types.add_type(algebraic_type)
            }
            Definition::Variant {
                enum_declaration_id: parent_enum,
                ..
            } => {
                let parent_declaration = self.resolver.declarations.get_declaration(parent_enum)?;
                let Definition::EnumType {
                    type_parameters, ..
                } = parent_declaration.definition
                else {
                    return Err(CompileError::ExpectedEnumDefinition(parent_enum));
                };

                let type_argument_types: SmallVec<[TypeId; 4]> = self
                    .resolver
                    .scopes
                    .get_namespace_entries(type_parameters)
                    .iter()
                    .map(|&(_, type_parameter_declaration_id)| {
                        self.resolver
                            .type_parameter_map
                            .get(&type_parameter_declaration_id)
                            .copied()
                            .unwrap_or_else(|| self.resolver.types.create_inferred_type(None))
                    })
                    .collect();

                let type_arguments = self.resolver.types.add_type_members(type_argument_types);
                let algebraic_type = Type::Algebraic {
                    declaration_id: parent_enum,
                    type_arguments,
                };

                self.resolver.types.add_type(algebraic_type)
            }
            Definition::Use {
                source_declaration_id,
                ..
            } => {
                let target_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(source_declaration_id)?;

                match target_declaration.definition {
                    Definition::Local { type_id, .. }
                    | Definition::Constant { type_id, .. }
                    | Definition::AssociatedConstant { type_id, .. } => {
                        self.resolver.resolve_type(type_id)?
                    }
                    _ => {
                        let algebraic_type = Type::Algebraic {
                            declaration_id: source_declaration_id,
                            type_arguments: TypeMembers::default(),
                        };

                        self.resolver.types.add_type(algebraic_type)
                    }
                }
            }
            _ => {
                todo!()
            }
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
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
                return Err(CompileError::ExpectedFieldDefinition(field_declaration_id));
            };

            let field_type_id = self.resolver.resolve_type(type_id)?;
            let value_type_id = self.visit_expression(field_value, ())?;

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

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;

        let type_id = if let Some(inner) = expression {
            self.visit_expression(inner, ())?
        } else {
            TypeId::UNIT
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_block_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let mut block_type_id = TypeId::UNIT;

        for child in reader.children() {
            if child.node.kind.is_expression() {
                block_type_id = self.visit_expression(child, ())?;
            } else {
                self.visit_statement(child)?;
                block_type_id = TypeId::UNIT;
            }
        }

        self.resolver.add_type_binding(reader.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(&mut self, reader: SyntaxReader, _: ()) -> Result<TypeId, CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        let condition_type_id = self.visit_expression(condition, ())?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;

        let then_type_id = self.visit_expression(then_branch, ())?;

        if let Some(else_branch) = else_branch {
            let else_type_id = self.visit_expression(else_branch, ())?;

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
        _: (),
    ) -> Result<TypeId, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, ())?;
        let right_type_id = self.visit_expression(right, ())?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, left_type_id);

        if matches!(reader.node.kind, SyntaxKind::AdditionAssignmentExpression) {
            Ok(TypeId::UNIT)
        } else {
            Ok(left_type_id)
        }
    }

    fn visit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, ())?;
        let right_type_id = self.visit_expression(right, ())?;

        self.unify_types(left_type_id, Some(reader), right_type_id, right)?;

        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logic_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        let left_type_id = self.visit_expression(left, ())?;
        let right_type_id = self.visit_expression(right, ())?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), left_type_id, left)?;
        self.unify_types(TypeId::BOOLEAN, Some(reader), right_type_id, right)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_negation_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        let operand_type_id = self.visit_expression(operand, ())?;

        self.resolver.add_type_binding(reader.id, operand_type_id);

        Ok(operand_type_id)
    }

    fn visit_not_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        let operand_type_id = self.visit_expression(operand, ())?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), operand_type_id, operand)?;
        self.resolver.add_type_binding(reader.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_while_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        let condition_type_id = self.visit_expression(condition, ())?;

        self.unify_types(TypeId::BOOLEAN, Some(reader), condition_type_id, condition)?;
        self.visit_expression(body, ())?;
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_break_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        self.resolver.add_type_binding(reader.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<TypeId, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        let callee_type_id = self.visit_expression(callee, ())?;
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
                let parameter_entries =
                    self.resolver.scopes.get_namespace_entries(value_parameters);
                let mut parameter_iter = parameter_entries.iter();

                if callee.node.kind == SyntaxKind::FieldAccessExpression {
                    parameter_iter.next(); // For method calls, skip the `self` parameter
                }

                for argument in arguments.children() {
                    let Some(&(_, parameter_declaration_id)) = parameter_iter.next() else {
                        return Err(CompileError::ExpectedArguments {
                            function_type: resolved_callee_type_id,
                            expected_count: parameter_entries.len(),
                            found_count: arguments.child_count(),
                            found_position: arguments.position(),
                        });
                    };
                    let parameter_declaration = self
                        .resolver
                        .declarations
                        .get_declaration(parameter_declaration_id)?;
                    let parameter_type_id = match parameter_declaration.definition {
                        Definition::Local { type_id, .. } => type_id,
                        _ => continue,
                    };
                    let resolved_parameter_type_id =
                        self.resolver.resolve_type(parameter_type_id)?;
                    let argument_type_id = self.visit_expression(argument, ())?;

                    self.unify_types(resolved_parameter_type_id, None, argument_type_id, argument)?;

                    argument_count += 1;
                }

                if parameter_iter.next().is_some() {
                    return Err(CompileError::ExpectedArguments {
                        function_type: resolved_callee_type_id,
                        expected_count: parameter_entries.len(),
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
                    enum_declaration_id: parent_enum,
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

                let type_parameter_entries =
                    self.resolver.scopes.get_namespace_entries(type_parameters);
                for (&(_, parameter_declaration_id), argument_index) in
                    type_parameter_entries.iter().zip(type_arguments.as_range())
                {
                    let argument_type_id = *self.resolver.types.get_type_member(argument_index)?;

                    self.resolver
                        .type_parameter_map
                        .insert(parameter_declaration_id, argument_type_id);
                }

                let mut argument_count = 0;
                let field_entries = self.resolver.scopes.get_namespace_entries(fields);
                let mut field_iter = field_entries.iter();

                for argument in arguments.children() {
                    let Some(&(_, field_declaration_id)) = field_iter.next() else {
                        argument_count += 1;
                        continue;
                    };

                    let field_declaration = self
                        .resolver
                        .declarations
                        .get_declaration(field_declaration_id)?;

                    let Definition::Field {
                        type_id: field_type_id,
                        ..
                    } = field_declaration.definition
                    else {
                        return Err(CompileError::ExpectedFieldDefinition(field_declaration_id));
                    };

                    let resolved_field_type_id = self.resolver.resolve_type(field_type_id)?;
                    let argument_type_id = self.visit_expression(argument, ())?;

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
        _: (),
    ) -> Result<TypeId, CompileError> {
        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = reader.as_component()?;

        self.visit_expression(struct_expression, ())?;

        let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
        let field_declaration = self
            .resolver
            .declarations
            .get_declaration(field_declaration_id)?;
        let type_id = if let Definition::Field { type_id, .. } = field_declaration.definition {
            self.resolver.resolve_type(type_id)?
        } else {
            return Err(CompileError::ExpectedFieldDefinition(field_declaration_id));
        };

        self.resolver.add_type_binding(reader.id, type_id);

        Ok(type_id)
    }

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<TypeId, CompileError> {
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
                    return_type_id,
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
            _ => Err(CompileError::UnexpectedSyntax {
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

    fn visit_path(&mut self, _: SyntaxReader, _: ()) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_simple_path(&mut self, _: SyntaxReader, _: ()) -> Result<(), CompileError> {
        Ok(())
    }
}
