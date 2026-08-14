use crate::syntax::{
    error::SyntaxError,
    node::{SyntaxFlags, SyntaxKind},
    reader::{SyntaxIterator, SyntaxPairIterator, SyntaxReader},
};

pub trait SyntaxComponent<'a>: Sized {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError>;
}

pub struct Root<'a> {
    pub items: &'a SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for Root<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::Root);

        Ok(Self { items: reader })
    }
}

pub struct ModItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ModItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ModItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::UseItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
            path: children.expect_next()?,
        })
    }
}

pub struct FunctionItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub value_parameters: Option<SyntaxReader<'a>>,
    pub return_type: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for FunctionItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::FunctionItem);

        let mut children = reader.children();

        let public = reader.node.flags.get_flag(SyntaxFlags::PUBLIC);
        let name = children.expect_next()?;
        let type_parameters = if reader.node.flags.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let value_parameters = if reader.node.flags.get_flag(SyntaxFlags::VALUE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let return_type = if reader.node.flags.get_flag(SyntaxFlags::RETURN_TYPE) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if reader.node.flags.get_flag(SyntaxFlags::WHERE_CLAUSE) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let body = children.next();

        Ok(Self {
            public,
            name,
            type_parameters,
            value_parameters,
            return_type,
            where_clause,
            body,
        })
    }
}

pub struct ValueParameters<'a> {
    pub parameters: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for ValueParameters<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ValueParameters);

        Ok(Self {
            parameters: reader.children(),
        })
    }
}

pub enum ValueParameter<'a> {
    SelfParameter(SyntaxReader<'a>),
    Named {
        name: SyntaxReader<'a>,
        type_notation: SyntaxReader<'a>,
    },
}

impl<'a> SyntaxComponent<'a> for ValueParameter<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ValueParameter);

        if reader.child_count() == 1 {
            Ok(Self::SelfParameter(reader.single_child()?))
        } else {
            let (name, type_notation) = reader.binary_children()?;

            Ok(Self::Named {
                name,
                type_notation,
            })
        }
    }
}

pub struct StructItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub fields: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for StructItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::StructItem);

        let mut children = reader.children();

        let public = reader.node.flags.get_flag(SyntaxFlags::PUBLIC);
        let name = children.expect_next()?;
        let type_parameters = if reader.node.flags.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let fields = if reader.node.flags.get_flag(SyntaxFlags::FIELDS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = children.next();

        Ok(Self {
            public,
            name,
            type_parameters,
            fields,
            where_clause,
        })
    }
}

pub struct TupleFields<'a> {
    pub types: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for TupleFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::TupleFields);

        Ok(Self {
            types: reader.children(),
        })
    }
}

pub struct NamedFields<'a> {
    pub name_type_pairs: SyntaxPairIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for NamedFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::NamedFields);

        Ok(Self {
            name_type_pairs: reader.child_pairs(),
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::EnumItem);

        let mut children = reader.children();

        let public = reader.node.flags.get_flag(SyntaxFlags::PUBLIC);
        let name = children.expect_next()?;
        let type_parameters = if reader.node.flags.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let variants = children.expect_next()?;

        Ok(Self {
            public,
            name,
            type_parameters,
            variants,
        })
    }
}

pub struct EnumUnitVariant<'a> {
    pub name: &'a SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumUnitVariant<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::EnumUnitVariant);

        Ok(Self { name: reader })
    }
}

pub struct EnumItemTupleVariant<'a> {
    pub name: SyntaxReader<'a>,
    pub tuple_fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumItemTupleVariant<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::EnumTupleFieldsVariant);

        let (name, tuple_fields) = reader.binary_children()?;

        Ok(Self { name, tuple_fields })
    }
}

pub struct EnumNamedFieldsVariant<'a> {
    pub name: SyntaxReader<'a>,
    pub named_fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumNamedFieldsVariant<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::EnumNamedFieldsVariant);

        let (name, named_fields) = reader.binary_children()?;

        Ok(Self { name, named_fields })
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::LetStatement);

        let mut children = reader.children();

        Ok(Self {
            mutable: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::ExpressionStatement);

        Ok(Self {
            expression: reader.single_child()?,
        })
    }
}

pub struct BlockExpression<'a> {
    pub children: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for BlockExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::BlockExpression);

        Ok(Self {
            children: reader.children(),
        })
    }
}

pub struct AssignmentExpression<'a> {
    pub target: SyntaxReader<'a>,
    pub source: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for AssignmentExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::AssignmentExpression);

        let (target, value) = reader.binary_children()?;

        Ok(Self {
            target,
            source: value,
        })
    }
}

pub struct MathExpression<'a> {
    pub left: SyntaxReader<'a>,
    pub right: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for MathExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::NegationExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::NotExpression);

        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct ReferenceExpression<'a> {
    pub operand: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ReferenceExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ReferenceExpression);

        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct ArrayExpression<'a> {
    pub elements: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ArrayExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::ArrayRepeatExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::IndexExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::IfExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::WhileExpression);

        let (condition, body) = reader.binary_children()?;

        Ok(Self { condition, body })
    }
}

pub struct CallExpression<'a> {
    pub callee: SyntaxReader<'a>,
    pub arguments: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for CallExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::CallExpression);

        if reader.child_count() == 1 {
            Ok(Self {
                callee: reader.single_child()?,
                arguments: None,
            })
        } else {
            let (callee, arguments) = reader.binary_children()?;

            Ok(Self {
                callee,
                arguments: Some(arguments),
            })
        }
    }
}

