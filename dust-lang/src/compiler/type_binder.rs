use tracing::debug;

use crate::{
    compiler::{CompileContext, CompileError, TypeId, TypeNode, get_type_id},
    source::{Position, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
    r#type::Type,
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    file_id: SourceFileId,

    syntax: &'a Syntax,

    context: &'a mut CompileContext,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        file_id: SourceFileId,
        context: &'a mut CompileContext,
        syntax_tree: &'a Syntax,
    ) -> Self {
        Self {
            file_id,
            context,
            syntax: syntax_tree,
        }
    }

    pub fn resolve_main(mut self) -> Result<TypeId, CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::MissingSourceFile {
                file_id: SourceFileId::MAIN,
            })?
            .root()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId::ROOT,
            })?;

        self.visit_main_function_item(main_root, ())
    }
}

impl<'a> SyntaxVisitor for TypeBinder<'a> {
    type Input = ();

    type Output = TypeId;

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for main function");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let last_child = children.len() - 1;
        let return_type_id = self.context.types.create_inferred_type();
        let main_type_id = self.context.types.add_type(TypeNode::Function {
            type_parameters: (0, 0),
            value_parameters: (0, 0),
            return_type_id,
        });

        for (index, child) in children.into_iter().enumerate() {
            let child_type = self.visit(child, ())?;

            self.context.add_type_binding(child.id, child_type);

            if index == last_child {
                let unified = self.context.types.unify_types(return_type_id, child_type)?;

                if !unified {
                    let expected = self.context.types.get_full_type(return_type_id).ok_or(
                        CompileError::MissingType {
                            type_id: return_type_id,
                        },
                    )?;
                    let found = self.context.types.get_full_type(child_type).ok_or(
                        CompileError::MissingType {
                            type_id: child_type,
                        },
                    )?;

                    return Err(CompileError::TypeConflict {
                        expected,
                        found,
                        position: Position::new(self.file_id, child.span()),
                    });
                }
            }
        }

        self.context.add_type_binding(node.id, return_type_id);

