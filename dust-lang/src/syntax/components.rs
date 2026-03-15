use tracing::debug;

use crate::syntax::{error::SyntaxError, node::SyntaxKind, reader::SyntaxReader};

pub trait SyntaxComponent<'a>: Sized {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError>;
}

pub struct ModuleItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ModuleItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting module item");
        debug_assert!(matches!(reader.kind(), SyntaxKind::ModuleItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.modifier(),
            name: children.expect_next()?,
            body: children.next(),
        })
    }
}

pub struct UseItem<'a> {
    pub public: bool,
    pub path: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for UseItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting use item");
        debug_assert!(matches!(reader.kind(), SyntaxKind::UseItem));

        Ok(Self {
            public: reader.modifier(),
            path: reader.single_child()?,
        })
    }
}

pub struct FunctionItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub parameters: SyntaxReader<'a>,
    pub return_type: Option<SyntaxReader<'a>>,
    pub body: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for FunctionItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function item");
        debug_assert!(matches!(reader.kind(), SyntaxKind::FunctionItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.modifier(),
            name: children.expect_next()?,
            parameters: children.expect_next()?,
            body: children.expect_next()?,
            return_type: children.next(),
        })
    }
}

pub struct FunctionParameters<'a> {
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub value_parameters: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for FunctionParameters<'a> {
    fn from_reader(node: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function parameters");
        debug_assert!(node.kind() == SyntaxKind::FunctionParameters);

        let mut children = node.children();

        Ok(Self {
            value_parameters: children.expect_next()?,
            type_parameters: children.next(),
        })
    }
}

pub struct StructItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for StructItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct item");
        debug_assert!(reader.kind() == SyntaxKind::StructItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.modifier(),
            name: children.expect_next()?,
            fields: children.expect_next()?,
            type_parameters: children.next(),
        })
    }
}

pub struct StructField<'a> {
    pub public: bool,
    pub name: Option<SyntaxReader<'a>>,
    pub r#type: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for StructField<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct field");
        debug_assert!(reader.kind() == SyntaxKind::StructField);

        let (r#type, name) = reader.single_or_binary_children()?;

        Ok(Self {
            public: reader.modifier(),
            r#type,
            name,
        })
    }
}
