use smallvec::SmallVec;
use tracing::{Level, info, span};

use crate::{
    compiler::CompileError,
    native_function::NativeFunction,
    resolver::{Declaration, DeclarationId, DeclarationKind, Resolver, Scope, ScopeId, ScopeKind},
    source::{Position, Source, SourceFileId, Span},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    r#type::{FunctionType, Type},
};

pub struct Binder<'a> {
    file_id: SourceFileId,

    source: Source,

    resolver: &'a mut Resolver,

    syntax_tree: &'a SyntaxTree,

    current_scope: ScopeId,
}

impl<'a> Binder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: Source,
        resolver: &'a mut Resolver,
        syntax_tree: &'a SyntaxTree,
        current_scope: ScopeId,
    ) -> Self {
        Self {
            file_id,
            source,
            resolver,
            syntax_tree,
            current_scope,
        }
    }

    pub fn bind_native_functions(&mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind_native_functions");
        let _enter = span.enter();

        let read_line = NativeFunction::READ_LINE;
        let read_line_type_id = self.resolver.add_function_type(&read_line.r#type());
        let read_line_declaration = Declaration {
            kind: DeclarationKind::NativeFunction,
            scope_id: ScopeId::MAIN,
            type_id: read_line_type_id,
            position: Position::new(self.file_id, Span(0, 0)),
            is_public: true,
        };

        let write_line = NativeFunction::WRITE_LINE;
        let write_line_type_id = self.resolver.add_function_type(&write_line.r#type());
        let write_line_declaration = Declaration {
            kind: DeclarationKind::NativeFunction,
            scope_id: ScopeId::MAIN,
            type_id: write_line_type_id,
            position: Position::new(self.file_id, Span(0, 0)),
            is_public: true,
        };

        self.resolver
            .add_declaration("read_line", read_line_declaration);
        self.resolver
            .add_declaration("write_line", write_line_declaration);

        Ok(())
    }

    pub fn bind_main(mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind");
        let _enter = span.enter();

        let root_node =
            *self
                .syntax_tree
                .get_node(SyntaxId(0))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(0),
                })?;

        self.bind_item(&root_node)?;

        Ok(())
    }

    fn bind_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.bind_main_function_item(node)?,
            SyntaxKind::PublicFunctionItem => self.bind_function_item(node)?,
            SyntaxKind::PublicModuleItem => self.bind_module_item(node)?,
            SyntaxKind::UseItem => self.bind_use_item(node)?,
            _ => todo!(),
        }

        Ok(())
    }

    fn bind_main_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding main function");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let end_children = start_children + child_count;
        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_item() {
                self.bind_item(&child_node)?;
            }
        }

        Ok(())
    }

    fn bind_module_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding module");

        let (start_children, child_count) = (node.children.0, node.children.1);
        let children = self
            .syntax_tree
            .get_children(start_children, child_count)
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind,
                start_index: node.children.0,
                count: node.children.1,
            })?;

        for &child_id in children.iter() {
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;

            if child_node.kind.is_item() {
                self.bind_item(&child_node)?;
            }
        }

        Ok(())
    }

    fn bind_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding function item");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let function_expression_id = SyntaxId(node.children.1);
        let function_expression_node = *self.syntax_tree.get_node(function_expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_expression_id,
            },
        )?;

        let function_signature_id = SyntaxId(function_expression_node.children.0);
        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_signature_id,
            },
        )?;

        let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
        let value_parameter_list_node = *self.syntax_tree.get_node(value_parameter_list_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: value_parameter_list_id,
            },
        )?;

        let value_parameter_node_ids = self
            .syntax_tree
            .get_children(
                value_parameter_list_node.children.0,
                value_parameter_list_node.children.1,
            )
            .ok_or(CompileError::MissingChildren {
                parent_kind: value_parameter_list_node.kind,
                start_index: value_parameter_list_node.children.0,
                count: value_parameter_list_node.children.1,
            })?;
        let value_parameter_nodes = value_parameter_node_ids
            .iter()
            .map(|&id| {
                self.syntax_tree
                    .get_node(id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
            })
            .collect::<Result<SmallVec<[&SyntaxNode; 4]>, CompileError>>()?;

        let files = &self.source.read_files();
        let file = files
            .get(self.file_id.0 as usize)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?;

        let function_scope = self.resolver.add_scope(Scope {
            kind: ScopeKind::Function,
            parent: self.current_scope,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let mut value_parameters = Vec::new();
        let mut current_parameter_name = "";

        for (index, node) in value_parameter_nodes.iter().enumerate() {
            let is_name = index % 2 == 0;

            if is_name {
                current_parameter_name = unsafe {
                    str::from_utf8_unchecked(
                        &file.source_code.as_ref()[node.span.0 as usize..node.span.1 as usize],
                    )
                };
            } else {
                let r#type = match node.kind {
                    SyntaxKind::BooleanType => Type::Boolean,
                    SyntaxKind::ByteType => Type::Byte,
                    SyntaxKind::CharacterType => Type::Character,
                    SyntaxKind::FloatType => Type::Float,
                    SyntaxKind::IntegerType => Type::Integer,
                    SyntaxKind::StringType => Type::String,
                    _ => {
                        todo!()
                    }
                };
                let type_id = self.resolver.add_type(&r#type);
                let parameter_declaration = Declaration {
                    kind: DeclarationKind::Local { shadowed: None },
                    scope_id: function_scope,
                    type_id,
                    position: Position::new(self.file_id, node.span),
                    is_public: false,
                };

                self.resolver
                    .add_declaration(current_parameter_name, parameter_declaration);
                value_parameters.push(r#type);
            }
        }

        let function_return_type_id = SyntaxId(function_signature_node.children.1);
        let function_return_type_node = *self.syntax_tree.get_node(function_return_type_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_return_type_id,
            },
        )?;

        let return_type = match function_return_type_node.kind {
            SyntaxKind::BooleanType => Type::Boolean,
            SyntaxKind::ByteType => Type::Byte,
            SyntaxKind::CharacterType => Type::Character,
            SyntaxKind::FloatType => Type::Float,
            SyntaxKind::IntegerType => Type::Integer,
            SyntaxKind::StringType => Type::String,
            _ => {
                todo!()
            }
        };

        let function_type = FunctionType {
            type_parameters: Vec::new(),
            value_parameters,
            return_type,
        };
        let function_type_id = self.resolver.add_function_type(&function_type);

        let path_bytes = &files
            .get(self.file_id.0 as usize)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?
            .source_code
            .as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let path_str = unsafe { str::from_utf8_unchecked(path_bytes) };
        let declaration = Declaration {
            kind: DeclarationKind::Function {
                inner_scope_id: function_scope,
            },
            scope_id: self.current_scope,
            type_id: function_type_id,
            position: Position::new(self.file_id, node.span),
            is_public: true,
        };

        self.resolver.add_declaration(path_str, declaration);

        Ok(())
    }

    fn bind_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Binding use item");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let path_segment_ids = self
            .syntax_tree
            .get_children(path_node.children.0, path_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: path_node.kind,
                start_index: path_node.children.0,
                count: path_node.children.1,
            })?;
        let path_segment_nodes = path_segment_ids
            .iter()
            .map(|&id| {
                self.syntax_tree
                    .get_node(id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
            })
            .collect::<Result<SmallVec<[&SyntaxNode; 4]>, CompileError>>()?;

        let files = &self.source.read_files();

        let mut current_segment_index = 0;
        let mut current_scope_id = self.current_scope;
        let mut current_declaration_id = DeclarationId(0);

        loop {
            let segment_node = if current_segment_index < path_segment_nodes.len() {
                path_segment_nodes[current_segment_index]
            } else {
                break;
            };
            current_segment_index += 1;

            let segment_name_bytes = &files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?
                .source_code
                .as_ref()[segment_node.span.0 as usize..segment_node.span.1 as usize];
            let segment_name = unsafe { str::from_utf8_unchecked(segment_name_bytes) };

            let declarations = self.resolver.find_declarations(segment_name).ok_or(
                CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment_node.span),
                },
            )?;
            let referenced_declaration_info = declarations
                .iter()
                .find(|(_, declaration)| declaration.scope_id == current_scope_id);

            let (declaration_id, _) = match referenced_declaration_info {
                Some((id, declaration)) => {
                    if let DeclarationKind::Module { inner_scope_id, .. } = &declaration.kind {
                        current_scope_id = *inner_scope_id;
                    }

                    (id, declaration)
                }
                None => {
                    return Err(CompileError::UndeclaredVariable {
                        name: segment_node.to_string(),
                        position: Position::new(self.file_id, path_node.span),
                    });
                }
            };
            current_declaration_id = *declaration_id;

            if current_segment_index == path_segment_nodes.len() - 1 {
                drop(path_segment_nodes);

                break;
            }
        }

        self.resolver
            .get_scope_mut(self.current_scope)
            .ok_or(CompileError::MissingScope {
                scope_id: self.current_scope,
            })?
            .imports
            .push(current_declaration_id);

        Ok(())
    }
}