        Ok(main_type_id)
    }

    fn visit_module_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for module");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for function");

        let function_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        self.visit_function_expression(function_expression, ())?;

        Ok(TypeId::NONE)
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for use item");

        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for expression statement");

        let expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(expression, ())?;

        Ok(TypeId::NONE)
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for let statement");

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let path = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let expression_statement = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;
        let expression_type_id = self.visit(expression, ())?;
        let declaration_id = self
            .context
            .get_declaration_binding(&path.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let declaration = *self.context.get_declaration(*declaration_id).ok_or(
            CompileError::MissingDeclaration {
                declaration_id: *declaration_id,
            },
        )?;

        let unified = self
            .context
            .types
            .unify_types(declaration.type_id, expression_type_id)?;

        if !unified {
            let expected = self
                .context
                .types
                .get_full_type(declaration.type_id)
                .ok_or(CompileError::MissingType {
                    type_id: declaration.type_id,
                })?;
            let found = self.context.types.get_full_type(expression_type_id).ok_or(
                CompileError::MissingType {
                    type_id: expression_type_id,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected,
                found,
                position: Position::new(self.file_id, expression.span()),
            });
        }

        self.context
            .add_type_binding(expression.id, expression_type_id);
        self.context.add_type_binding(path.id, declaration.type_id);
        self.context.add_type_binding(node.id, TypeId::NONE);

        Ok(TypeId::NONE)
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for binary assignment statement");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let path_type = {
            let raw = self.visit(path, input)?;

            self.context.types.infer_type(raw)
        };
        let expression_type = {
            let raw = self.visit(expression, input)?;

            self.context.types.infer_type(raw)
        };

        let unified = self
            .context
            .types
            .unify_inferred_types(path_type, expression_type)?;
        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionAssignmentStatement
                if (path_type == TypeId::STRING && expression_type == TypeId::CHARACTER)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::STRING)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::CHARACTER)
        );

        if !unified && !is_character_concatenation {
            let expected = self
                .context
                .types
                .get_full_type(path_type)
                .ok_or(CompileError::MissingType { type_id: path_type })?;
            let found = self.context.types.get_full_type(expression_type).ok_or(
                CompileError::MissingType {
                    type_id: expression_type,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected,
                found,
                position: Position::new(self.file_id, expression.span()),
            });
        }

        self.context.add_type_binding(path.id, path_type);
        self.context
            .add_type_binding(expression.id, expression_type);
        self.context.add_type_binding(node.id, TypeId::NONE);

        Ok(TypeId::NONE)
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for reassignment statement");

        todo!()
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for boolean expression");

        self.context.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for byte expression");

        self.context.add_type_binding(node.id, TypeId::BYTE);

        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for character expression");

        self.context.add_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for float expression");

        self.context.add_type_binding(node.id, TypeId::FLOAT);

        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for integer expression");

        self.context.add_type_binding(node.id, TypeId::INTEGER);

        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for string expression");

        self.context.add_type_binding(node.id, TypeId::STRING);

        Ok(TypeId::STRING)
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for list expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let mut element_type = None;

        for child in children {
            if let Some(element_type) = element_type {
                let child_type = self.visit(child, ())?;
                let unified = self.context.types.unify_types(element_type, child_type);

                if !unified? {
                    let expected = self.context.types.get_full_type(element_type).ok_or(
                        CompileError::MissingType {
                            type_id: element_type,
                        },
                    )?;
                    let found = self.context.types.get_full_type(child_type).ok_or(
                        CompileError::MissingType {
                            type_id: child_type,
                        },
                    )?;

                    return Err(CompileError::TypeConflict {
                        expected,
                        found,
                        position: Position::new(self.file_id, child.span()),
                    });
                }
            } else {
                let child_type = self.visit(child, ())?;

                element_type = Some(child_type);
            }
        }

        let element_type = if let Some(element_type) = element_type {
            element_type
        } else {
            self.context.types.create_inferred_type()
        };
        let list_type = self.context.types.add_type(TypeNode::List { element_type });

        self.context.add_type_binding(node.id, list_type);

        Ok(list_type)
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for index expression");

        let list_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let index_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let list_type_id = {
            let raw = self.visit(list_expression, input)?;

            self.context.types.infer_type(raw)
        };
        let index_type_id = {
            let raw = self.visit(index_expression, input)?;

            self.context.types.infer_type(raw)
        };

        if index_type_id != TypeId::INTEGER {
            let found = self.context.types.get_full_type(index_type_id).ok_or(
                CompileError::MissingType {
                    type_id: index_type_id,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected: Type::Integer,
                found,
                position: Position::new(self.file_id, index_expression.span()),
            });
        }

        let list_type =
            *self
                .context
                .types
                .get_type(list_type_id)
                .ok_or(CompileError::MissingType {
                    type_id: list_type_id,
                })?;
        let element_type = match list_type {
            TypeNode::List { element_type } => {
                self.context.add_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(CompileError::CannotIndex {
                    r#type: self.context.types.get_full_type(list_type_id).ok_or(
                        CompileError::MissingType {
                            type_id: list_type_id,
                        },
                    )?,
                    position: Position::new(self.file_id, list_expression.span()),
                });
            }
        };

        self.context.add_type_binding(node.id, element_type);
        self.context
            .add_type_binding(list_expression.id, list_type_id);

        Ok(element_type)
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for path expression");

        let declaration_id = self
            .context
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let declaration = *self.context.get_declaration(*declaration_id).ok_or(
            CompileError::MissingDeclaration {
                declaration_id: *declaration_id,
            },
        )?;

        self.context.add_type_binding(node.id, declaration.type_id);

        Ok(declaration.type_id)
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for block expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let mut block_type_id = TypeId::NONE;

        for child in children {
            let child_type = {
                let raw = self.visit(child, ())?;

                self.context.types.infer_type(raw)
            };
            block_type_id = child_type;

            self.context.add_type_binding(child.id, child_type);
        }

        self.context.add_type_binding(node.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for if expression");

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let condition = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let then_expression = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let condition_type = {
            let raw = self.visit(condition, ())?;

            self.context.types.infer_type(raw)
        };

        if condition_type != TypeId::BOOLEAN {
            let found = self.context.types.get_full_type(condition_type).ok_or(
                CompileError::MissingType {
                    type_id: condition_type,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, condition.span()),
            });
        }

        let then_type = self.visit(then_expression, ())?;

        if let Some(else_expression) = children.next() {
            let else_type = self.visit_else_expression(else_expression, ())?;

            let unified = self.context.types.unify_types(then_type, else_type)?;

            if !unified {
                let expected = self
                    .context
                    .types
                    .get_full_type(then_type)
                    .ok_or(CompileError::MissingType { type_id: then_type })?;
                let found = self
                    .context
                    .types
                    .get_full_type(else_type)
                    .ok_or(CompileError::MissingType { type_id: else_type })?;

                return Err(CompileError::TypeConflict {
                    expected,
                    found,
                    position: Position::new(self.file_id, else_expression.span()),
                });
            }

            self.context.add_type_binding(node.id, then_type);
        }

        self.context.add_type_binding(node.id, then_type);

        Ok(then_type)
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for else expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;

        self.visit_expression(child, ())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for math binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let left_type = self.visit(left_child, ())?;
        let right_type = self.visit(right_child, ())?;

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
            let unified = self.context.types.unify_types(left_type, right_type)?;

            if unified {
                left_type
            } else {
                let expected = self
                    .context
                    .types
                    .get_full_type(left_type)
                    .ok_or(CompileError::MissingType { type_id: left_type })?;
                let found = self.context.types.get_full_type(right_type).ok_or(
                    CompileError::MissingType {
                        type_id: right_type,
                    },
                )?;

                return Err(CompileError::TypeConflict {
                    expected,
                    found,
                    position: Position::new(self.file_id, right_child.span()),
                });
            }
        };

        self.context.add_type_binding(node.id, math_expression_type);

        Ok(math_expression_type)
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for comparison binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let left_type = self.visit(left_child, ())?;
        let right_type = self.visit(right_child, ())?;

        let unified = self.context.types.unify_types(left_type, right_type)?;

        if !unified {
            let expected = self
                .context
                .types
                .get_full_type(left_type)
                .ok_or(CompileError::MissingType { type_id: left_type })?;
            let found =
                self.context
                    .types
                    .get_full_type(right_type)
                    .ok_or(CompileError::MissingType {
                        type_id: right_type,
                    })?;

            return Err(CompileError::TypeConflict {
                expected,
                found,
                position: Position::new(self.file_id, right_child.span()),
            });
        }

        self.context.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for logical binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let left_type = {
            let raw = self.visit(left_child, input)?;

            self.context.types.infer_type(raw)
        };
        let right_type = {
            let raw = self.visit(right_child, input)?;

            self.context.types.infer_type(raw)
        };

        if left_type != TypeId::BOOLEAN {
            let found = self
                .context
                .types
                .get_full_type(left_type)
                .ok_or(CompileError::MissingType { type_id: left_type })?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, left_child.span()),
            });
        }

        if right_type != TypeId::BOOLEAN {
            let found =
                self.context
                    .types
                    .get_full_type(right_type)
                    .ok_or(CompileError::MissingType {
                        type_id: right_type,
                    })?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, right_child.span()),
            });
        }

        self.context.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for unary negation expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let child_type = {
            let raw = self.visit(child, ())?;

            self.context.types.infer_type(raw)
        };

        match child_type {
            TypeId::BOOLEAN | TypeId::BYTE | TypeId::FLOAT | TypeId::INTEGER => {
                self.context.add_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => {
                let found = self.context.types.get_full_type(child_type).ok_or(
                    CompileError::MissingType {
                        type_id: child_type,
                    },
                )?;

                Err(CompileError::CannotApplyOperator {
                    operator: node.kind(),
                    r#type: found,
                    position: Position::new(self.file_id, child.span()),
                })
            }
        }
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for while expression");

        let condition = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let condition_type = {
            let raw = self.visit(condition, ())?;

            self.context.types.infer_type(raw)
        };

        if condition_type != TypeId::BOOLEAN {
            let found = self.context.types.get_full_type(condition_type).ok_or(
                CompileError::MissingType {
                    type_id: condition_type,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, condition.span()),
            });
        }

        self.visit(body, ())?;

        Ok(TypeId::NONE)
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for function expression");

        let signature = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let value_parameters_node = signature.left_child().ok_or(CompileError::MissingChild {
            parent_kind: signature.inner().kind,
            child_index: 0,
        })?;
        let value_parameter_nodes =
            value_parameters_node
                .multiple_children()
                .ok_or(CompileError::MissingChildren {
                    parent_kind: value_parameters_node.inner().kind,
                    start_index: value_parameters_node.inner().children.0,
                    count: value_parameters_node.inner().children.1,
                })?;
        let return_type = signature.right_child();
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let mut value_parameter_types = Vec::new();

        for parameter_node in value_parameter_nodes {
            let parameter_name = parameter_node
                .left_child()
                .ok_or(CompileError::MissingChild {
                    parent_kind: parameter_node.inner().kind,
                    child_index: 0,
                })?;
            let parameter_type_node =
                parameter_node
                    .right_child()
                    .ok_or(CompileError::MissingChild {
                        parent_kind: parameter_node.inner().kind,
                        child_index: 1,
                    })?;

            let parameter_type = get_type_id(parameter_type_node, self.context)?;

            value_parameter_types.push(parameter_type);
        }

        let return_type_id = {
            let raw = if let Some(return_type_node) = return_type {
                get_type_id(return_type_node, self.context)?
            } else {
                TypeId::NONE
            };

            self.context.types.infer_type(raw)
        };

        let value_parameter_children = self.context.types.add_type_members(&value_parameter_types);
        let function_type = self.context.types.add_type(TypeNode::Function {
            type_parameters: (0, 0),
            value_parameters: value_parameter_children,
            return_type_id,
        });

        self.context.add_type_binding(node.id, function_type);
        self.context.add_type_binding(body.id, return_type_id);

        self.visit(body, ())?;

        Ok(function_type)
    }

    fn visit_call_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for call expression");

        todo!()
    }
}
