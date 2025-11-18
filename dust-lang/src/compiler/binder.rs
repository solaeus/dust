use smallvec::SmallVec;
use tracing::{Level, info, span};

use crate::{
    compiler::CompileError,
    resolver::{
        Declaration, DeclarationId, DeclarationKind, ModuleKind, Resolver, Scope, ScopeId,
        ScopeKind, TypeId,
    },
    source::{Position, Source, SourceFileId, Span},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    r#type::{FunctionType, Type},
};

pub struct Binder<'a> {
    file_id: SourceFileId,

    source: Source,

    resolver: &'a mut Resolver,

    syntax_tree: &'a SyntaxTree,

    current_scope_id: ScopeId,
}

impl<'a> Binder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: Source,
        resolver: &'a mut Resolver,
        syntax_tree: &'a SyntaxTree,
        current_scope_id: ScopeId,
    ) -> Self {
        Self {
            file_id,
            source,
            resolver,
            syntax_tree,
            current_scope_id,
        }
    }

    pub fn bind_main(mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind_main");
        let _enter = span.enter();

        let root_node =
            *self
                .syntax_tree
                .get_node(SyntaxId(0))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(0),
                })?;

        self.bind_main_function_item(&root_node)?;

        Ok(())
    }

    pub fn bind_file_module(mut self, module_name: &str) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "bind_module");
        let _enter = span.enter();

        let declaration = Declaration {
            kind: DeclarationKind::Module {
                kind: ModuleKind::File,
                inner_scope_id: self.current_scope_id,
            },
            scope_id: ScopeId::MAIN,
            type_id: TypeId::NONE,
            position: Position::new(self.file_id, Span(0, 0)),
            is_public: true,
        };
        let root_node = *self
            .syntax_tree
            .top_node()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId(0),
            })?;
        let child_ids = self
            .syntax_tree
            .get_children(root_node.children.0, root_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: root_node.kind,
                start_index: root_node.children.0,
                count: root_node.children.1,
            })?;

        let mut current_child_index = 0;

        while current_child_index < child_ids.len() {
            let child_id = child_ids[current_child_index];
            let child_node =
                *self
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if child_node.kind.is_item() {
                self.bind_item(child_id, &child_node)?;
            }
        }

        self.resolver.add_declaration(module_name, declaration);
        self.bind_module_item(&root_node)?;

        Ok(())
    }

    fn bind_item(&mut self, node_id: SyntaxId, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.bind_main_function_item(node)?,
            SyntaxKind::FunctionItem => self.bind_function_item(node_id, node)?,
            SyntaxKind::PublicFunctionItem => self.bind_function_item(node_id, node)?,
            SyntaxKind::ModuleItem => self.bind_module_item(node)?,
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
                self.bind_item(child_id, &child_node)?;
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
                self.bind_item(child_id, &child_node)?;
            }
        }

        Ok(())
    }

    pub fn bind_function_item(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        info!("Binding function item");

        debug_assert!(matches!(
            node.kind,
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem
        ));

        let function_expression_id = SyntaxId(node.children.1);
        let function_expression_node = *self.syntax_tree.get_node(function_expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_expression_id,
            },
        )?;

        debug_assert_eq!(
            function_expression_node.kind,
            SyntaxKind::FunctionExpression
        );

        let function_signature_id = SyntaxId(function_expression_node.children.0);
        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_signature_id,
            },
        )?;

        debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

        let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
        let value_parameter_list_node = *self.syntax_tree.get_node(value_parameter_list_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: value_parameter_list_id,
            },
        )?;

        debug_assert_eq!(
            value_parameter_list_node.kind,
            SyntaxKind::FunctionValueParameters
        );

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
            parent: self.current_scope_id,
            imports: SmallVec::new(),
            modules: SmallVec::new(),
        });
        let mut value_parameters = Vec::new();
        let mut parameter_ids = SmallVec::<[DeclarationId; 4]>::new();

        for node in value_parameter_nodes {
            let parameter_name_node_id = node.children.0;
            let parameter_name_node = *self
                .syntax_tree
                .get_node(SyntaxId(parameter_name_node_id))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(parameter_name_node_id),
                })?;

            let parameter_name = unsafe {
                str::from_utf8_unchecked(
                    &file.source_code.as_ref()
                        [parameter_name_node.span.0 as usize..parameter_name_node.span.1 as usize],
                )
            };

            let parameter_type_node_id = node.children.1;
            let parameter_type_node = *self
                .syntax_tree
                .get_node(SyntaxId(parameter_type_node_id))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(parameter_type_node_id),
                })?;

            let r#type = match parameter_type_node.kind {
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
            let parameter_declaration = Declaration {
                kind: DeclarationKind::Local { shadowed: None },
                scope_id: function_scope,
                type_id: self.resolver.add_type(&r#type),
                position: Position::new(self.file_id, parameter_name_node.span),
                is_public: false,
            };

            value_parameters.push(r#type);

            let parameter_id = self
                .resolver
                .add_declaration(parameter_name, parameter_declaration);

            parameter_ids.push(parameter_id);
        }

        let function_return_type_id = SyntaxId(function_signature_node.children.1);
        let return_type = {
            if function_return_type_id == SyntaxId::NONE {
                Type::None
            } else {
                let function_return_type_node = *self
                    .syntax_tree
                    .get_node(function_return_type_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: function_return_type_id,
                    })?;

                match function_return_type_node.kind {
                    SyntaxKind::BooleanType => Type::Boolean,
                    SyntaxKind::ByteType => Type::Byte,
                    SyntaxKind::CharacterType => Type::Character,
                    SyntaxKind::FloatType => Type::Float,
                    SyntaxKind::IntegerType => Type::Integer,
                    SyntaxKind::StringType => Type::String,
                    _ => {
                        todo!()
                    }
                }
            }
        };

        let function_type = FunctionType {
            type_parameters: Vec::new(),
            value_parameters,
            return_type,
        };
        let function_type_id = self.resolver.add_function_type(&function_type);

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        debug_assert_eq!(path_node.kind, SyntaxKind::Path);

        let path_bytes = &files
            .get(self.file_id.0 as usize)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?
            .source_code
            .as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let path_str = unsafe { str::from_utf8_unchecked(path_bytes) };

        let parameter_children = self.resolver.add_parameters(&parameter_ids);
        let declaration = Declaration {
            kind: DeclarationKind::Function {
                inner_scope_id: function_scope,
                syntax_id: node_id,
                parameters: parameter_children,
            },
            scope_id: self.current_scope_id,
            type_id: function_type_id,
            position: Position::new(self.file_id, path_node.span),
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
        let mut current_scope_id = self.current_scope_id;
        let mut current_declaration_id;

        loop {
            let segment_node = path_segment_nodes[current_segment_index];

            let segment_name_bytes = &files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?
                .source_code
                .as_ref()[segment_node.span.0 as usize..segment_node.span.1 as usize];
            let segment_name = unsafe { str::from_utf8_unchecked(segment_name_bytes) };

            let (declaration_id, declaration) = self
                .resolver
                .find_declaration_in_scope(segment_name, current_scope_id)
                .ok_or(CompileError::UndeclaredVariable {
                    name: segment_name.to_string(),
                    position: Position::new(self.file_id, segment_node.span),
                })?;

            current_declaration_id = declaration_id;
            current_scope_id =
                if let DeclarationKind::Module { inner_scope_id, .. } = &declaration.kind {
                    *inner_scope_id
                } else {
                    declaration.scope_id
                };
            current_segment_index += 1;

            if current_segment_index == path_segment_nodes.len() {
                drop(path_segment_nodes);

                break;
            }
        }

        self.resolver
            .get_scope_mut(self.current_scope_id)
            .ok_or(CompileError::MissingScope {
                scope_id: self.current_scope_id,
            })?
            .imports
            .push(current_declaration_id);

        println!(
            "Importing module declaration ID {:?} into scope ID {:?}",
            current_declaration_id, self.current_scope_id
        );

        Ok(())
    }
}
