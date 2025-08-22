use std::{collections::HashMap, range::Range};

use crate::{
    CompileError,
    syntax_tree::{SyntaxKind, SyntaxNode, SyntaxTree},
};

#[derive(Debug)]
pub struct PathResolver {
    modules: Vec<Module>,

    identifier_pool: String,

    imports: Vec<u32>,

    exports: Vec<u32>,
}

impl PathResolver {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            identifier_pool: String::new(),
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn get_module(&self, index: u32) -> Option<&Module> {
        self.modules.get(index as usize)
    }

    pub fn build_module_shallow(
        &mut self,
        path: &str,
        syntax_tree: &SyntaxTree,
    ) -> Result<u32, CompileError> {
        let top_node = syntax_tree
            .top_node()
            .ok_or(CompileError::MissingSyntaxNode { id: 0 })?;
        let path_start = self.identifier_pool.len() as u32;

        self.identifier_pool.push_str(path);

        let path_end = self.identifier_pool.len() as u32;
        let module = Module {
            path: Range {
                start: path_start,
                end: path_end,
            },
            imports: Range { start: 0, end: 0 },
        };
        let module_index = self.modules.len() as u32;

        self.build_from_node_shallow(top_node, syntax_tree)?;
        self.modules.push(module);

        Ok(module_index)
    }

    fn build_from_node_shallow(
        &self,
        node: &SyntaxNode,
        syntax_tree: &SyntaxTree,
    ) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem | SyntaxKind::ModuleItem | SyntaxKind::BlockExpression => {
                let children_start = node.payload.0 as usize;
                let children_end = children_start + node.payload.1 as usize;
                let child_indexes = &syntax_tree.children[children_start..children_end];

                for index in child_indexes {
                    let child_node = syntax_tree
                        .get_node(*index)
                        .ok_or(CompileError::MissingSyntaxNode { id: *index })?;

                    self.build_from_node_shallow(child_node, syntax_tree)?;
                }
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Module {
    path: Range<u32>,
    imports: Range<u32>,
}
