use std::collections::HashMap;

use crate::syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: Self = Self(u32::MAX);
    pub const BOOLEAN: Self = Self(u32::MAX - 1);
    pub const BYTE: Self = Self(u32::MAX - 2);
    pub const CHARACTER: Self = Self(u32::MAX - 3);
    pub const FLOAT: Self = Self(u32::MAX - 4);
    pub const INTEGER: Self = Self(u32::MAX - 5);
    pub const STRING: Self = Self(u32::MAX - 6);
}

#[derive(Debug)]
pub struct TypeResolver {
    types: Vec<TypeId>,

    nodes: Vec<TypeNode>,

    children: Vec<TypeId>,

    interner: HashMap<u64, u32>,
}

impl TypeResolver {
    pub fn new(syntax_tree: &SyntaxTree) -> Self {
        let mut type_table = Self {
            types: vec![TypeId::NONE; syntax_tree.nodes.len()],
            nodes: Vec::new(),
            children: Vec::new(),
            interner: HashMap::new(),
        };

        type_table.fill_types(0, syntax_tree);

        type_table
    }

    pub fn get_type(&self, syntax_node: SyntaxId) -> Option<TypeId> {
        self.types.get(syntax_node.0 as usize).copied()
    }

    fn fill_types(&mut self, node_index: u32, syntax_tree: &SyntaxTree) -> Result<(), ()> {
        let node_index = node_index as usize;
        let node = syntax_tree.nodes.get(node_index).unwrap(); // TODO: Create errors
        let node_type = match node.kind {
            SyntaxKind::LetStatement => {
                let (first_child, child_count) = node.payload;

                if child_count == 2 {
                    let expression_id = SyntaxId(first_child + 1);
                    let expression = syntax_tree.get_node(expression_id).unwrap(); // TODO: Create errors

                    self.resolve_evaluation_type(expression, syntax_tree)
                } else {
                    let type_syntax_id = SyntaxId(first_child + 1);
                    let explicit_type = self.resolve_defined_type(type_syntax_id, syntax_tree);

                    let expression_id = SyntaxId(first_child + 2);
                    let expression = syntax_tree.get_node(expression_id).unwrap(); // TODO: Create errors
                    let inferred_type = self.resolve_evaluation_type(expression, syntax_tree);

                    if explicit_type != inferred_type {
                        todo!("Emit an error here.");
                    }

                    explicit_type
                }
            }
            _ => None,
        }
        .ok_or(())?;

        self.types[node_index] = node_type;

        Ok(())
    }

    fn resolve_evaluation_type(
        &self,
        node: &SyntaxNode,
        syntax_tree: &SyntaxTree,
    ) -> Option<TypeId> {
        match node.kind {
            SyntaxKind::MainFunctionItem => {
                let children_start = node.payload.0 as usize;
                let child_count = node.payload.1 as usize;

                if child_count == 0 {
                    return Some(TypeId::NONE);
                }

                let last_child_index = children_start + child_count - 1;
                let last_child = syntax_tree.nodes.get(last_child_index).unwrap(); // TODO: Create errors

                self.resolve_evaluation_type(last_child, syntax_tree)
            }
            SyntaxKind::BooleanExpression => Some(TypeId::BOOLEAN),
            SyntaxKind::ByteExpression => Some(TypeId::BYTE),
            SyntaxKind::CharacterExpression => Some(TypeId::CHARACTER),
            SyntaxKind::FloatExpression => Some(TypeId::FLOAT),
            SyntaxKind::IntegerExpression => Some(TypeId::INTEGER),
            SyntaxKind::StringExpression => Some(TypeId::STRING),
            SyntaxKind::GroupedExpression => {
                let expression_id = SyntaxId(node.payload.0);
                let expression = syntax_tree.get_node(expression_id).unwrap(); // TODO: Create errors

                self.resolve_evaluation_type(expression, syntax_tree)
            }
            _ => todo!(),
        }
    }

    fn resolve_defined_type(&self, node_id: SyntaxId, syntax_tree: &SyntaxTree) -> Option<TypeId> {
        let node = syntax_tree.get_node(node_id).unwrap(); // TODO: Create errors

        match node.kind {
            SyntaxKind::BooleanType => Some(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Some(TypeId::BYTE),
            SyntaxKind::CharacterType => Some(TypeId::CHARACTER),
            SyntaxKind::FloatType => Some(TypeId::FLOAT),
            SyntaxKind::IntegerType => Some(TypeId::INTEGER),
            SyntaxKind::StringType => Some(TypeId::STRING),
            SyntaxKind::TypePath => self.get_type(node_id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
enum TypeNode {
    BooleanArray(usize),
    ByteArray(usize),
    CharacterArray(usize),
    FloatArray(usize),
    IntegerArray(usize),
    StringArray(usize),
    CustomArray(u32, usize),
    CustomList(u32),
    Function {
        type_parameters: (u32, u32),
        value_parameters: (u32, u32),
        return_type: u32,
    },
}
