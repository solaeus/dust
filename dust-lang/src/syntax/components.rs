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
            public: reader.node.modifier.is_public(),
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
            public: reader.node.modifier.is_public(),
            path: reader.single_child()?,
        })
    }
}

pub struct FunctionItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub signature: SyntaxReader<'a>,
    pub body: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for FunctionItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::FunctionItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name: children.expect_next()?,
            signature: children.expect_next()?,
            body: children.expect_next()?,
        })
    }
}

pub struct FunctionSignature<'a> {
    pub parameters: SyntaxReader<'a>,
    pub return_type: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for FunctionSignature<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting function signature");
        debug_assert!(reader.node.kind == SyntaxKind::FunctionSignature);

        let modifier = reader.node.modifier;
        let mut children = reader.children();

        let parameters = children.expect_next()?;
        let return_type = if modifier.has_return_type() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.has_where_clause() {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            parameters,
            return_type,
            where_clause,
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

pub struct ValueParameters<'a> {
    pub name_type_pairs: ArrayChunks<SyntaxReaderIterator<'a>, 2>,
}

impl<'a> SyntaxComponent<'a> for ValueParameters<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting value parameters");
        debug_assert!(reader.node.kind == SyntaxKind::ValueParameters);

        Ok(Self {
            name_type_pairs: reader.children().array_chunks(),
        })
    }
}

pub struct StructItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub fields: SyntaxReader<'a>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for StructItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting struct item");
        debug_assert!(reader.node.kind == SyntaxKind::StructItem);

        let mut children = reader.children();

        let name = children.expect_next()?;
        let fields = children.expect_next()?;

        let mut type_parameters = None;
        let mut where_clause = None;

        if let Some(child) = children.next() {
            if child.node.kind == SyntaxKind::WhereClause {
                where_clause = Some(child);
            } else {
                type_parameters = Some(child);

                if let Some(child) = children.next() {
                    where_clause = Some(child);
                }
            }
        }

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name,
            fields,
            type_parameters,
            where_clause,
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
            public: reader.node.modifier.is_public(),
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

        if reader.has_children() {
            let (name, fields) = reader.binary_children()?;

            Ok(Self {
                name,
                fields: Some(fields),
            })
        } else {
            Ok(Self {
                name: *reader,
                fields: None,
            })
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
            mutable: reader.node.modifier.is_public(),
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
    pub source: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for AssignmentExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting assignment expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::AssignmentExpression));

        let (target, value) = reader.binary_children()?;

        Ok(Self {
            target,
            source: value,
        })
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
                | SyntaxKind::AdditionAssignmentExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::SubtractionAssignmentExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::MultiplicationAssignmentExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::DivisionAssignmentExpression
                | SyntaxKind::ModuloExpression
                | SyntaxKind::ModuloAssignmentExpression
                | SyntaxKind::ExponentExpression
                | SyntaxKind::ExponentAssignmentExpression
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

pub struct NotExpression<'a> {
    pub operand: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for NotExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting not expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::NotExpression));

        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct ArrayExpression<'a> {
    pub elements: SyntaxReaderIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting array expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ArrayExpression));

        Ok(Self {
            elements: reader.children(),
        })
    }
}

pub struct ArrayRepeatExpression<'a> {
    pub element: SyntaxReader<'a>,
    pub length: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayRepeatExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting array repeat expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::ArrayRepeatExpression
        ));

        let (element, length) = reader.binary_children()?;

        Ok(Self { element, length })
    }
}

pub struct IndexExpression<'a> {
    pub collection: SyntaxReader<'a>,
    pub index: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for IndexExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting index expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::IndexExpression));

        let (collection, index) = reader.binary_children()?;

        Ok(Self { collection, index })
    }
}

pub struct RangeExpression<'a> {
    pub start: SyntaxReader<'a>,
    pub end: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for RangeExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting range expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression
        ));

        let (start, end) = reader.binary_children()?;

        Ok(Self { start, end })
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

pub struct ConstItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_annotation: SyntaxReader<'a>,
    pub value: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ConstItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting const item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ConstItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name: children.expect_next()?,
            type_annotation: children.expect_next()?,
            value: children.expect_next()?,
        })
    }
}

pub struct TypeItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub aliased_type: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for TypeItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting type item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TypeItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name: children.expect_next()?,
            aliased_type: children.expect_next()?,
            type_parameters: children.next(),
        })
    }
}

