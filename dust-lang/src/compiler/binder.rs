use smallvec::SmallVec;
use tracing::{Level, info, span};

use crate::{
    compiler::{
        CompileError,
        context::{CompileContext, Declaration, DeclarationKind, Scope, ScopeId, ScopeKind},
        type_graph::{TypeId, TypeNode},
    },
    source::{Position, Source, SourceFileId},
    syntax::{SyntaxId, SyntaxKind, SyntaxReader, SyntaxTree, SyntaxVisitor},
};

pub struct Binder<'a> {
    file_identifier: &'a str,

    file_id: SourceFileId,

    source: Source,

    context: &'a mut CompileContext,

    syntax_tree: &'a SyntaxTree,

    current_scope_id: ScopeId,
}

impl<'a> Binder<'a> {
    pub fn new(
        module_identifier: &'a str,
        file_id: SourceFileId,
        source: Source,
        context: &'a mut CompileContext,
        syntax_tree: &'a SyntaxTree,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            file_identifier: module_identifier,
            file_id,
            source,
            context,
            syntax_tree,
            current_scope_id,
        }
    }

    pub fn bind(mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind");
        let _enter = span.enter();

        let root = self
            .syntax_tree
            .root()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId::ROOT,
            })?;

        match root.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(root)?,
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => {
                self.visit_module_item(root)?
            }
            _ => {
                return Err(CompileError::InvalidSyntaxNode { kind: root.kind() });
            }
        }

        Ok(())
    }

    fn get_type_id(
        node: SyntaxReader<'_>,
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
                let lise_type_id = context.types.add_type(TypeNode::List { element_type_id });

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
    type Output = ();

    type Error = CompileError;

    fn visit_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node),
            SyntaxKind::ModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem => self.visit_function_item(node),
            SyntaxKind::UseItem => self.visit_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind(),
                position: Position::new(self.file_id, node.span()),
            }),
        }
    }

    fn visit_statement(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        match node.kind() {
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node)
            }
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind(),
                position: Position::new(self.file_id, node.span()),
            }),
        }
    }

    fn visit_expression(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        match node.kind() {
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node),
            SyntaxKind::BlockExpression => self.visit_block_expression(node),
            SyntaxKind::IfExpression => self.visit_if_expression(node),
            SyntaxKind::WhileExpression => self.visit_while_expression(node),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind(),
                position: Position::new(self.file_id, node.span()),
            }),
        }
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
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
            self.visit(child)?;
        }

        Ok(())
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_function_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_integer_expression(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn visit_path_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn visit_block_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_if_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_let_statement(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
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

        self.visit_expression(expression)?;

        let files = self.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
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

        self.context.add_declaration(variable_name, declaration);

        Ok(())
    }

    fn visit_math_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_while_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_function_expression(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_call_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}
