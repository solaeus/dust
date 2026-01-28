use crate::{
    compiler::{CompileContext, CompileError, TypeId, TypeNode},
    source::{Position, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
    r#type::Type,
};

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
        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let last_child = children.len() - 1;
        let mut main_type_id = TypeId::NONE;

        for (index, child) in children.into_iter().enumerate() {
            let child_type = self.visit(child, ())?;

            if index == last_child {
                main_type_id = child_type;
            }

            self.context.add_type_binding(child.id, child_type);
        }

        Ok(main_type_id)
    }

    fn visit_module_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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
        let expression_type_id = self.visit_expression(expression, ())?;
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

        Ok(TypeId::NONE)
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;
        // let expression = expression_statement
        //     .left_child()
        //     .ok_or(CompileError::MissingChild {
        //         parent_kind: expression_statement.kind(),
        //         child_index: 0,
        //     })?;

        let expression_type_id = self.visit_expression(expression, input)?;
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

        self.context.add_type_binding(path.id, declaration.type_id);
        self.context
            .add_type_binding(expression.id, expression_type_id);

        Ok(TypeId::NONE)
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_boolean_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::STRING)
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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
        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let mut block_type_id = TypeId::NONE;

        for child in children {
            let child_type = self.visit(child, ())?;
            block_type_id = child_type;

            self.context.add_type_binding(child.id, child_type);
        }

        self.context.add_type_binding(node.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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

        let left_type_inferred = self.context.types.infer_type(left_type);
        let right_type_inferred = self.context.types.infer_type(right_type);

        let unified = self
            .context
            .types
            .unify_types(left_type_inferred, right_type_inferred)?;

        if !unified {
            let expected = self.context.types.get_full_type(left_type_inferred).ok_or(
                CompileError::MissingType {
                    type_id: left_type_inferred,
                },
            )?;
            let found = self
                .context
                .types
                .get_full_type(right_type_inferred)
                .ok_or(CompileError::MissingType {
                    type_id: right_type_inferred,
                })?;

            return Err(CompileError::TypeConflict {
                expected,
                found,
                position: Position::new(self.file_id, right_child.span()),
            });
        }

        let math_expression_type = match (node.kind(), left_type_inferred, right_type_inferred) {
            (SyntaxKind::AdditionExpression, TypeId::BYTE, TypeId::BYTE) => TypeId::BYTE,
            (SyntaxKind::AdditionExpression, TypeId::FLOAT, TypeId::FLOAT) => TypeId::FLOAT,
            (SyntaxKind::AdditionExpression, TypeId::INTEGER, TypeId::INTEGER) => TypeId::INTEGER,
            (
                SyntaxKind::AdditionExpression,
                TypeId::CHARACTER | TypeId::STRING,
                TypeId::CHARACTER | TypeId::STRING,
            ) => TypeId::STRING,
            (_, left, right) if left == right => left,
            _ => {
                let expected = self.context.types.get_full_type(left_type_inferred).ok_or(
                    CompileError::MissingType {
                        type_id: left_type_inferred,
                    },
                )?;
                let found = self
                    .context
                    .types
                    .get_full_type(right_type_inferred)
                    .ok_or(CompileError::MissingType {
                        type_id: right_type_inferred,
                    })?;

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

        let left_type = self.context.types.infer_type(left_type);
        let right_type = self.context.types.infer_type(right_type);

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
        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let left_type = self.visit(left_child, input)?;
        let right_type = self.visit(right_child, input)?;

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
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let child_type = self.visit(child, input)?;

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

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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

        Ok(list_type)
    }

    fn visit_while_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_call_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }
}
