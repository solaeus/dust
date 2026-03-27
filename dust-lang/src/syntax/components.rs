use std::iter::ArrayChunks;

use tracing::debug;

use crate::syntax::{
    error::SyntaxError,
    node::SyntaxKind,
    reader::{SyntaxReader, SyntaxReaderIterator},
};

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
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ModuleItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier,
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
        debug_assert!(matches!(reader.node.kind, SyntaxKind::UseItem));

        Ok(Self {
            public: reader.node.modifier,
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
        debug_assert!(matches!(reader.node.kind, SyntaxKind::FunctionItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier,
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
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function parameters");
        debug_assert!(reader.node.kind == SyntaxKind::FunctionParameters);

        let mut children = reader.children();

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
        debug_assert!(reader.node.kind == SyntaxKind::StructItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier,
            name: children.expect_next()?,
            fields: children.expect_next()?,
            type_parameters: children.next(),
        })
    }
}

pub struct StructItemStructFields<'a> {
    pub name_type_pairs: ArrayChunks<SyntaxReaderIterator<'a>, 2>,
}

impl<'a> SyntaxComponent<'a> for StructItemStructFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct item struct fields");
        debug_assert!(reader.node.kind == SyntaxKind::StructItemStructFields);

        Ok(Self {
            name_type_pairs: reader.children().array_chunks(),
        })
    }
}

pub struct StructItemTupleFields<'a> {
    pub types: SyntaxReaderIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for StructItemTupleFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct item tuple fields");
        debug_assert!(reader.node.kind == SyntaxKind::StructItemTupleFields);

        Ok(Self {
            types: reader.children(),
        })
    }
}

pub struct EnumItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub variants: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting enum item");
        debug_assert!(reader.node.kind == SyntaxKind::EnumItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier,
            name: children.expect_next()?,
            variants: children.expect_next()?,
            type_parameters: children.next(),
        })
    }
}

pub struct EnumVariant<'a> {
    pub name: SyntaxReader<'a>,
    pub fields: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for EnumVariant<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting enum variant");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::EnumUnitVariant
                | SyntaxKind::EnumStructVariant
                | SyntaxKind::EnumTupleVariant
        ));

        match reader.node.kind {
            SyntaxKind::EnumUnitVariant => Ok(Self {
                name: *reader,
                fields: None,
            }),
            SyntaxKind::EnumStructVariant | SyntaxKind::EnumTupleVariant => {
                let mut children = reader.children();

                Ok(Self {
                    name: children.expect_next()?,
                    fields: Some(children.expect_next()?),
                })
            }
            _ => unreachable!(),
        }
    }
}

pub struct LetStatement<'a> {
    pub mutable: bool,
    pub name: SyntaxReader<'a>,
    pub expression: SyntaxReader<'a>,
    pub type_notation: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for LetStatement<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting let statement");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::LetStatement));

        let mut children = reader.children();

        Ok(Self {
            mutable: reader.node.modifier,
            name: children.expect_next()?,
            expression: children.expect_next()?,
            type_notation: children.next(),
        })
    }
}

pub struct ExpressionStatement<'a> {
    pub expression: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ExpressionStatement<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting expression statement");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ExpressionStatement));

        Ok(Self {
            expression: reader.single_child()?,
        })
    }
}

pub struct AssignmentExpression<'a> {
    pub target: SyntaxReader<'a>,
    pub value: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for AssignmentExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting assignment expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::AssignmentExpression));

        let (target, value) = reader.binary_children()?;

        Ok(Self { target, value })
    }
}

pub struct CompoundAssignmentExpression<'a> {
    pub target: SyntaxReader<'a>,
    pub value: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for CompoundAssignmentExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting compound assignment expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::AdditionAssignmentExpression
                | SyntaxKind::SubtractionAssignmentExpression
                | SyntaxKind::MultiplicationAssignmentExpression
                | SyntaxKind::DivisionAssignmentExpression
                | SyntaxKind::ModuloAssignmentExpression
                | SyntaxKind::ExponentAssignmentExpression
        ));

        let (target, value) = reader.binary_children()?;

        Ok(Self { target, value })
    }
}