pub struct ImplItem<'a> {
    pub self_type: SyntaxReader<'a>,
    pub body: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub type_arguments: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ImplItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting impl item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ImplItem));

        let modifier = reader.node.modifier;
        let mut children = reader.children();

        let self_type = children.expect_next()?;
        let body = children.expect_next()?;
        let type_parameters = if modifier.has_type_parameters() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let type_arguments = if modifier.has_type_arguments() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.has_where_clause() {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            self_type,
            body,
            type_parameters,
            type_arguments,
            where_clause,
        })
    }
}

pub struct ImplTraitItem<'a> {
    pub self_type: SyntaxReader<'a>,
    pub body: SyntaxReader<'a>,
    pub trait_path: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub type_arguments: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ImplTraitItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting impl trait item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ImplTraitItem));

        let modifier = reader.node.modifier;
        let mut children = reader.children();

        let self_type = children.expect_next()?;
        let body = children.expect_next()?;
        let trait_path = children.expect_next()?;
        let type_parameters = if modifier.has_type_parameters() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let type_arguments = if modifier.has_type_arguments() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.has_where_clause() {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            self_type,
            body,
            trait_path,
            type_parameters,
            type_arguments,
            where_clause,
        })
    }
}

pub struct TraitItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub body: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub supertraits: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TraitItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting trait item");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TraitItem));

        let modifier = reader.node.modifier;
        let mut children = reader.children();

        let name = children.expect_next()?;
        let body = children.expect_next()?;
        let type_parameters = if modifier.has_type_parameters() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let supertraits = if modifier.has_supertraits() {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.has_where_clause() {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name,
            body,
            type_parameters,
            supertraits,
            where_clause,
        })
    }
}

pub struct TraitFunctionItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub signature: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for TraitFunctionItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting trait method");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TraitFunctionItem));

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.modifier.is_public(),
            name: children.expect_next()?,
            signature: children.expect_next()?,
        })
    }
}

pub struct TraitConstItem<'a> {
    pub name: SyntaxReader<'a>,
    pub type_notation: SyntaxReader<'a>,
    pub value: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TraitConstItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting trait const");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TraitConst));

        let mut children = reader.children();

        let name = children.expect_next()?;
        let type_notation = children.expect_next()?;
        let value = children.next();

        Ok(Self {
            name,
            type_notation,
            value,
        })
    }
}

pub struct TraitTypeItem<'a> {
    pub name: SyntaxReader<'a>,
    pub aliased_type: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TraitTypeItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting trait type");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TraitType));

        let mut children = reader.children();

        let name = children.expect_next()?;
        let aliased_type = children.next();

        Ok(Self { name, aliased_type })
    }
}

pub struct TypeParameter<'a> {
    pub name: SyntaxReader<'a>,
    pub bounds: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TypeParameter<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting type parameter");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TypeParameter));

        let mut children = reader.children();

        Ok(Self {
            name: children.expect_next()?,
            bounds: children.next(),
        })
    }
}

pub struct TraitBounds<'a> {
    pub bounds: SyntaxReaderIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for TraitBounds<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting trait bounds");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::TraitBounds));

        Ok(Self {
            bounds: reader.children(),
        })
    }
}

pub struct WherePredicate<'a> {
    pub bounded_type: SyntaxReader<'a>,
    pub bounds: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for WherePredicate<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting where predicate");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::WherePredicate));

        let (bounded_type, bounds) = reader.binary_children()?;

        Ok(Self {
            bounded_type,
            bounds,
        })
    }
}

pub struct WhereClause<'a> {
    pub predicates: SyntaxReaderIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for WhereClause<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting where clause");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::WhereClause));

        Ok(Self {
            predicates: reader.children(),
        })
    }
}

pub struct FieldAccessExpression<'a> {
    pub operand: SyntaxReader<'a>,
    pub field_name: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for FieldAccessExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting field access expression");
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::FieldAccessExpression
        ));

        let (operand, field_name) = reader.binary_children()?;

        Ok(Self {
            operand,
            field_name,
        })
    }
}

pub struct PathExpression<'a> {
    pub segments: SyntaxReaderIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for PathExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting path expression");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::PathExpression));

        Ok(Self {
            segments: reader.children(),
        })
    }
}

pub struct PathSegment<'a> {
    pub type_arguments: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for PathSegment<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting path segment");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::PathSegment));

        let mut children = reader.children();

        Ok(Self {
            type_arguments: children.next(),
        })
    }
}

pub struct ArrayType<'a> {
    pub element_type: SyntaxReader<'a>,
    pub length: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayType<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug!("Visiting array type");
        debug_assert!(matches!(reader.node.kind, SyntaxKind::ArrayType));

        let (element_type, length) = reader.binary_children()?;

        Ok(Self {
            element_type,
            length,
        })
    }
}
