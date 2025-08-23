use std::hash::Hasher;

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::{
    CompileError,
    syntax_tree::{SyntaxKind, SyntaxNode, SyntaxTree},
};

#[derive(Debug)]
pub struct Program {
    modules: Vec<Module>,

    imports: Vec<ModuleId>,

    symbols: IndexMap<u64, Symbol, FxBuildHasher>,
}

impl Program {
    pub fn add_symbol(&mut self, identifier: &str, module: ModuleId) -> Symbol {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write(identifier.as_bytes());
            hasher.finish()
        };

        if let Some(existing) = self.symbols.get(&hash) {
            return *existing;
        }

        let id = SymbolId(self.symbols.len() as u32);

        self.symbols.insert(hash, Symbol { id, module });

        Symbol { id, module }
    }

    pub fn get_module(&self, id: ModuleId) -> Option<&Module> {
        self.modules.get(id.0 as usize)
    }

    pub fn build_module_shallow(
        &mut self,
        identifier: &str,
        syntax_tree: &SyntaxTree,
    ) -> Result<ModuleId, CompileError> {
        let top_node = syntax_tree.top_node().unwrap_or_else(|| todo!());
        let module_id = ModuleId(self.modules.len() as u32);
        let symbol = self.add_symbol(identifier, module_id);

        let module = Module {
            path: symbol,
            imports: (0, 0),
        };

        self.modules.push(module);
        self.build_from_node_shallow(top_node, syntax_tree)?;

        Ok(module_id)
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
            SyntaxKind::UseItem => {
                let (left_payload, right_payload) = node.payload;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Module {
    path: Symbol,
    imports: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    id: SymbolId,
    module: ModuleId,
}
