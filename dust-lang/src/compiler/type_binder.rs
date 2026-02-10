use smallvec::SmallVec;
use tracing::debug;

use crate::{
    compiler::{
        CompileError, Resolver, TypeId, TypeNode,
        error::InternalError,
        resolver::{DeclarationId, DeclarationMembers, TypeMembers},
    },
    source::{Position, SourceFileId},
    parser::syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    file_id: SourceFileId,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,
}

impl<'a> TypeBinder<'a> {
    pub fn new(file_id: SourceFileId, syntax: &'a Syntax, resolver: &'a mut Resolver) -> Self {
        Self {
            file_id,
            syntax,
            resolver,
        }
    }

    pub fn bind_main(mut self) -> Result<TypeId, CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxTree(
                SourceFileId::MAIN,
            )))?
            .root()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxNode(
                SyntaxId::ROOT,
            )))?;

        self.visit_main(main_root)
    }

    pub fn infer_type(&mut self, type_id: TypeId) -> Result<TypeId, CompileError> {
        if let TypeNode::Inferred {
            resolved: Some(resolved),
            ..
        } = self.resolver.get_type(type_id)?
        {
            self.infer_type(*resolved)
        } else {
            Ok(type_id)
        }
    }

    fn unify_types(
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

    fn unify_inferred_types(
        &mut self,
        left: TypeId,
        left_position: Option<Position>,
        right: TypeId,
        right_position: Position,
    ) -> Result<(), CompileError> {
        if left == right {
            return Ok(());
        }

        let left_type_node = *self.resolver.get_type(left)?;
        let right_type_node = *self.resolver.get_type(right)?;

        match (left_type_node, right_type_node) {
            (
                TypeNode::Inferred {
                    inferred_id: id,
                    resolved: None,
                },
                _,
            ) => {
                if let Some(node) = self.resolver.get_type_mut(left) {
                    *node = TypeNode::Inferred {
                        inferred_id: id,
                        resolved: Some(right),
                    };
                }

                Ok(())
            }
            (
                _,
                TypeNode::Inferred {
                    inferred_id: id,
                    resolved: None,
                },
            ) => {
                if let Some(node) = self.resolver.get_type_mut(right) {
                    *node = TypeNode::Inferred {
                        inferred_id: id,
                        resolved: Some(left),
                    };
                }

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
                    .resolver
                    .get_type_members(left_value_parameters)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_value_types = self
                    .resolver
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
                    .resolver
                    .get_declaration_members(left_fields)?
                    .iter()
                    .map(|declaration_id| {
                        self.resolver.get_declaration_type(declaration_id).copied()
                    })
                    .try_collect::<SmallVec<[TypeId; 8]>>()?;
                let right_field_types = self
                    .resolver
                    .get_declaration_members(right_fields)?
                    .iter()
                    .map(|declaration_id| {
                        self.resolver.get_declaration_type(declaration_id).copied()
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
}

impl SyntaxVisitor for TypeBinder<'_> {
    type MainOutput = TypeId;
    type ItemOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = TypeId;
    type TypeOutput = TypeId;
    type PathOutput = TypeId;

    fn visit_main(&mut self, node: SyntaxReader) -> Result<Self::MainOutput, CompileError> {
        debug!("Binding types for main function");

        let main_return_type_id = self.resolver.create_inferred_type();
        let main_function_type_id = self.resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: TypeMembers::default(),
            return_type_id: main_return_type_id,
        });
        let main_function_declaration_id = *self.resolver.get_declaration_binding(&node.id)?;

        self.resolver
            .set_type_binding(node.id, main_function_type_id);
        self.resolver
            .set_declaration_type(main_function_declaration_id, main_function_type_id);

        let children = node.multiple_children()?;
        let last_child = children.len() - 1;

        for (index, child) in children.enumerate() {
            let child_type = if child.is_item() {
                self.visit_item(child)?;

                TypeId::NONE
            } else if child.is_statement() {
                self.visit_statement(child)?;

                TypeId::NONE
            } else {
                self.visit_expression(child, ())?
            };

            if index == last_child {
                self.unify_types(
                    main_return_type_id,
                    Some(Position::new(self.file_id, node.span())),
                    child_type,
                    Position::new(self.file_id, child.span()),
                )?;
            }
        }

        Ok(main_return_type_id)
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding types for module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding types for function item");

        let function_expression = node.right_child()?;

        self.visit_function_expression(function_expression, ())?;

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding types for use item");

        todo!()
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        debug!("Binding types for struct item");

        let (struct_name, struct_fields_list) = node.binary_children()?;
        let struct_fields = struct_fields_list.multiple_children()?;

        let mut fields = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let (field_name, field_type) = field.binary_children()?;

            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_type_id = self.visit_type(field_type)?;

            self.resolver
                .set_declaration_type(field_declaration_id, field_type_id);
            fields.push(field_declaration_id);
        }

        let declaration_id = *self.resolver.get_declaration_binding(&struct_name.id)?;
        let fields = self.resolver.add_declaration_members(&fields);
        let struct_type = TypeNode::Struct {
            declaration_id,
            fields,
            generics: DeclarationMembers::default(),
        };
        let struct_type_id = self.resolver.add_type(struct_type);

        self.resolver
            .set_declaration_type(declaration_id, struct_type_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Binding types for expression statement");

        self.visit_expression(node.left_child()?, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Binding types for let statement");

        let mut children = node.multiple_children()?;
        let path = children.expect_next()?;
        let expression_statement = children.expect_next()?;
        let type_notation = children.next();
        let expression = expression_statement.left_child()?;

        let expression_type_id = self.visit_expression(expression, ())?;

        if let Some(type_notation) = type_notation {
            let explicit_type = self.visit_type(type_notation)?;

            self.unify_types(
                expression_type_id,
                Some(Position::new(self.file_id, expression.span())),
                explicit_type,
                Position::new(self.file_id, type_notation.span()),
            )?;
        }

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;

        self.resolver
            .set_type_binding(expression.id, expression_type_id);
        self.resolver.set_type_binding(node.id, TypeId::NONE);

        self.resolver
            .set_declaration_type(declaration_id, expression_type_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Binding types for binary assignment statement");

        let (path, expression) = node.binary_children()?;

        let path_type = {
            let raw = self.visit_path(path)?;

            self.infer_type(raw)?
        };
        let expression_type = {
            let raw = self.visit_expression(expression, ())?;

            self.infer_type(raw)?
        };

        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionAssignmentStatement
                if (path_type == TypeId::STRING && expression_type == TypeId::CHARACTER)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::STRING)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::CHARACTER)
        );

        let unified = self.unify_inferred_types(
            path_type,
            Some(Position::new(self.file_id, path.span())),
            expression_type,
            Position::new(self.file_id, expression.span()),
        );

        if unified.is_err() && is_character_concatenation {
            self.resolver.set_type_binding(path.id, TypeId::STRING);
            self.resolver
                .set_type_binding(expression.id, TypeId::CHARACTER);

            return Ok(());
        }

        unified?;

        self.resolver.set_type_binding(path.id, path_type);
        self.resolver
            .set_type_binding(expression.id, expression_type);

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Binding types for reassignment statement");

        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.left_child()?;

        let path_type = self.visit_path(path)?;
        let expression_type = self.visit_expression(expression, ())?;

        self.unify_types(
            path_type,
            Some(Position::new(self.file_id, path.span())),
            expression_type,
            Position::new(self.file_id, path.span()),
        )?;
        self.resolver.set_type_binding(path.id, path_type);
        self.resolver
            .set_type_binding(expression.id, expression_type);
        self.resolver.set_type_binding(node.id, TypeId::NONE);

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for boolean expression");

        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for byte expression");

        self.resolver.set_type_binding(node.id, TypeId::BYTE);

        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for character expression");

        self.resolver.set_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for float expression");

        self.resolver.set_type_binding(node.id, TypeId::FLOAT);

        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for integer expression");

        self.resolver.set_type_binding(node.id, TypeId::INTEGER);

        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for string expression");

        self.resolver.set_type_binding(node.id, TypeId::STRING);

        Ok(TypeId::STRING)
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for list expression");

        let children = node.multiple_children()?;

        let mut previous = None;

        for child in children {
            let child_type = self.visit_expression(child, ())?;

            if let Some((element_type, previous_span)) = previous {
                self.unify_types(
                    element_type,
                    Some(Position::new(self.file_id, previous_span)),
                    child_type,
                    Position::new(self.file_id, child.span()),
                )?;
            } else {
                previous = Some((child_type, child.span()));
            }
        }

        let element_type = if let Some((element_type, _)) = previous {
            element_type
        } else {
            self.resolver.create_inferred_type()
        };
        let list_type = self.resolver.add_type(TypeNode::List { element_type });

        self.resolver.set_type_binding(node.id, list_type);

        Ok(list_type)
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for index expression");

        let (list_expression, index_expression) = node.binary_children()?;

        let list_type_id = {
            let raw = self.visit_expression(list_expression, input)?;

            self.infer_type(raw)?
        };
        let index_type_id = {
            let raw = self.visit_expression(index_expression, input)?;

            self.infer_type(raw)?
        };

        if index_type_id != TypeId::INTEGER {
            return Err(CompileError::ExpectedIntegerIndex {
                found: index_type_id,
                position: Position::new(self.file_id, index_expression.span()),
            });
        }

        let list_type = *self.resolver.get_type(list_type_id)?;
        let element_type = match list_type {
            TypeNode::List { element_type } => {
                self.resolver.set_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(CompileError::CannotIndex {
                    type_id: list_type_id,
                    position: Position::new(self.file_id, list_expression.span()),
                });
            }
        };

        self.resolver.set_type_binding(node.id, element_type);
        self.resolver
            .set_type_binding(list_expression.id, list_type_id);

        Ok(element_type)
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for path expression");
        debug_assert_eq!(node.kind(), SyntaxKind::PathExpression);

        let declaration_id = self.resolver.get_declaration_binding(&node.id)?;
        let type_id = *self.resolver.get_declaration_type(declaration_id)?;

        self.resolver.set_type_binding(node.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for struct expression");

        let fields = node.right_child()?.multiple_children()?;

        let declaration_id = *self.resolver.get_declaration_binding(&node.id)?;
        let declared_struct_type_id = *self.resolver.get_declaration_type(&declaration_id)?;

        for field in fields {
            let (field_name, field_expression) = field.binary_children()?;

            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let declared_field_type_id =
                *self.resolver.get_declaration_type(&field_declaration_id)?;
            let actual_field_type_id = self.visit_expression(field_expression, ())?;

            self.unify_types(
                declared_field_type_id,
                Some(Position::new(self.file_id, field_name.span())),
                actual_field_type_id,
                Position::new(self.file_id, field_expression.span()),
            )?;
            self.resolver
                .set_type_binding(field_expression.id, declared_field_type_id);
        }

        self.resolver
            .set_type_binding(node.id, declared_struct_type_id);

        Ok(declared_struct_type_id)
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for block expression");

        let children = node.multiple_children()?;

        let mut block_type_id = TypeId::NONE;

        for child in children {
            let child_type = if child.is_item() {
                self.visit_item(child)?;

                TypeId::NONE
            } else if child.is_statement() {
                self.visit_statement(child)?;

                TypeId::NONE
            } else {
                self.visit_expression(child, ())?
            };
            block_type_id = child_type;

            self.resolver.set_type_binding(child.id, child_type);
        }

        self.resolver.set_type_binding(node.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for if expression");

        let mut children = node.multiple_children()?;

        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                found: condition_type,
                node_kind: node.kind(),
                position: Position::new(self.file_id, condition.span()),
            });
        }

        let then_type = self.visit_block_expression(then_block, ())?;

        if let Some(else_block) = else_block {
            let else_type = self.visit_else_expression(else_block, ())?;

            self.unify_types(
                then_type,
                Some(then_block.position()),
                else_type,
                else_block.position(),
            )?;
        }

        self.resolver.set_type_binding(node.id, then_type);

        Ok(then_type)
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for else expression");

        self.visit_block_expression(node.left_child()?, ())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, ())?;

            self.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, ())?;

            self.infer_type(raw)?
        };

        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentStatement
                if (left_type == TypeId::STRING && right_type == TypeId::CHARACTER)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::STRING)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::CHARACTER)
        );

        let math_expression_type = if is_character_concatenation {
            TypeId::STRING
        } else {
            self.unify_inferred_types(
                left_type,
                Some(left_expression.position()),
                right_type,
                right_expression.position(),
            )?;

            left_type
        };

        self.resolver
            .set_type_binding(node.id, math_expression_type);

        Ok(math_expression_type)
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for comparison binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = self.visit_expression(left_expression, ())?;
        let right_type = self.visit_expression(right_expression, ())?;

        self.unify_types(
            left_type,
            Some(Position::new(self.file_id, left_expression.span())),
            right_type,
            Position::new(self.file_id, right_expression.span()),
        )?;
        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for logical binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, input)?;

            self.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, input)?;

            self.infer_type(raw)?
        };

        if left_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                found: left_type,
                node_kind: left_expression.kind(),
                position: Position::new(self.file_id, left_expression.span()),
            });
        }

        if right_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                found: right_type,
                node_kind: right_expression.kind(),
                position: Position::new(self.file_id, right_expression.span()),
            });
        }

        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for unary negation expression");

        let expression = node.left_child()?;
        let child_type = {
            let raw = self.visit_expression(expression, ())?;

            self.infer_type(raw)?
        };

        match child_type {
            TypeId::BOOLEAN | TypeId::BYTE | TypeId::FLOAT | TypeId::INTEGER => {
                self.resolver.set_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => Err(CompileError::CannotApplyOperator {
                operator: node.kind(),
                type_id: child_type,
                position: Position::new(self.file_id, expression.span()),
            }),
        }
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for while expression");

        let (condition, body) = node.binary_children()?;

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                found: condition_type,
                node_kind: condition.kind(),
                position: Position::new(self.file_id, condition.span()),
            });
        }

        self.visit_block_expression(body, ())?;

        Ok(TypeId::NONE)
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for function expression");

        let (signature, body) = node.binary_children()?;
        let value_parameters_list = signature.left_child()?;
        let return_type = if signature.has_right_child() {
            Some(signature.right_child()?)
        } else {
            None
        };
        let value_parameters = value_parameters_list.multiple_children()?;

        let mut value_parameter_types = SmallVec::<[TypeId; 8]>::new();

        for parameter_node in value_parameters {
            let parameter_name = parameter_node.left_child()?;
            let parameter_type = parameter_node.right_child()?;

            let parameter_declaration_id =
                *self.resolver.get_declaration_binding(&parameter_name.id)?;
            let parameter_type_id = self.visit_type(parameter_type)?;

            value_parameter_types.push(parameter_type_id);
            self.resolver
                .set_declaration_type(parameter_declaration_id, parameter_type_id);
        }

        let value_parameter_children = self.resolver.add_type_members(&value_parameter_types);
        let return_type_id = {
            if let Some(return_type_node) = return_type {
                let raw = self.visit_type(return_type_node)?;

                self.infer_type(raw)?
            } else {
                TypeId::NONE
            }
        };
        let function_type_id = self.resolver.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: value_parameter_children,
            return_type_id,
        });

        self.resolver.set_type_binding(node.id, function_type_id);
        self.resolver.set_type_binding(body.id, return_type_id);

        let function_id = self.resolver.get_declaration_binding(&node.id);

        if let Ok(function_declaration_id) = function_id {
            self.resolver
                .set_declaration_type(*function_declaration_id, function_type_id);
        }

        self.visit_block_expression(body, ())?;

        Ok(function_type_id)
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Binding types for call expression");

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.multiple_children()?;

        let callee_type = {
            let raw = self.visit_expression(callee, ())?;

            self.infer_type(raw)?
        };

        let TypeNode::Function {
            value_parameters,
            return_type_id,
            ..
        } = *self.resolver.get_type(callee_type)?
        else {
            return Err(CompileError::ExpectedFunctionType {
                found: callee_type,
                position: Position::new(self.file_id, callee.span()),
            });
        };

        let expected_parameters = self
            .resolver
            .get_type_members(value_parameters)?
            .iter()
            .copied()
            .collect::<SmallVec<[TypeId; 8]>>();

        if arguments.len() != expected_parameters.len() {
            return Err(CompileError::ExpectedArguments {
                function_type: callee_type,
                found_position: callee.position(),
                expected_count: expected_parameters.len(),
                found_count: arguments.len(),
            });
        }

        for (argument, expected_type_id) in arguments.zip(expected_parameters.into_iter()) {
            let argument_type = self.visit_expression(argument, ())?;

            self.unify_types(expected_type_id, None, argument_type, argument.position())?;
        }

        self.resolver.set_type_binding(node.id, return_type_id);

        Ok(return_type_id)
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<TypeId, CompileError> {
        match node.kind() {
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Ok(TypeId::BYTE),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::FloatType => Ok(TypeId::FLOAT),
            SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_node = node.left_child()?;
                let element_type_id = self.visit_type(element_type_node)?;
                let list_type_id = self.resolver.add_type(TypeNode::List {
                    element_type: element_type_id,
                });

                Ok(list_type_id)
            }
            SyntaxKind::FunctionType => {
                let type_node = {
                    let type_node_value_parameters = if node.has_left_child() {
                        let value_parameters = node.left_child()?.multiple_children()?;
                        let mut value_parameter_ids = SmallVec::<[TypeId; 4]>::new();

                        for value_parameter in value_parameters {
                            let type_id = if value_parameter.id == SyntaxId::NONE {
                                TypeId::NONE
                            } else {
                                self.visit_type(value_parameter)?
                            };

                            value_parameter_ids.push(type_id);
                        }

                        self.resolver.add_type_members(&value_parameter_ids)
                    } else {
                        TypeMembers::default()
                    };

                    let return_type_id = if node.has_right_child() {
                        self.visit_type(node.right_child()?)?
                    } else {
                        TypeId::NONE
                    };

                    TypeNode::Function {
                        type_parameters: DeclarationMembers::default(),
                        value_parameters: type_node_value_parameters,
                        return_type_id,
                    }
                };
                let function_type_id = self.resolver.add_type(type_node);

                Ok(function_type_id)
            }
            SyntaxKind::TypePath => {
                let declaration_id = self.resolver.get_declaration_binding(&node.id)?;
                let type_id = self.resolver.get_declaration_type(declaration_id)?;

                Ok(*type_id)
            }
            _ => Err(CompileError::Internal(InternalError::InvalidSyntaxNode(
                node.kind(),
            ))),
        }
    }

    fn visit_path(&mut self, _node: SyntaxReader) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }
}
