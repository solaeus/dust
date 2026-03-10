use tracing::debug;

use crate::syntax::{SyntaxError, SyntaxKind, SyntaxReader};

pub trait Component<'src>: Sized {
    fn from_reader(reader: &'src SyntaxReader<'src>) -> Result<Self, SyntaxError>;
}

pub struct Root<'src> {
    pub items: SyntaxReader<'src>,
}

pub struct ModuleItem<'src> {
    pub public: bool,
    pub name: SyntaxReader<'src>,
    pub body: Option<SyntaxReader<'src>>,
}

impl<'src> ModuleItem<'src> {
    pub fn new(node: &'src SyntaxReader<'src>) -> Result<Self, SyntaxError> {
        debug!("Visiting module item");
        debug_assert!(matches!(
            node.kind(),
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem
        ));

        let mut children = node.children()?;

        let public = node.kind() == SyntaxKind::PublicModuleItem;
        let name = children.expect_next()?;
        let body = children.next();

        Ok(Self { public, name, body })
    }
}