pub struct MathExpression<'a> {
    pub left: SyntaxReader<'a>,
    pub right: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for MathExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting math expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::AdditionExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::ModuloExpression
                | SyntaxKind::ExponentExpression
        ));

        let (left, right) = reader.binary_children()?;

        Ok(Self { left, right })
    }
}

pub struct ComparisonExpression<'a> {
    pub left: SyntaxReader<'a>,
    pub right: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ComparisonExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting comparison expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::EqualExpression
                | SyntaxKind::NotEqualExpression
                | SyntaxKind::LessThanExpression
                | SyntaxKind::GreaterThanExpression
                | SyntaxKind::LessThanOrEqualExpression
                | SyntaxKind::GreaterThanOrEqualExpression
        ));

        let (left, right) = reader.binary_children()?;

        Ok(Self { left, right })
    }
}

pub struct LogicExpression<'a> {
    pub left: SyntaxReader<'a>,
    pub right: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for LogicExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting logic expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::AndExpression | SyntaxKind::OrExpression
        ));

        let (left, right) = reader.binary_children()?;

        Ok(Self { left, right })
    }
}

pub struct NegationExpression<'a> {
    pub operand: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for NegationExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting negation expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::NegationExpression));

        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct IndexExpression<'a> {
    pub list: SyntaxReader<'a>,
    pub index: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for IndexExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting index expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::IndexExpression));

        let (list, index) = reader.binary_children()?;

        Ok(Self { list, index })
    }
}

pub struct IfExpression<'a> {
    pub condition: SyntaxReader<'a>,
    pub then_branch: SyntaxReader<'a>,
    pub else_branch: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for IfExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting if expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::IfExpression));

        let mut children = reader.children();

        Ok(Self {
            condition: children.expect_next()?,
            then_branch: children.expect_next()?,
            else_branch: children.next(),
        })
    }
}

pub struct WhileExpression<'a> {
    pub condition: SyntaxReader<'a>,
    pub body: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for WhileExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting while expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::WhileExpression));

        let (condition, body) = reader.binary_children()?;

        Ok(Self { condition, body })
    }
}

pub struct CallExpression<'a> {
    pub callee: SyntaxReader<'a>,
    pub arguments: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for CallExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting call expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::CallExpression));

        let (callee, arguments) = reader.binary_children()?;

        Ok(Self { callee, arguments })
    }
}

pub struct StructExpression<'a> {
    pub path: SyntaxReader<'a>,
    pub fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for StructExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::StructExpression));

        let (path, fields) = reader.binary_children()?;

        Ok(Self { path, fields })
    }
}

pub struct StructExpressionStructFields<'a> {
    pub name_expression_pairs: ArrayChunks<SyntaxReaderIterator<'a>, 2>,
}

impl<'a> SyntaxComponent<'a> for StructExpressionStructFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct expression struct fields");
        debug_assert!(reader.node.kind == SyntaxKind::StructExpressionStructFields);

        Ok(Self {
            name_expression_pairs: reader.children().array_chunks(),
        })
    }
}

pub struct GroupedExpression<'a> {
    pub expression: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for GroupedExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting grouped expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::GroupedExpression));

        let expression = if reader.child_count() == 0 {
            None
        } else {
            Some(reader.single_child()?)
        };

        Ok(Self { expression })
    }
}

pub struct FunctionType<'a> {
    pub value_parameter_types: SyntaxReader<'a>,
    pub return_type: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for FunctionType<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert!(matches!(reader.node.kind, SyntaxKind::FunctionType));

        let mut children = reader.children();

        Ok(Self {
            value_parameter_types: children.expect_next()?,
            return_type: children.next(),
        })
    }
}
