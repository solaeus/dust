use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdentifierStack {
    locals: Vec<Local>,
    scope_depth: usize,
}

impl IdentifierStack {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn with_data(locals: Vec<Local>, scope_depth: usize) -> Self {
        Self {
            locals,
            scope_depth,
        }
    }

    pub fn clear(&mut self) {
        self.locals.clear();
        self.scope_depth = 0;
    }

    pub fn local_count(&self) -> usize {
        self.locals.len()
    }

    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.locals
            .iter()
            .rev()
            .any(|local| &local.identifier == identifier)
    }

    pub fn get(&self, index: usize) -> Option<&Local> {
        self.locals.get(index)
    }

    pub fn get_index(&self, identifier: &Identifier) -> Option<u8> {
        self.locals
            .iter()
            .rev()
            .position(|local| &local.identifier == identifier)
            .map(|index| index as u8)
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;
    }

    pub fn declare(&mut self, identifier: Identifier) {
        self.locals.push(Local {
            identifier,
            depth: self.scope_depth,
        });
    }
}

impl Default for IdentifierStack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Local {
    pub identifier: Identifier,
    pub depth: usize,
}
