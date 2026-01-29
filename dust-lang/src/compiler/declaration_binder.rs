use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::{
        CompileError,
        context::{
            CompileContext, Declaration, DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind,
        },
        get_type_id,
        type_graph::TypeId,
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct DeclarationBinder<'a> {
    file_id: SourceFileId,

    source: &'a Source,

    syntax: &'a Syntax,

    context: &'a mut CompileContext,

    current_scope_id: ScopeId,
}

impl<'a> DeclarationBinder<'a> {
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
}

impl<'a> SyntaxVisitor for DeclarationBinder<'a> {
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
        debug!("Binding main function");

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
                start_index: node.inner().children.0,
                count: node.inner().children.1,
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
        debug!("Binding module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding function item");

        let function_name = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let function_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let _signature = function_expression
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: function_expression.kind(),
                child_index: 0,
            })?;
        let body = function_expression
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: function_expression.kind(),
                child_index: 1,
            })?;

        let outer_scope_id = self.current_scope_id;
        let function_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: outer_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        self.current_scope_id = function_scope_id;

        self.context.add_scope_binding(body.id, function_scope_id);
        self.visit(body, ())?;

        self.current_scope_id = outer_scope_id;

        let is_public = match node.kind() {
            SyntaxKind::PublicFunctionItem => true,
            SyntaxKind::FunctionItem => false,
            _ => unreachable!(),
        };
        let function_declaration = Declaration {
            kind: DeclarationKind::Function {
                inner_scope_id: function_scope_id,
                file_id: self.file_id,
                syntax_id: node.id,
                parameters: (0, 0),
                prototype_index: None,
            },
            scope_id: self.current_scope_id,
            type_id: TypeId::NONE,
            position: Position::new(self.file_id, function_name.span()),
            is_public,
        };

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let function_name_str = source_file.source_code.get(
            function_name.span().0 as usize,
            function_name.span().1 as usize,
        );
        let function_declaration_id = self
            .context
            .add_declaration(function_name_str, function_declaration);

        self.context
            .add_declaration_binding(function_expression.id, function_declaration_id);

        Ok(())
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding use item");

        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding expression statement");

        let expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding let statement");

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
            get_type_id(type_node, self.context)?
        } else {
            self.context.types.create_inferred_type()
        };
        let declaration = Declaration {
            kind: declaration_kind,
            scope_id: self.current_scope_id,
            type_id,
            position: Position::new(self.file_id, path.span()),
            is_public: false,
        };
        let declaration_id = self.context.add_declaration(variable_name, declaration);

        self.context
            .add_declaration_binding(path.id, declaration_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Binding binary assignment statement");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(expression, input)?;

        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file
            .source_code
            .get(path.span().0 as usize, path.span().1 as usize);

        let (declaration_id, _) = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path.span()),
            })?;

        self.context
            .add_declaration_binding(path.id, declaration_id);

        Ok(())
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
        Ok(())
    }

    fn visit_byte_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_character_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_float_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
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

    fn visit_list_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(())
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding index expression");

        let list = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let index = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(list, ())?;
        self.visit_expression(index, ())?;

        Ok(())
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding path expression");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let path_segments = path
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: path.kind(),
                start_index: path.inner().children.0,
                count: path.inner().children.1,
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
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding block expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let block_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Block,
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let parent_scope_id = self.current_scope_id;
        self.current_scope_id = block_scope_id;

        for child in children {
            self.visit(child, ())?;
        }

        self.current_scope_id = parent_scope_id;

        self.context.add_scope_binding(node.id, block_scope_id);

        Ok(())
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding if expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.inner().kind,
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        for child in children {
            self.visit(child, ())?;
        }

        Ok(())
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding else expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(child, ())?;

        Ok(())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding math binary expression");

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

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding comparison binary expression");

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

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding logical binary expression");

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

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding unary negation expression");

        let expression = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(expression, ())?;

        Ok(())
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding while expression");

        let condition = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        self.visit_expression(condition, ())?;
        self.visit(body, ())?;

        Ok(())
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Binding function expression");

        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let outer_scope_id = self.current_scope_id;
        let function_scope_id = self.context.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: outer_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });

        self.current_scope_id = function_scope_id;

        self.context.add_scope_binding(body.id, function_scope_id);
        self.visit(body, ())?;

        self.current_scope_id = outer_scope_id;

        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }
}