pub struct MethodCallExpression<'a> {
    pub method_parent: SyntaxReader<'a>,
    pub method: SyntaxReader<'a>,
    pub type_arguments: Option<SyntaxReader<'a>>,
    pub value_arguments: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for MethodCallExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::MethodCallExpression);

        let mut children = reader.children();

        let method_parent = children.expect_next()?;
        let method = children.expect_next()?;
        let type_arguments = if reader.node.flags.get_flag(SyntaxFlags::TYPE_ARGUMENTS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let value_arguments = if reader.node.flags.get_flag(SyntaxFlags::VALUE_ARGUMENTS) {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            method_parent,
            method,
            type_arguments,
            value_arguments,
        })
    }
}

pub struct StructExpression<'a> {
    pub path: SyntaxReader<'a>,
    pub fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for StructExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::StructExpression);

        let (path, fields) = reader.binary_children()?;

        Ok(Self { path, fields })
    }
}

pub struct StructExpressionStructFields<'a> {
    pub name_expression_pairs: SyntaxPairIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for StructExpressionStructFields<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::StructExpressionNamedFields);

        Ok(Self {
            name_expression_pairs: reader.child_pairs(),
        })
    }
}

pub struct GroupedExpression<'a> {
    pub expression: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for GroupedExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::GroupedExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::FunctionType);

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
    pub type_notation: SyntaxReader<'a>,
    pub value: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ConstItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ConstItem);

        let mut children = reader.children();

        Ok(Self {
            public: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
            name: children.expect_next()?,
            type_notation: children.expect_next()?,
            value: children.next(),
        })
    }
}

pub struct TypeItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub aliased_type: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TypeItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::TypeItem);

        let mut children = reader.children();

        let public = reader.node.flags.get_flag(SyntaxFlags::PUBLIC);
        let name = children.expect_next()?;
        let type_parameters = if reader.node.flags.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let aliased_type = children.next();

        Ok(Self {
            public,
            name,
            aliased_type,
            type_parameters,
        })
    }
}

pub struct ImplItem<'a> {
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub trait_path: Option<SyntaxReader<'a>>,
    pub trait_type_arguments: Option<SyntaxReader<'a>>,
    pub self_name: SyntaxReader<'a>,
    pub self_type_arguments: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
    pub body: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ImplItem<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ImplItem);

        let modifier = reader.node.flags;
        let mut children = reader.children();

        let type_parameters = if modifier.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let trait_path = if modifier.get_flag(SyntaxFlags::TYPE_NAME) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let trait_type_arguments = if modifier.get_flag(SyntaxFlags::TRAIT_TYPE_ARGUMENTS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let self_name = children.expect_next()?;
        let self_type_arguments = if modifier.get_flag(SyntaxFlags::TYPE_ARGUMENTS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.get_flag(SyntaxFlags::WHERE_CLAUSE) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let body = children.expect_next()?;

        Ok(Self {
            self_name,
            trait_path,
            trait_type_arguments,
            body,
            type_parameters,
            self_type_arguments,
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
        debug_assert_eq!(reader.node.kind, SyntaxKind::TraitItem);

        let modifier = reader.node.flags;
        let mut children = reader.children();

        let name = children.expect_next()?;
        let body = children.expect_next()?;
        let type_parameters = if modifier.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let supertraits = if modifier.get_flag(SyntaxFlags::SUPERTRAITS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let where_clause = if modifier.get_flag(SyntaxFlags::WHERE_CLAUSE) {
            Some(children.expect_next()?)
        } else {
            None
        };

        Ok(Self {
            public: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
            name,
            body,
            type_parameters,
            supertraits,
            where_clause,
        })
    }
}

pub struct TypeParameter<'a> {
    pub name: SyntaxReader<'a>,
    pub bounds: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for TypeParameter<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::TypeParameter);

        let mut children = reader.children();

        Ok(Self {
            name: children.expect_next()?,
            bounds: children.next(),
        })
    }
}

pub struct TraitBounds<'a> {
    pub bounds: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for TraitBounds<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::TraitBounds);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::WherePredicate);

        let (bounded_type, bounds) = reader.binary_children()?;

        Ok(Self {
            bounded_type,
            bounds,
        })
    }
}

pub struct WhereClause<'a> {
    pub predicates: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for WhereClause<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::WhereClause);

        Ok(Self {
            predicates: reader.children(),
        })
    }
}

pub struct FieldAccessExpression<'a> {
    pub struct_expression: SyntaxReader<'a>,
    pub field_name: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for FieldAccessExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::FieldAccessExpression);

        let (struct_expression, field_name) = reader.binary_children()?;

        Ok(Self {
            struct_expression,
            field_name,
        })
    }
}

pub struct PathExpression<'a> {
    pub segments: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for PathExpression<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::PathExpression);

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
        debug_assert_eq!(reader.node.kind, SyntaxKind::PathSegment);

        if reader.has_children() {
            Ok(Self {
                type_arguments: Some(reader.single_child()?),
            })
        } else {
            Ok(Self {
                type_arguments: None,
            })
        }
    }
}

pub struct ArrayType<'a> {
    pub element_type: SyntaxReader<'a>,
    pub length: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayType<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ArrayType);

        let (element_type, length) = reader.binary_children()?;

        Ok(Self {
            element_type,
            length,
        })
    }
}

pub struct TupleType<'a> {
    pub element_types: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for TupleType<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::TupleType);

        Ok(Self {
            element_types: reader.children(),
        })
    }
}

pub struct ReferenceType<'a> {
    pub referenced_type: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ReferenceType<'a> {
    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert_eq!(reader.node.kind, SyntaxKind::ReferenceType);

        Ok(Self {
            referenced_type: reader.single_child()?,
        })
    }
}
