use smallvec::SmallVec;
use tracing::info;

use crate::{
    compiler::{
        CompileError,
        context::{
            CompileContext, Declaration, DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind,
        },
        type_graph::{TypeId, TypeNode},
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct Binder<'a> {
    file_id: SourceFileId,

    source: &'a Source,

    syntax: &'a Syntax,

    context: &'a mut CompileContext,

    current_scope_id: ScopeId,
}

impl<'a> Binder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: &'a Source,
        syntax: &'a Syntax,
        context: &'a mut CompileContext,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            file_id,
            source,
            syntax,
            context,
            current_scope_id,
        }
    }

    pub fn bind_main(mut self) -> Result<(), CompileError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(CompileError::MissingSyntaxTree {
                file_id: SourceFileId::MAIN,
            })?
            .root()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId::ROOT,
            })?;

        self.visit_main_function_item(main_root, ())
    }

    fn get_type_id(
        node: SyntaxReader,
        context: &mut CompileContext,
    ) -> Result<TypeId, CompileError> {
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

                let element_type_id = Self::get_type_id(element_type_node, context)?;
                let lise_type_id = context.types.add_type(TypeNode::List {
                    element_type: element_type_id,
                });

                Ok(lise_type_id)
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
                            start_index: function_value_parameters_node.node.children.0,
                            count: function_value_parameters_node.node.children.1,
                        })?;

                        let mut value_parameter_type_ids = SmallVec::<[TypeId; 4]>::new();

                        for value_parameter in value_parameters {
                            let type_id = if value_parameter.id == SyntaxId::NONE {
                                TypeId::NONE
                            } else {
                                Self::get_type_id(value_parameter, context)?
                            };

                            value_parameter_type_ids.push(type_id);
                        }

                        context.types.add_type_members(&value_parameter_type_ids)
                    } else {
                        (0, 0)
                    };

                    let return_type_id = if node.has_right_child() {
                        let function_return_type_node =
                            node.right_child().ok_or(CompileError::MissingChild {
                                parent_kind: node.kind(),
                                child_index: 1,
                            })?;

                        Self::get_type_id(function_return_type_node, context)?
                    } else {
                        TypeId::NONE
                    };

                    TypeNode::Function {
                        type_parameters: (0, 0),
                        value_parameters: type_node_value_parameters,
                        return_type_id,
                    }
                };
                let function_type_id = context.types.add_type(function_type_node);

                Ok(function_type_id)
            }
            _ => {
                todo!()
            }
        }
    }
}

impl<'a> SyntaxVisitor for Binder<'a> {
    type Input = ();

    type Output = ();

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding main function");

        self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: ScopeId::PROJECT,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: SyntaxKind::MainFunctionItem,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;

        for child in children {
            self.visit(child, ())?;
        }

        Ok(())
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
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding let statement");
        debug_assert!(matches!(
            node.node.kind,
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement
        ));

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.node.kind,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;
        let path = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
            child_index: 0,
        })?;
        let expression_statement = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
            child_index: 1,
        })?;
        let type_notation = children.next();
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.node.kind,
                child_index: 0,
            })?;

        self.visit_expression(expression, ())?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file
            .source_code
            .get(path.span().0 as usize, path.span().1 as usize);

        let shadowed = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .map(|(id, _)| id);
        let declaration_kind = if node.kind() == SyntaxKind::LetMutStatement {
            DeclarationKind::LocalMutable { shadowed }
        } else {
            DeclarationKind::Local { shadowed }
        };
        let type_id = if let Some(type_node) = type_notation {
            Self::get_type_id(type_node, self.context)?
        } else {
            self.context.types.create_inferred_type()
        };
        let declaration = Declaration {
            kind: declaration_kind,
            scope_id: self.current_scope_id,
            type_id,
            position: Position::new(self.file_id, node.node.span),
            is_public: false,
        };
        let declaration_id = self.context.add_declaration(variable_name, declaration);

        self.context
            .add_declaration_binding(node.id, declaration_id);

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let path_segments = path
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: path.kind(),
                start_index: path.node.children.0,
                count: path.node.children.1,
            })?;

        let mut current_declaration_id = DeclarationId(0);
        let mut current_type_id = TypeId::NONE;
        let mut current_scope_id = self.current_scope_id;

        for segment in path_segments {
            let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;
            let segment_name = source_file.source_code.get_span(segment.span());
            let (next_declaration_id, next_declaration) = self
                .context
                .find_declaration_in_scope(segment_name, current_scope_id)
                .ok_or(CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment.span()),
                })?;

            current_declaration_id = next_declaration_id;
            current_type_id = next_declaration.type_id;
            current_scope_id = next_declaration.scope_id;
        }

        self.context
            .add_declaration_binding(node.id, current_declaration_id);
        self.context.add_type_binding(node.id, current_type_id);

        Ok(())
    }

    fn visit_block_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_if_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let left_expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(left_expression, ())?;
        self.visit_expression(right_expression, ())?;

        Ok(())
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.node.kind,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;

        for child in children {
            self.visit_expression(child, ())?;
        }

        Ok(())
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
