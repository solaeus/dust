use tracing::debug;

use crate::syntax::{error::SyntaxError, node::SyntaxKind, reader::SyntaxReader};

pub struct ModuleItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> ModuleItem<'a> {
    pub fn new(node: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting module item");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem
        ));

        let mut children = node.children();

        let public = node.kind() == SyntaxKind::PublicModuleItem;
        let name = children.expect_next()?;
        let body = children.next();

        Ok(Self { public, name, body })
    }
}

pub struct FunctionItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub parameters: SyntaxReader<'a>,
    pub return_type: Option<SyntaxReader<'a>>,
    pub body: SyntaxReader<'a>,
}

impl<'a> FunctionItem<'a> {
    pub fn new(node: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function item");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem
        ));

        let mut children = node.children();

        let public = node.kind() == SyntaxKind::PublicFunctionItem;
        let name = children.expect_next()?;
        let parameters = children.expect_next()?;
        let body = children.expect_next()?;
        let return_type = children.next();

        Ok(Self {
            public,
            name,
            parameters,
            return_type,
            body,
        })
    }
}

pub struct FunctionParameters<'a> {
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub value_parameters: SyntaxReader<'a>,
}

impl<'a> FunctionParameters<'a> {
    pub fn new(node: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function parameters");
        debug_assert!(node.kind() == SyntaxKind::FunctionParameters);

        let mut children = node.children();

        let value_parameters = children.expect_next()?;
        let types_parameters = children.next();

        Ok(Self {
            value_parameters,
            type_parameters: types_parameters,
        })
    }
}
