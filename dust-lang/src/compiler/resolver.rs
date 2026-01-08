use crate::{
    compiler::{CompileContext, CompileError, ScopeId, TypeId},
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

pub struct Resolver<'a> {
    file_id: SourceFileId,

    source: &'a Source,

    context: &'a mut CompileContext,

    syntax: &'a Syntax,

    current_scope_id: ScopeId,
}

impl<'a> Resolver<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: &'a Source,
        context: &'a mut CompileContext,
        syntax_tree: &'a Syntax,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            file_id,
            source,
            context,
            syntax: syntax_tree,
            current_scope_id,
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

        self.visit_main_function_item(main_root)
    }
}

impl<'a> SyntaxVisitor for Resolver<'a> {
    type Output = TypeId;

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::Output, CompileError> {
        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;
        let last_child = children.len() - 1;
        let mut main_type_id = TypeId::NONE;

        for (index, child) in children.into_iter().enumerate() {
            let child_type_id = self.visit(child)?;

            if index == last_child {
                main_type_id = child_type_id;
            }
        }

        Ok(main_type_id)
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_item(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        _: SyntaxReader,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_let_statement(&mut self, node: SyntaxReader) -> Result<Self::Output, CompileError> {
        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.node.kind,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;
        let expression_statement = children.nth(1).ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.node.kind,
                child_index: 0,
            })?;
        let expression_type_id = self.visit_expression(expression)?;
        let declaration_id = self
            .context
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let declaration = self.context.get_declaration(*declaration_id).ok_or(
            CompileError::MissingDeclaration {
                declaration_id: *declaration_id,
            },
        )?;

        self.context
            .types
            .unify_types(declaration.type_id, expression_type_id);

        Ok(TypeId::NONE)
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_integer_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError> {
        Ok(TypeId::STRING)
    }

    fn visit_path_expression(&mut self, node: SyntaxReader) -> Result<Self::Output, CompileError> {
        let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSourceFile {
                file_id: self.file_id,
            },
        )?;
        let variable_name = source_file.source_code.get_span(node.span());
        let (_, declaration) = self
            .context
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, node.span()),
            })?;

        Ok(declaration.type_id)
    }

    fn visit_block_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_if_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_math_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_while_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_call_expression(&mut self, _: SyntaxReader) -> Result<Self::Output, CompileError> {
        todo!()
    }
}
