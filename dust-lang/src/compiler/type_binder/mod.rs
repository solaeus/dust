use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    resolver::{
        Resolver,
        declarations::Definition,
        types::{InferredTypeConstraint, Type, TypeId},
    },
    syntax::{
        Syntax,
        components::{
            AssignmentExpression, CallExpression, ComparisonExpression,
            CompoundAssignmentExpression, ExpressionStatement, GroupedExpression, IfExpression,
            LetStatement, LogicExpression, MathExpression, NegationExpression, NotExpression,
            StructExpression, StructExpressionStructFields, WhileExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
        visitor::SyntaxVisitor,
    },
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<ErrorKind>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
    ) -> Self {
        Self {
            syntax,
            resolver,
            errors,
        }
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

    pub fn infer_type(&self, type_id: TypeId) -> Result<TypeId, CompileError> {
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

    pub fn unify_types(
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

    pub fn unify_inferred_types<'b>(
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
    type PathOutput = TypeId;

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
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_list_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_path_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let type_id = match declaration.definition {
            Definition::Local { type_id, .. } => self.resolver.resolve_type(type_id)?,
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
            Definition::Use { item, .. } => {
                let target_declaration = self.resolver.declarations.get_declaration(item)?;

                match target_declaration.definition {
                    Definition::Local { type_id, .. } => self.resolver.resolve_type(type_id)?,
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

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        let callee_type_id = self.visit_expression(callee, None)?;
        let resolved_callee_type_id = self.infer_type(callee_type_id)?;
        let callee_type = *self.resolver.types.get_type(resolved_callee_type_id)?;
        let (value_parameters, return_type_id) = match callee_type {
            Type::FunctionDefinition { declaration_id, .. } => {
                let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

                match declaration.definition {
                    Definition::Function {
                        value_parameters,
                        return_type_id,
                        ..
                    } => (value_parameters, return_type_id),
                    Definition::NativeFunction {
                        value_parameters,
                        return_type_id,
                        ..
                    } => (value_parameters, return_type_id),
                    _ => {
                        return Err(CompileError::ExpectedFunctionType {
                            found: resolved_callee_type_id,
                            position: callee.position(),
                        });
                    }
                }
            }
            _ => {
                return Err(CompileError::ExpectedFunctionType {
                    found: resolved_callee_type_id,
                    position: callee.position(),
                });
            }
        };

        let mut argument_count = 0;
        let mut parameter_range = value_parameters.as_range();

        for argument in arguments.children() {
            let Some(parameter_index) = parameter_range.next() else {
                return Err(CompileError::ExpectedArguments {
                    function_type: resolved_callee_type_id,
                    expected_count: value_parameters.len(),
                    found_count: arguments.child_count(),
                    found_position: arguments.position(),
                });
            };
            let parameter_type_id = *self.resolver.types.get_type_member(parameter_index)?;
            let resolved_parameter_type_id = self.resolver.resolve_type(parameter_type_id)?;
            let argument_type_id = self.visit_expression(argument, None)?;

            self.unify_types(resolved_parameter_type_id, None, argument_type_id, argument)?;

            argument_count += 1;
        }

        if parameter_range.next().is_some() {
            return Err(CompileError::ExpectedArguments {
                function_type: resolved_callee_type_id,
                expected_count: value_parameters.len(),
                found_count: argument_count,
                found_position: arguments.position(),
            });
        }

        let resolved_return_type_id = self.resolver.resolve_type(return_type_id)?;

        self.resolver
            .add_type_binding(reader.id, resolved_return_type_id);

        Ok(resolved_return_type_id)
    }

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        todo!()
    }

    fn visit_path(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }

    fn visit_simple_path(
        &mut self,
        reader: SyntaxReader,
        _: (),
    ) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }
}

#[cfg(test)]
mod tests;
