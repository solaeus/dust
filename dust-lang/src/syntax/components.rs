use crate::syntax::{
    error::SyntaxError,
    node::{SyntaxFlags, SyntaxKind},
    reader::{SyntaxIterator, SyntaxPairIterator, SyntaxReader},
};

pub trait SyntaxComponent<'a>: Sized {
    const SYNTAX_KIND: SyntaxKind;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError>;
}

pub struct Root<'a> {
    pub items: &'a SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for Root<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self { items: reader })
    }
}

pub struct ModItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for ModItem<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::ModItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::UseItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let mut children = reader.children();

        Ok(Self {
            public: reader.node.flags.get_flag(SyntaxFlags::PUBLIC),
            path: children.expect_next()?,
        })
    }
}

pub struct FnItem<'a> {
    pub public: bool,
    pub name: SyntaxReader<'a>,
    pub type_parameters: Option<SyntaxReader<'a>>,
    pub value_parameters: Option<SyntaxReader<'a>>,
    pub return_type: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
    pub body: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for FnItem<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::FnItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    pub name_type_pairs: SyntaxPairIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for ValueParameters<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::ValueParameters;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert!(reader.node.kind == SyntaxKind::ValueParameters);

        Ok(Self {
            name_type_pairs: reader.child_pairs(),
        })
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::StructItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert!(reader.node.kind == SyntaxKind::StructItem);

        let mut children = reader.children();

        let public = reader.node.flags.get_flag(SyntaxFlags::PUBLIC);
        let name = children.expect_next()?;
        let type_parameters = if reader.node.flags.get_flag(SyntaxFlags::TYPE_PARAMETERS) {
            Some(children.expect_next()?)
        } else {
            None
        };
        let fields = if reader.node.flags.get_flag(SyntaxFlags::NAMED_FIELDS) {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::TupleFields;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            types: reader.children(),
        })
    }
}

pub struct NamedFields<'a> {
    pub name_type_pairs: SyntaxPairIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for NamedFields<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::NamedFields;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::EnumItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::EnumUnitVariant;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self { name: reader })
    }
}

pub struct EnumItemTupleVariant<'a> {
    pub name: SyntaxReader<'a>,
    pub tuple_fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumItemTupleVariant<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::EnumTupleFieldsVariant;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (name, tuple_fields) = reader.binary_children()?;

        Ok(Self { name, tuple_fields })
    }
}

pub struct EnumNamedFieldsVariant<'a> {
    pub name: SyntaxReader<'a>,
    pub named_fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for EnumNamedFieldsVariant<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::EnumNamedFieldsVariant;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::LetStatement;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::ExpressionStatement;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            expression: reader.single_child()?,
        })
    }
}

pub struct BlockExpression<'a> {
    pub children: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for BlockExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::BlockExpression;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::AssignmentExpression;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct NotExpression<'a> {
    pub operand: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for NotExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            operand: reader.single_child()?,
        })
    }
}

pub struct ArrayExpression<'a> {
    pub elements: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for ArrayExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (collection, index) = reader.binary_children()?;

        Ok(Self { collection, index })
    }
}

pub struct RangeExpression<'a> {
    pub start: SyntaxReader<'a>,
    pub end: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for RangeExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (condition, body) = reader.binary_children()?;

        Ok(Self { condition, body })
    }
}

pub struct CallExpression<'a> {
    pub callee: SyntaxReader<'a>,
    pub arguments: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for CallExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (callee, arguments) = reader.binary_children()?;

        Ok(Self { callee, arguments })
    }
}

pub struct StructExpression<'a> {
    pub path: SyntaxReader<'a>,
    pub fields: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for StructExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (path, fields) = reader.binary_children()?;

        Ok(Self { path, fields })
    }
}

pub struct StructExpressionStructFields<'a> {
    pub name_expression_pairs: SyntaxPairIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for StructExpressionStructFields<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert!(reader.node.kind == SyntaxKind::StructExpressionNamedFields);

        Ok(Self {
            name_expression_pairs: reader.child_pairs(),
        })
    }
}

pub struct GroupedExpression<'a> {
    pub expression: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for GroupedExpression<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::ConstItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::TypeItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    pub self_name: SyntaxReader<'a>,
    pub type_arguments: Option<SyntaxReader<'a>>,
    pub where_clause: Option<SyntaxReader<'a>>,
    pub body: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for ImplItem<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::ImplItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
        let self_name = children.expect_next()?;
        let type_arguments = if modifier.get_flag(SyntaxFlags::TYPE_ARGUMENTS) {
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
            body,
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::TraitItem;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        debug_assert!(matches!(
            reader.node.kind,
            SyntaxKind::FieldAccessExpression
        ));

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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            segments: reader.children(),
        })
    }
}

pub struct PathSegment<'a> {
    pub type_arguments: Option<SyntaxReader<'a>>,
}

impl<'a> SyntaxComponent<'a> for PathSegment<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
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
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        let (element_type, length) = reader.binary_children()?;

        Ok(Self {
            element_type,
            length,
        })
    }
}

pub struct SliceType<'a> {
    pub element_type: SyntaxReader<'a>,
}

impl<'a> SyntaxComponent<'a> for SliceType<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            element_type: reader.single_child()?,
        })
    }
}

pub struct TupleType<'a> {
    pub element_types: SyntaxIterator<'a>,
}

impl<'a> SyntaxComponent<'a> for TupleType<'a> {
    const SYNTAX_KIND: SyntaxKind = SyntaxKind::Root;

    fn from_reader(reader: &'a SyntaxReader<'a>) -> Result<Self, SyntaxError> {
        Ok(Self {
            element_types: reader.children(),
        })
    }
}
