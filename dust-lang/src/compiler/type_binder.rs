use smallvec::SmallVec;
use tracing::debug;

use crate::{
    compiler::{
        CompileError, Resolver, TypeId, TypeNode, error::InternalError, resolver::DeclarationId,
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
    r#type::Type,
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    file_id: SourceFileId,

    source: &'a Source,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: &'a Source,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
    ) -> Self {
        Self {
            file_id,
            syntax,
            source,
            resolver,
        }
    }

    pub fn bind_main(mut self) -> Result<TypeId, CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::Internal(InternalError::MissingSourceFile(
                SourceFileId::MAIN,
            )))?
            .root()
            .ok_or(CompileError::Internal(InternalError::MissingSyntaxNode(
                SyntaxId::ROOT,
            )))?;

        self.visit_main_function_item(main_root, ())
    }

    pub fn infer_type(
        &mut self,
        type_id: TypeId,
        position: Position,
    ) -> Result<TypeId, CompileError> {
        match self.resolver.get_type(type_id) {
            Some(TypeNode::Inferred {
                resolved: Some(resolved),
                ..
            }) => self.infer_type(*resolved, position),
            Some(TypeNode::Inferred { resolved: None, .. }) => {
                Err(CompileError::CannotInferType { position })
            }
            Some(_) => Ok(type_id),
            None => Err(CompileError::MissingType { type_id }),
        }
    }

    fn unify_types(
        &mut self,
        left: TypeId,
        left_position: Position,
        right: TypeId,
        right_position: Position,
    ) -> Result<(), CompileError> {
        let left_inferred = self.infer_type(left, left_position)?;
        let right_inferred = self.infer_type(right, right_position)?;

        self.unify_inferred_types(left_inferred, left_position, right_inferred, right_position)
    }

    fn unify_inferred_types(
        &mut self,
        left: TypeId,
        left_position: Position,
        right: TypeId,
        right_position: Position,
    ) -> Result<(), CompileError> {
        fn create_error(
            binder: &TypeBinder,
            left: TypeId,
            right: TypeId,
            right_position: Position,
        ) -> CompileError {
            let expected = if let Some(r#type) = binder.resolver.get_full_type(left, binder.source)
            {
                r#type
            } else {
                return CompileError::MissingType { type_id: left };
            };
            let found = if let Some(r#type) = binder.resolver.get_full_type(right, binder.source) {
                r#type
            } else {
                return CompileError::MissingType { type_id: right };
            };

            CompileError::TypeConflict {
                expected,
                found,
                position: right_position,
            }
        }

        if left == right {
            return Ok(());
        }

        let left_type_node = *self
            .resolver
            .get_type(left)
            .ok_or(CompileError::MissingType { type_id: left })?;
        let right_type_node = *self
            .resolver
            .get_type(right)
            .ok_or(CompileError::MissingType { type_id: right })?;

        match (left_type_node, right_type_node) {
            (
                TypeNode::Inferred {
                    inferred_id: id,
                    resolved: None,
                },
                _,
            )
            | (
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
                    .get_type_members(left_value_parameters.0, left_value_parameters.1)
                    .ok_or(CompileError::MissingTypeMembers {
                        start_index: left_value_parameters.0,
                        count: left_value_parameters.1,
                    })?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_value_types = self
                    .resolver
                    .get_type_members(right_value_parameters.0, right_value_parameters.1)
                    .ok_or(CompileError::MissingTypeMembers {
                        start_index: right_value_parameters.0,
                        count: right_value_parameters.1,
                    })?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();

                for (left_value_type, right_value_type) in
                    left_value_types.iter().zip(right_value_types.iter())
                {
                    self.unify_types(
                        *left_value_type,
                        left_position,
                        *right_value_type,
                        right_position,
                    )?;
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
                    return Err(create_error(self, left, right, right_position));
                }

                let left_field_types = self
                    .resolver
                    .get_declaration_members(left_fields.0, left_fields.1)
                    .ok_or(CompileError::MissingDeclarationMembers {
                        declaration_id: left_declaration_id,
                    })?
                    .iter()
                    .map(|declaration_id| {
                        self.resolver
                            .get_declaration_type(declaration_id)
                            .copied()
                            .ok_or(CompileError::MissingDeclarationType {
                                declaration_id: *declaration_id,
                            })
                    })
                    .collect::<Result<SmallVec<[TypeId; 8]>, CompileError>>()?;
                let right_field_types = self
                    .resolver
                    .get_declaration_members(right_fields.0, right_fields.1)
                    .ok_or(CompileError::MissingDeclarationMembers {
                        declaration_id: right_declaration_id,
                    })?
                    .iter()
                    .map(|declaration_id| {
                        self.resolver
                            .get_declaration_type(declaration_id)
                            .copied()
                            .ok_or(CompileError::MissingDeclarationType {
                                declaration_id: *declaration_id,
                            })
                    })
                    .collect::<Result<SmallVec<[TypeId; 8]>, CompileError>>()?;

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
                    Err(create_error(self, left, right, right_position))
                }
            }
        }
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
        let return_type_id = self.resolver.create_inferred_type();
        let main_type_id = self.resolver.add_type(TypeNode::Function {
            type_parameters: (0, 0),
            value_parameters: (0, 0),
            return_type_id,
        });

        for (index, child) in children.into_iter().enumerate() {
            let child_type = self.visit(child, ())?;

            self.resolver.set_type_binding(child.id, child_type);

            if index == last_child {
                self.unify_types(
                    return_type_id,
                    Position::new(self.file_id, node.span()),
                    child_type,
                    Position::new(self.file_id, child.span()),
                )?;
            }
        }

        self.resolver.set_type_binding(node.id, return_type_id);

        Ok(main_type_id)
    }

    fn visit_module_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for function item");

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

    fn visit_struct_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for struct item");

        let struct_name = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let struct_fields = node
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: 1,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let mut fields = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let field_name = field.left_child().ok_or(CompileError::MissingChild {
                parent_kind: field.inner().kind,
                child_index: 0,
            })?;
            let field_type = field.right_child().ok_or(CompileError::MissingChild {
                parent_kind: field.inner().kind,
                child_index: 1,
            })?;

            let field_declaration_id = *self
                .resolver
                .get_declaration_binding(&field_name.id)
                .ok_or(CompileError::MissingDeclarationBinding {
                    syntax_id: field_name.id,
                })?;
            let field_type_id = self.visit_type(field_type, ())?;

            self.resolver
                .set_declaration_type(field_declaration_id, field_type_id);
            fields.push(field_declaration_id);
        }

        let declaration_id = *self
            .resolver
            .get_declaration_binding(&struct_name.id)
            .ok_or(CompileError::MissingDeclarationBinding {
                syntax_id: struct_name.id,
            })?;
        let fields = self.resolver.add_declaration_members(&fields);
        let struct_type = TypeNode::Struct {
            declaration_id,
            fields,
            generics: (0, 0),
        };
        let struct_type_id = self.resolver.add_type(struct_type);

        self.resolver
            .set_declaration_type(declaration_id, struct_type_id);

        Ok(TypeId::NONE)
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
        let type_notation = children.next();
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        let expression_type_id = self.visit(expression, ())?;

        if let Some(type_notation) = type_notation {
            let explicit_type = self.visit_type(type_notation, ())?;

            self.unify_types(
                expression_type_id,
                Position::new(self.file_id, expression.span()),
                explicit_type,
                Position::new(self.file_id, type_notation.span()),
            )?;
        }

        let declaration_id = *self
            .resolver
            .get_declaration_binding(&path.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: path.id })?;

        self.resolver
            .set_type_binding(expression.id, expression_type_id);
        self.resolver.set_type_binding(node.id, TypeId::NONE);
        self.resolver
            .set_declaration_type(declaration_id, expression_type_id);

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

            self.infer_type(raw, Position::new(self.file_id, path.span()))?
        };
        let expression_type = {
            let raw = self.visit(expression, input)?;

            self.infer_type(raw, Position::new(self.file_id, path.span()))?
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
            Position::new(self.file_id, path.span()),
            expression_type,
            Position::new(self.file_id, expression.span()),
        );

        if unified.is_err() && is_character_concatenation {
            self.resolver.set_type_binding(path.id, TypeId::STRING);
            self.resolver
                .set_type_binding(expression.id, TypeId::CHARACTER);
            self.resolver.set_type_binding(node.id, TypeId::NONE);

            return Ok(TypeId::NONE);
        }

        unified?;

        self.resolver.set_type_binding(path.id, path_type);
        self.resolver
            .set_type_binding(expression.id, expression_type);
        self.resolver.set_type_binding(node.id, TypeId::NONE);

        Ok(TypeId::NONE)
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for reassignment statement");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 0,
        })?;
        let expression_statement = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        let path_type = self.visit(path, ())?;
        let expression_type = self.visit(expression, ())?;

        self.unify_types(
            path_type,
            Position::new(self.file_id, path.span()),
            expression_type,
            Position::new(self.file_id, path.span()),
        )?;
        self.resolver.set_type_binding(path.id, path_type);
        self.resolver
            .set_type_binding(expression.id, expression_type);
        self.resolver.set_type_binding(node.id, TypeId::NONE);

        Ok(TypeId::NONE)
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for boolean expression");

        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for byte expression");

        self.resolver.set_type_binding(node.id, TypeId::BYTE);

        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for character expression");

        self.resolver.set_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for float expression");

        self.resolver.set_type_binding(node.id, TypeId::FLOAT);

        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for integer expression");

        self.resolver.set_type_binding(node.id, TypeId::INTEGER);

        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for string expression");

        self.resolver.set_type_binding(node.id, TypeId::STRING);

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

        let mut previous = None;

        for child in children {
            if let Some((element_type, previous_span)) = previous {
                let child_type = self.visit(child, ())?;

                self.unify_types(
                    element_type,
                    Position::new(self.file_id, previous_span),
                    child_type,
                    Position::new(self.file_id, child.span()),
                )?;
            } else {
                let child_type = self.visit(child, ())?;

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
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for index expression");

        let list_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let index_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let list_type_id = {
            let raw = self.visit(list_expression, input)?;

            self.infer_type(raw, Position::new(self.file_id, list_expression.span()))?
        };
        let index_type_id = {
            let raw = self.visit(index_expression, input)?;

            self.infer_type(raw, Position::new(self.file_id, index_expression.span()))?
        };

        if index_type_id != TypeId::INTEGER {
            return Err(CompileError::ExpectedIntegerIndex {
                found: index_type_id,
                position: Position::new(self.file_id, index_expression.span()),
            });
        }

        let list_type = *self
            .resolver
            .get_type(list_type_id)
            .ok_or(CompileError::MissingType {
                type_id: list_type_id,
            })?;
        let element_type = match list_type {
            TypeNode::List { element_type } => {
                self.resolver.set_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(CompileError::CannotIndex {
                    r#type: self
                        .resolver
                        .get_full_type(list_type_id, self.source)
                        .ok_or(CompileError::MissingType {
                            type_id: list_type_id,
                        })?,
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
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for path expression");

        let declaration_id = *self
            .resolver
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let type_id = *self
            .resolver
            .get_declaration_type(&declaration_id)
            .ok_or(CompileError::MissingDeclarationType { declaration_id })?;

        self.resolver.set_type_binding(node.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for struct expression");

        let fields = node
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: node.inner().children.1,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let declaration_id = *self
            .resolver
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let declared_struct_type_id = *self
            .resolver
            .get_declaration_type(&declaration_id)
            .ok_or(CompileError::MissingDeclarationType { declaration_id })?;

        for field in fields {
            let field_name = field.left_child().ok_or(CompileError::MissingChild {
                parent_kind: field.inner().kind,
                child_index: 0,
            })?;
            let field_expression = field.right_child().ok_or(CompileError::MissingChild {
                parent_kind: field.inner().kind,
                child_index: 1,
            })?;

            let field_declaration_id = *self
                .resolver
                .get_declaration_binding(&field_name.id)
                .ok_or(CompileError::MissingDeclarationBinding {
                    syntax_id: field.id,
                })?;
            let field_declaration = self.resolver.get_declaration(field_declaration_id).ok_or(
                CompileError::MissingDeclaration {
                    declaration_id: field_declaration_id,
                },
            )?;
            let declared_field_type_id = *self
                .resolver
                .get_declaration_type(&field_declaration_id)
                .ok_or(CompileError::MissingDeclarationType {
                    declaration_id: field_declaration_id,
                })?;
            let actual_field_type_id = self.visit(field_expression, input)?;

            self.unify_types(
                declared_field_type_id,
                field_declaration.name,
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

                self.resolver.infer_type(raw)
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

            self.resolver.infer_type(raw)
        };

        if condition_type != TypeId::BOOLEAN {
            let found = self
                .resolver
                .get_full_type(condition_type, self.source)
                .ok_or(CompileError::MissingType {
                    type_id: condition_type,
                })?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, condition.span()),
            });
        }

        let then_type = self.visit(then_expression, ())?;

        if let Some(else_expression) = children.next() {
            let else_type = self.visit_else_expression(else_expression, ())?;

            let unified = self.resolver.unify_types(then_type, else_type)?;

            if !unified {
                let expected = self
                    .resolver
                    .get_full_type(then_type, self.source)
                    .ok_or(CompileError::MissingType { type_id: then_type })?;
                let found = self
                    .resolver
                    .get_full_type(else_type, self.source)
                    .ok_or(CompileError::MissingType { type_id: else_type })?;

                return Err(CompileError::TypeConflict {
                    expected,
                    found,
                    position: Position::new(self.file_id, else_expression.span()),
                });
            }

            self.resolver.set_type_binding(node.id, then_type);
        }

        self.resolver.set_type_binding(node.id, then_type);

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

        let left_type = {
            let raw = self.visit(left_child, ())?;

            self.resolver.infer_type(raw)
        };
        let right_type = {
            let raw = self.visit(right_child, ())?;

            self.resolver.infer_type(raw)
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
            let unified = self.resolver.unify_inferred_types(left_type, right_type)?;

            if unified {
                left_type
            } else {
                let expected = self
                    .resolver
                    .get_full_type(left_type, self.source)
                    .ok_or(CompileError::MissingType { type_id: left_type })?;
                let found = self.resolver.get_full_type(right_type, self.source).ok_or(
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

        self.resolver
            .set_type_binding(node.id, math_expression_type);

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

        let unified = self.resolver.unify_types(left_type, right_type)?;

        if !unified {
            let expected = self
                .resolver
                .get_full_type(left_type, self.source)
                .ok_or(CompileError::MissingType { type_id: left_type })?;
            let found = self.resolver.get_full_type(right_type, self.source).ok_or(
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

        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

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

            self.resolver.infer_type(raw)
        };
        let right_type = {
            let raw = self.visit(right_child, input)?;

            self.resolver.infer_type(raw)
        };

        if left_type != TypeId::BOOLEAN {
            let found = self
                .resolver
                .get_full_type(left_type, self.source)
                .ok_or(CompileError::MissingType { type_id: left_type })?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, left_child.span()),
            });
        }

        if right_type != TypeId::BOOLEAN {
            let found = self.resolver.get_full_type(right_type, self.source).ok_or(
                CompileError::MissingType {
                    type_id: right_type,
                },
            )?;

            return Err(CompileError::TypeConflict {
                expected: Type::Boolean,
                found,
                position: Position::new(self.file_id, right_child.span()),
            });
        }

        self.resolver.set_type_binding(node.id, TypeId::BOOLEAN);

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

            self.resolver.infer_type(raw)
        };

        match child_type {
            TypeId::BOOLEAN | TypeId::BYTE | TypeId::FLOAT | TypeId::INTEGER => {
                self.resolver.set_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => {
                let found = self.resolver.get_full_type(child_type, self.source).ok_or(
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

            self.resolver.infer_type(raw)
        };

        if condition_type != TypeId::BOOLEAN {
            let found = self
                .resolver
                .get_full_type(condition_type, self.source)
                .ok_or(CompileError::MissingType {
                    type_id: condition_type,
                })?;

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
        let value_parameters = signature
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: signature.inner().kind,
                child_index: 0,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: signature.inner().kind,
                start_index: signature.inner().children.0,
                count: signature.inner().children.1,
            })?;
        let return_type = signature.right_child();
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let mut value_parameter_types = Vec::new();

        for parameter_node in value_parameters {
            let parameter_name = parameter_node
                .left_child()
                .ok_or(CompileError::MissingChild {
                    parent_kind: parameter_node.inner().kind,
                    child_index: 0,
                })?;
            let parameter_type =
                parameter_node
                    .right_child()
                    .ok_or(CompileError::MissingChild {
                        parent_kind: parameter_node.inner().kind,
                        child_index: 1,
                    })?;

            let parameter_declaration_id = *self
                .resolver
                .get_declaration_binding(&parameter_name.id)
                .ok_or(CompileError::MissingDeclarationBinding {
                    syntax_id: parameter_name.id,
                })?;
            let parameter_type = self.visit_type(parameter_type, ())?;

            value_parameter_types.push(parameter_type);

            self.resolver
                .set_declaration_type(parameter_declaration_id, parameter_type);
        }

        let return_type_id = {
            let raw = if let Some(return_type_node) = return_type {
                self.visit_type(return_type_node, ())?
            } else {
                TypeId::NONE
            };

            self.resolver.infer_type(raw)
        };

        let value_parameter_children = self.resolver.add_type_members(&value_parameter_types);
        let function_type = self.resolver.add_type(TypeNode::Function {
            type_parameters: (0, 0),
            value_parameters: value_parameter_children,
            return_type_id,
        });

        self.resolver.set_type_binding(node.id, function_type);
        self.resolver.set_type_binding(body.id, return_type_id);

        let function_id = self.resolver.get_declaration_binding(&node.id).copied();

        if let Some(function_id) = function_id {
            self.resolver
                .set_declaration_type(function_id, function_type);
        }

        self.visit(body, ())?;

        Ok(function_type)
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding types for call expression");

        let callee = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let arguments = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let callee_type = {
            let raw = self.visit(callee, ())?;

            self.resolver.infer_type(raw)
        };

        let TypeNode::Function {
            value_parameters,
            return_type_id,
            ..
        } = *self
            .resolver
            .get_type(callee_type)
            .ok_or(CompileError::MissingType {
                type_id: callee_type,
            })?
        else {
            return Err(CompileError::ExpectedFunctionType {
                type_id: callee_type,
            });
        };

        let expected_parameters = self
            .resolver
            .get_type_members(value_parameters.0, value_parameters.1)
            .ok_or(CompileError::MissingTypeMembers {
                start_index: value_parameters.0,
                count: value_parameters.1,
            })?
            .to_vec();

        let argument_nodes =
            arguments
                .multiple_children()
                .ok_or(CompileError::MissingChildren {
                    parent_kind: arguments.kind(),
                    start_index: arguments.inner().children.0,
                    count: arguments.inner().children.1,
                })?;

        if argument_nodes.len() != expected_parameters.len() {
            return Err(CompileError::ExpectedFunctionType {
                type_id: callee_type,
            });
        }

        for (argument, expected_type) in argument_nodes.zip(expected_parameters.into_iter()) {
            let argument_type = {
                let raw = self.visit(argument, ())?;
                self.resolver.infer_type(raw)
            };

            let unified = self.resolver.unify_types(expected_type, argument_type)?;

            if !unified {
                let expected = self
                    .resolver
                    .get_full_type(expected_type, self.source)
                    .ok_or(CompileError::MissingType {
                        type_id: expected_type,
                    })?;
                let found = self
                    .resolver
                    .get_full_type(argument_type, self.source)
                    .ok_or(CompileError::MissingType {
                        type_id: argument_type,
                    })?;

                return Err(CompileError::TypeConflict {
                    expected,
                    found,
                    position: Position::new(self.file_id, argument.span()),
                });
            }
        }

        self.resolver.set_type_binding(node.id, return_type_id);

        Ok(return_type_id)
    }

    fn visit_type(&mut self, node: SyntaxReader, _: Self::Input) -> Result<TypeId, CompileError> {
        match node.kind() {
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Ok(TypeId::BYTE),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::FloatType => Ok(TypeId::FLOAT),
            SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_node = node.left_child().ok_or(CompileError::MissingChild {
                    parent_kind: node.kind(),
                    child_index: 0,
                })?;

                let element_type_id = self.visit_type(element_type_node, ())?;
                let list_type_id = self.resolver.add_type(TypeNode::List {
                    element_type: element_type_id,
                });

                Ok(list_type_id)
            }
            SyntaxKind::FunctionType => {
                let function_type_node = {
                    let type_node_value_parameters = if node.has_left_child() {
                        let function_value_parameters_node =
                            node.left_child().ok_or(CompileError::MissingChild {
                                parent_kind: node.kind(),
                                child_index: 0,
                            })?;

                        let value_parameters = function_value_parameters_node
                            .multiple_children()
                            .ok_or(CompileError::MissingChildren {
                            parent_kind: function_value_parameters_node.kind(),
                            start_index: function_value_parameters_node.inner().children.0,
                            count: function_value_parameters_node.inner().children.1,
                        })?;

                        let mut value_parameter_type_ids = SmallVec::<[TypeId; 4]>::new();

                        for value_parameter in value_parameters {
                            let type_id = if value_parameter.id == SyntaxId::NONE {
                                TypeId::NONE
                            } else {
                                self.visit_type(value_parameter, ())?
                            };

                            value_parameter_type_ids.push(type_id);
                        }

                        self.resolver.add_type_members(&value_parameter_type_ids)
                    } else {
                        (0, 0)
                    };

                    let return_type_id = if node.has_right_child() {
                        let function_return_type_node =
                            node.right_child().ok_or(CompileError::MissingChild {
                                parent_kind: node.kind(),
                                child_index: 1,
                            })?;

                        self.visit_type(function_return_type_node, ())?
                    } else {
                        TypeId::NONE
                    };

                    TypeNode::Function {
                        type_parameters: (0, 0),
                        value_parameters: type_node_value_parameters,
                        return_type_id,
                    }
                };
                let function_type_id = self.resolver.add_type(function_type_node);

                Ok(function_type_id)
            }
            SyntaxKind::TypePath => {
                let declaration_id = *self
                    .resolver
                    .get_declaration_binding(&node.id)
                    .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
                let type_id = self
                    .resolver
                    .get_declaration_type(&declaration_id)
                    .copied()
                    .unwrap_or_else(|| self.resolver.create_inferred_type());

                Ok(type_id)
            }
            _ => Err(CompileError::InvalidSyntaxNode { kind: node.kind() }),
        }
    }
}
