use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{BuiltInFunction, Context, FunctionType, Identifier, RangeableType, StructType, Type};

use super::{AstError, Node, Span, Statement};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Expression {
    Block(Node<Box<BlockExpression>>),
    Break(Node<Option<Box<Expression>>>),
    Call(Node<Box<CallExpression>>),
    FieldAccess(Node<Box<FieldAccessExpression>>),
    Grouped(Node<Box<Expression>>),
    Identifier(Node<Identifier>),
    If(Node<Box<IfExpression>>),
    List(Node<Box<ListExpression>>),
    ListIndex(Node<Box<ListIndexExpression>>),
    Literal(Node<Box<LiteralExpression>>),
    Loop(Node<Box<LoopExpression>>),
    Map(Node<Box<MapExpression>>),
    Operator(Node<Box<OperatorExpression>>),
    Range(Node<Box<RangeExpression>>),
    Struct(Node<Box<StructExpression>>),
    TupleAccess(Node<Box<TupleAccessExpression>>),
}

impl Expression {
    pub fn r#break(expression: Option<Expression>, position: Span) -> Self {
        Self::Break(Node::new(expression.map(Box::new), position))
    }

    pub fn map<T: Into<Vec<(Node<Identifier>, Expression)>>>(pairs: T, position: Span) -> Self {
        Self::Map(Node::new(
            Box::new(MapExpression {
                pairs: pairs.into(),
            }),
            position,
        ))
    }

    pub fn operator(operator_expression: OperatorExpression, position: Span) -> Self {
        Self::Operator(Node::new(Box::new(operator_expression), position))
    }

    pub fn exclusive_range(start: Expression, end: Expression, position: Span) -> Self {
        Self::Range(Node::new(
            Box::new(RangeExpression::Exclusive { start, end }),
            position,
        ))
    }

    pub fn inclusive_range(start: Expression, end: Expression, position: Span) -> Self {
        Self::Range(Node::new(
            Box::new(RangeExpression::Inclusive { start, end }),
            position,
        ))
    }

    pub fn call(invoker: Expression, arguments: Vec<Expression>, position: Span) -> Self {
        Self::Call(Node::new(
            Box::new(CallExpression { invoker, arguments }),
            position,
        ))
    }

    pub fn field_access(container: Expression, field: Node<Identifier>, position: Span) -> Self {
        Self::FieldAccess(Node::new(
            Box::new(FieldAccessExpression { container, field }),
            position,
        ))
    }

    pub fn tuple_access(tuple: Expression, index: Node<usize>, position: Span) -> Self {
        Self::TupleAccess(Node::new(
            Box::new(TupleAccessExpression { tuple, index }),
            position,
        ))
    }

    pub fn assignment(assignee: Expression, value: Expression, position: Span) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Assignment { assignee, value }),
            position,
        ))
    }

    pub fn comparison(
        left: Expression,
        operator: Node<ComparisonOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Comparison {
                left,
                operator,
                right,
            }),
            position,
        ))
    }

    pub fn compound_assignment(
        assignee: Expression,
        operator: Node<MathOperator>,
        modifier: Expression,
        position: Span,
    ) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::CompoundAssignment {
                assignee,
                operator,
                modifier,
            }),
            position,
        ))
    }

    pub fn math(
        left: Expression,
        operator: Node<MathOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Math {
                left,
                operator,
                right,
            }),
            position,
        ))
    }

    pub fn negation(expression: Expression, position: Span) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Negation(expression)),
            position,
        ))
    }

    pub fn not(expression: Expression, position: Span) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Not(expression)),
            position,
        ))
    }

    pub fn logic(
        left: Expression,
        operator: Node<LogicOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::Logic {
                left,
                operator,
                right,
            }),
            position,
        ))
    }

    pub fn error_propagation(expression: Expression, position: Span) -> Self {
        Self::Operator(Node::new(
            Box::new(OperatorExpression::ErrorPropagation(expression)),
            position,
        ))
    }

    pub fn infinite_loop(block: Node<BlockExpression>, position: Span) -> Self {
        Self::Loop(Node::new(
            Box::new(LoopExpression::Infinite { block }),
            position,
        ))
    }

    pub fn while_loop(condition: Expression, block: Node<BlockExpression>, position: Span) -> Self {
        Self::Loop(Node::new(
            Box::new(LoopExpression::While { condition, block }),
            position,
        ))
    }

    pub fn for_loop(
        identifier: Node<Identifier>,
        iterator: Expression,
        block: Node<BlockExpression>,
        position: Span,
    ) -> Self {
        Self::Loop(Node::new(
            Box::new(LoopExpression::For {
                identifier,
                iterator,
                block,
            }),
            position,
        ))
    }

    pub fn block(block: BlockExpression, position: Span) -> Self {
        Self::Block(Node::new(Box::new(block), position))
    }

    pub fn grouped(expression: Expression, position: Span) -> Self {
        Self::Grouped(Node::new(Box::new(expression), position))
    }

    pub fn r#struct(struct_expression: StructExpression, position: Span) -> Self {
        Self::Struct(Node::new(Box::new(struct_expression), position))
    }

    pub fn identifier<T: ToString>(to_string: T, position: Span) -> Self {
        Self::Identifier(Node::new(Identifier::new(to_string), position))
    }

    pub fn list<T: Into<Vec<Expression>>>(expressions: T, position: Span) -> Self {
        Self::List(Node::new(
            Box::new(ListExpression::Ordered(expressions.into())),
            position,
        ))
    }

    pub fn auto_fill_list(repeat: Expression, length: Expression, position: Span) -> Self {
        Self::List(Node::new(
            Box::new(ListExpression::AutoFill {
                repeat_operand: repeat,
                length_operand: length,
            }),
            position,
        ))
    }

    pub fn list_index(list: Expression, index: Expression, position: Span) -> Self {
        Self::ListIndex(Node::new(
            Box::new(ListIndexExpression { list, index }),
            position,
        ))
    }

    pub fn r#if(if_expression: IfExpression, position: Span) -> Self {
        Self::If(Node::new(Box::new(if_expression), position))
    }

    pub fn literal<T: Into<LiteralExpression>>(into_literal: T, position: Span) -> Self {
        Self::Literal(Node::new(Box::new(into_literal.into()), position))
    }

    pub fn has_block(&self) -> bool {
        matches!(
            self,
            Expression::Block(_) | Expression::If(_) | Expression::Loop(_)
        )
    }

    pub fn as_identifier(&self) -> Option<&Identifier> {
        if let Expression::Identifier(identifier) = self {
            Some(&identifier.inner)
        } else {
            None
        }
    }

    pub fn return_type(&self, context: &Context) -> Result<Option<Type>, AstError> {
        let return_type = match self {
            Expression::Block(block_expression) => block_expression.inner.return_type(context)?,
            Expression::Break(expression_node) => {
                if let Some(expression) = expression_node.inner.as_ref() {
                    expression.return_type(context)?
                } else {
                    None
                }
            }
            Expression::Call(call_expression) => {
                let CallExpression { invoker, .. } = call_expression.inner.as_ref();

                let invoker_type = invoker.return_type(context)?;

                if let Some(Type::Function(FunctionType { return_type, .. })) = invoker_type {
                    return_type.map(|r#type| *r#type)
                } else if let Some(Type::Struct(_)) = invoker_type {
                    invoker_type
                } else {
                    None
                }
            }
            Expression::FieldAccess(field_access_expression) => {
                let FieldAccessExpression { container, field } =
                    field_access_expression.inner.as_ref();

                let container_type = container.return_type(context)?;

                if let Some(Type::Struct(StructType::Fields { fields, .. })) = container_type {
                    fields
                        .into_iter()
                        .find(|(name, _)| name == &field.inner)
                        .map(|(_, r#type)| r#type)
                } else {
                    None
                }
            }
            Expression::Grouped(expression) => expression.inner.return_type(context)?,
            Expression::Identifier(identifier) => {
                context
                    .get_type(&identifier.inner)
                    .map_err(|error| AstError::ContextError {
                        error,
                        position: identifier.position,
                    })?
            }
            Expression::If(if_expression) => match if_expression.inner.as_ref() {
                IfExpression::If { .. } => None,
                IfExpression::IfElse { if_block, .. } => if_block.inner.return_type(context)?,
            },
            Expression::List(list_expression) => match list_expression.inner.as_ref() {
                ListExpression::AutoFill { repeat_operand, .. } => {
                    let item_type = repeat_operand.return_type(context)?;

                    if let Some(r#type) = item_type {
                        Some(Type::ListOf {
                            item_type: Box::new(r#type),
                        })
                    } else {
                        return Err(AstError::ExpectedType {
                            position: repeat_operand.position(),
                        });
                    }
                }
                ListExpression::Ordered(expressions) => {
                    if expressions.is_empty() {
                        return Ok(Some(Type::ListEmpty));
                    }

                    let item_type = expressions
                        .first()
                        .ok_or_else(|| AstError::ExpectedNonEmptyList {
                            position: self.position(),
                        })?
                        .return_type(context)?
                        .ok_or_else(|| AstError::ExpectedType {
                            position: expressions.first().unwrap().position(),
                        })?;

                    let length = expressions.len();

                    Some(Type::List {
                        item_type: Box::new(item_type),
                        length,
                    })
                }
            },
            Expression::ListIndex(list_index_expression) => {
                let ListIndexExpression { list, .. } = list_index_expression.inner.as_ref();

                let list_type =
                    list.return_type(context)?
                        .ok_or_else(|| AstError::ExpectedType {
                            position: list.position(),
                        })?;

                if let Type::List { item_type, .. } = list_type {
                    Some(*item_type)
                } else {
                    None
                }
            }
            Expression::Literal(literal_expression) => match literal_expression.inner.as_ref() {
                LiteralExpression::BuiltInFunction(built_in_function) => {
                    built_in_function.return_type()
                }
                LiteralExpression::Primitive(primitive_value) => match primitive_value {
                    PrimitiveValueExpression::Boolean(_) => Some(Type::Boolean),
                    PrimitiveValueExpression::Character(_) => Some(Type::Character),
                    PrimitiveValueExpression::Integer(_) => Some(Type::Integer),
                    PrimitiveValueExpression::Float(_) => Some(Type::Float),
                },
                LiteralExpression::String(string) => Some(Type::String {
                    length: Some(string.len()),
                }),
            },
            Expression::Loop(loop_expression) => match loop_expression.inner.as_ref() {
                LoopExpression::For { block, .. } => block.inner.return_type(context)?,
                LoopExpression::Infinite { .. } => None,
                LoopExpression::While { block, .. } => block.inner.return_type(context)?,
            },
            Expression::Map(map_expression) => {
                let MapExpression { pairs } = map_expression.inner.as_ref();

                let mut types = HashMap::with_capacity(pairs.len());

                for (key, value) in pairs {
                    let value_type =
                        value
                            .return_type(context)?
                            .ok_or_else(|| AstError::ExpectedType {
                                position: value.position(),
                            })?;

                    types.insert(key.inner.clone(), value_type);
                }

                Some(Type::Map { pairs: types })
            }
            Expression::Operator(operator_expression) => match operator_expression.inner.as_ref() {
                OperatorExpression::Assignment { .. } => None,
                OperatorExpression::Comparison { .. } => Some(Type::Boolean),
                OperatorExpression::CompoundAssignment { .. } => None,
                OperatorExpression::ErrorPropagation(expression) => {
                    expression.return_type(context)?
                }
                OperatorExpression::Negation(expression) => expression.return_type(context)?,
                OperatorExpression::Not(_) => Some(Type::Boolean),
                OperatorExpression::Math { left, .. } => left.return_type(context)?,
                OperatorExpression::Logic { .. } => Some(Type::Boolean),
            },
            Expression::Range(range_expression) => {
                let start = match range_expression.inner.as_ref() {
                    RangeExpression::Exclusive { start, .. } => start,
                    RangeExpression::Inclusive { start, .. } => start,
                };
                let start_type =
                    start
                        .return_type(context)?
                        .ok_or_else(|| AstError::ExpectedType {
                            position: start.position(),
                        })?;
                let rangeable_type = match start_type {
                    Type::Byte => RangeableType::Byte,
                    Type::Character => RangeableType::Character,
                    Type::Float => RangeableType::Float,
                    Type::Integer => RangeableType::Integer,
                    _ => {
                        return Err(AstError::ExpectedRangeableType {
                            position: start.position(),
                        })
                    }
                };

                Some(Type::Range {
                    r#type: rangeable_type,
                })
            }
            Expression::Struct(struct_expression) => match struct_expression.inner.as_ref() {
                StructExpression::Fields { name, fields } => {
                    let mut types = HashMap::with_capacity(fields.len());

                    for (field, expression) in fields {
                        let r#type = expression.return_type(context)?.ok_or_else(|| {
                            AstError::ExpectedType {
                                position: expression.position(),
                            }
                        })?;

                        types.insert(field.inner.clone(), r#type);
                    }

                    Some(Type::Struct(StructType::Fields {
                        name: name.inner.clone(),
                        fields: types,
                    }))
                }
            },
            Expression::TupleAccess(tuple_access_expression) => {
                let TupleAccessExpression { tuple, index } = tuple_access_expression.inner.as_ref();
                let tuple_value =
                    tuple
                        .return_type(context)?
                        .ok_or_else(|| AstError::ExpectedType {
                            position: tuple.position(),
                        })?;

                if let Type::Tuple {
                    fields: Some(fields),
                } = tuple_value
                {
                    fields.get(index.inner).cloned()
                } else {
                    Err(AstError::ExpectedTupleType {
                        position: tuple.position(),
                    })?
                }
            }
        };

        Ok(return_type)
    }

    pub fn position(&self) -> Span {
        match self {
            Expression::Block(block) => block.position,
            Expression::Break(expression_node) => expression_node.position,
            Expression::Call(call) => call.position,
            Expression::FieldAccess(field_access) => field_access.position,
            Expression::Grouped(grouped) => grouped.position,
            Expression::Identifier(identifier) => identifier.position,
            Expression::If(r#if) => r#if.position,
            Expression::List(list) => list.position,
            Expression::ListIndex(list_index) => list_index.position,
            Expression::Literal(literal) => literal.position,
            Expression::Loop(r#loop) => r#loop.position,
            Expression::Map(map) => map.position,
            Expression::Operator(operator) => operator.position,
            Expression::Range(range) => range.position,
            Expression::Struct(r#struct) => r#struct.position,
            Expression::TupleAccess(tuple_access) => tuple_access.position,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Block(block) => write!(f, "{}", block.inner),
            Expression::Break(break_node) => {
                if let Some(expression_node) = &break_node.inner {
                    write!(f, "break {};", expression_node)
                } else {
                    write!(f, "break;")
                }
            }
            Expression::Call(call) => write!(f, "{}", call.inner),
            Expression::FieldAccess(field_access) => write!(f, "{}", field_access.inner),
            Expression::Grouped(grouped) => write!(f, "({})", grouped.inner),
            Expression::Identifier(identifier) => write!(f, "{}", identifier.inner),
            Expression::If(r#if) => write!(f, "{}", r#if.inner),
            Expression::List(list) => write!(f, "{}", list.inner),
            Expression::ListIndex(list_index) => write!(f, "{}", list_index.inner),
            Expression::Literal(literal) => write!(f, "{}", literal.inner),
            Expression::Loop(r#loop) => write!(f, "{}", r#loop.inner),
            Expression::Map(map) => write!(f, "{}", map.inner),
            Expression::Operator(operator) => write!(f, "{}", operator.inner),
            Expression::Range(range) => write!(f, "{}", range),
            Expression::Struct(r#struct) => write!(f, "{}", r#struct.inner),
            Expression::TupleAccess(tuple_access) => write!(f, "{}", tuple_access),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MapExpression {
    pub pairs: Vec<(Node<Identifier>, Expression)>,
}

impl Display for MapExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{")?;

        for (index, (key, value)) in self.pairs.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{} = {}", key.inner, value)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TupleAccessExpression {
    pub tuple: Expression,
    pub index: Node<usize>,
}

impl Display for TupleAccessExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.tuple, self.index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RangeExpression {
    Exclusive { start: Expression, end: Expression },
    Inclusive { start: Expression, end: Expression },
}

impl Display for RangeExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RangeExpression::Exclusive { start, end } => write!(f, "{}..{}", start, end),
            RangeExpression::Inclusive { start, end } => write!(f, "{}..={}", start, end),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ListIndexExpression {
    pub list: Expression,
    pub index: Expression,
}

impl Display for ListIndexExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.list, self.index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CallExpression {
    pub invoker: Expression,
    pub arguments: Vec<Expression>,
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}(", self.invoker)?;

        for (index, argument) in self.arguments.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", argument)?;
        }

        write!(f, ")")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldAccessExpression {
    pub container: Expression,
    pub field: Node<Identifier>,
}

impl Display for FieldAccessExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.container, self.field)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ListExpression {
    AutoFill {
        repeat_operand: Expression,
        length_operand: Expression,
    },
    Ordered(Vec<Expression>),
}

impl Display for ListExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ListExpression::AutoFill {
                repeat_operand,
                length_operand,
            } => {
                write!(f, "[{};{}]", repeat_operand, length_operand)
            }
            ListExpression::Ordered(expressions) => {
                write!(f, "[")?;

                for (index, expression) in expressions.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", expression)?;
                }

                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValueExpression {
    Boolean(bool),
    Character(char),
    Float(f64),
    Integer(i64),
}

impl Display for PrimitiveValueExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveValueExpression::Boolean(boolean) => write!(f, "{boolean}"),
            PrimitiveValueExpression::Character(character) => write!(f, "'{character}'"),
            PrimitiveValueExpression::Float(float) => write!(f, "{float}"),
            PrimitiveValueExpression::Integer(integer) => write!(f, "{integer}"),
        }
    }
}

impl Eq for PrimitiveValueExpression {}

impl PartialOrd for PrimitiveValueExpression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrimitiveValueExpression {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PrimitiveValueExpression::Boolean(left), PrimitiveValueExpression::Boolean(right)) => {
                left.cmp(right)
            }
            (PrimitiveValueExpression::Boolean(_), _) => Ordering::Greater,
            (
                PrimitiveValueExpression::Character(left),
                PrimitiveValueExpression::Character(right),
            ) => left.cmp(right),
            (PrimitiveValueExpression::Character(_), _) => Ordering::Greater,
            (PrimitiveValueExpression::Float(left), PrimitiveValueExpression::Float(right)) => {
                left.to_bits().cmp(&right.to_bits())
            }
            (PrimitiveValueExpression::Float(_), _) => Ordering::Greater,
            (PrimitiveValueExpression::Integer(left), PrimitiveValueExpression::Integer(right)) => {
                left.cmp(right)
            }
            (PrimitiveValueExpression::Integer(_), _) => Ordering::Greater,
        }
    }
}

impl From<i64> for LiteralExpression {
    fn from(value: i64) -> Self {
        LiteralExpression::Primitive(PrimitiveValueExpression::Integer(value))
    }
}

impl From<String> for LiteralExpression {
    fn from(value: String) -> Self {
        LiteralExpression::String(value)
    }
}

impl From<&str> for LiteralExpression {
    fn from(value: &str) -> Self {
        LiteralExpression::String(value.to_string())
    }
}

impl From<f64> for LiteralExpression {
    fn from(value: f64) -> Self {
        LiteralExpression::Primitive(PrimitiveValueExpression::Float(value))
    }
}

impl From<bool> for LiteralExpression {
    fn from(value: bool) -> Self {
        LiteralExpression::Primitive(PrimitiveValueExpression::Boolean(value))
    }
}

impl From<char> for LiteralExpression {
    fn from(value: char) -> Self {
        LiteralExpression::Primitive(PrimitiveValueExpression::Character(value))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralExpression {
    BuiltInFunction(BuiltInFunction),
    Primitive(PrimitiveValueExpression),
    String(String),
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LiteralExpression::BuiltInFunction(built_in_function) => {
                write!(f, "{built_in_function}")
            }
            LiteralExpression::Primitive(primitive) => {
                write!(f, "{primitive}")
            }
            LiteralExpression::String(string) => write!(f, "\"{string}\""),
        }
    }
}

impl Eq for LiteralExpression {}

impl PartialOrd for LiteralExpression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LiteralExpression {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                LiteralExpression::BuiltInFunction(left),
                LiteralExpression::BuiltInFunction(right),
            ) => left.cmp(right),
            (LiteralExpression::BuiltInFunction(_), _) => Ordering::Greater,
            (LiteralExpression::Primitive(left), LiteralExpression::Primitive(right)) => {
                left.cmp(right)
            }
            (LiteralExpression::Primitive(_), _) => Ordering::Greater,
            (LiteralExpression::String(left), LiteralExpression::String(right)) => left.cmp(right),
            (LiteralExpression::String(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperatorExpression {
    Assignment {
        assignee: Expression,
        value: Expression,
    },
    Comparison {
        left: Expression,
        operator: Node<ComparisonOperator>,
        right: Expression,
    },
    CompoundAssignment {
        assignee: Expression,
        operator: Node<MathOperator>,
        modifier: Expression,
    },
    ErrorPropagation(Expression),
    Negation(Expression),
    Not(Expression),
    Math {
        left: Expression,
        operator: Node<MathOperator>,
        right: Expression,
    },
    Logic {
        left: Expression,
        operator: Node<LogicOperator>,
        right: Expression,
    },
}

impl Display for OperatorExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            OperatorExpression::Assignment { assignee, value } => {
                write!(f, "{} = {}", assignee, value)
            }
            OperatorExpression::Comparison {
                left,
                operator,
                right,
            } => {
                write!(f, "{} {} {}", left, operator, right)
            }
            OperatorExpression::CompoundAssignment {
                assignee,
                operator,
                modifier: value,
            } => write!(f, "{} {}= {}", assignee, operator, value),
            OperatorExpression::ErrorPropagation(expression) => write!(f, "{}?", expression),
            OperatorExpression::Negation(expression) => write!(f, "-{}", expression),
            OperatorExpression::Not(expression) => write!(f, "!{}", expression),
            OperatorExpression::Math {
                left,
                operator,
                right,
            } => {
                write!(f, "{} {} {}", left, operator, right)
            }
            OperatorExpression::Logic {
                left,
                operator,
                right,
            } => {
                write!(f, "{} {} {}", left, operator, right)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl Display for ComparisonOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let operator = match self {
            ComparisonOperator::Equal => "==",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
        };

        write!(f, "{}", operator)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl MathOperator {
    pub fn add(position: Span) -> Node<Self> {
        Node::new(MathOperator::Add, position)
    }

    pub fn subtract(position: Span) -> Node<Self> {
        Node::new(MathOperator::Subtract, position)
    }

    pub fn multiply(position: Span) -> Node<Self> {
        Node::new(MathOperator::Multiply, position)
    }

    pub fn divide(position: Span) -> Node<Self> {
        Node::new(MathOperator::Divide, position)
    }

    pub fn modulo(position: Span) -> Node<Self> {
        Node::new(MathOperator::Modulo, position)
    }
}

impl Display for MathOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let operator = match self {
            MathOperator::Add => "+",
            MathOperator::Subtract => "-",
            MathOperator::Multiply => "*",
            MathOperator::Divide => "/",
            MathOperator::Modulo => "%",
        };

        write!(f, "{}", operator)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogicOperator {
    And,
    Or,
}

impl Display for LogicOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let operator = match self {
            LogicOperator::And => "&&",
            LogicOperator::Or => "||",
        };

        write!(f, "{}", operator)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IfExpression {
    If {
        condition: Expression,
        if_block: Node<BlockExpression>,
    },
    IfElse {
        condition: Expression,
        if_block: Node<BlockExpression>,
        r#else: ElseExpression,
    },
}

impl IfExpression {
    pub fn r#if(condition: Expression, if_block: Node<BlockExpression>) -> Self {
        IfExpression::If {
            condition,
            if_block,
        }
    }

    pub fn if_else(
        condition: Expression,
        if_block: Node<BlockExpression>,
        else_block: Node<BlockExpression>,
    ) -> Self {
        IfExpression::IfElse {
            condition,
            if_block,
            r#else: ElseExpression::Block(else_block),
        }
    }

    pub fn if_else_if(
        condition: Expression,
        if_block: Node<BlockExpression>,
        else_if: Node<Box<IfExpression>>,
    ) -> Self {
        IfExpression::IfElse {
            condition,
            if_block,
            r#else: ElseExpression::If(else_if),
        }
    }
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IfExpression::If {
                condition,
                if_block,
            } => {
                write!(f, "if {} {}", condition, if_block)
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                write!(f, "if {} {} else {}", condition, if_block, r#else)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ElseExpression {
    Block(Node<BlockExpression>),
    If(Node<Box<IfExpression>>),
}

impl Display for ElseExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ElseExpression::Block(block) => write!(f, "{}", block),
            ElseExpression::If(r#if) => write!(f, "{}", r#if),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockExpression {
    Async(Vec<Statement>),
    Sync(Vec<Statement>),
}

impl BlockExpression {
    fn return_type(&self, context: &Context) -> Result<Option<Type>, AstError> {
        match self {
            BlockExpression::Async(statements) | BlockExpression::Sync(statements) => {
                if let Some(statement) = statements.last() {
                    statement.return_type(context)
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockExpression::Async(statements) => {
                write!(f, "async {{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "{}", statement)?;
                }

                write!(f, " }}")
            }
            BlockExpression::Sync(statements) => {
                write!(f, "{{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "{}", statement)?;
                }

                write!(f, " }}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoopExpression {
    Infinite {
        block: Node<BlockExpression>,
    },
    While {
        condition: Expression,
        block: Node<BlockExpression>,
    },
    For {
        identifier: Node<Identifier>,
        iterator: Expression,
        block: Node<BlockExpression>,
    },
}

impl Display for LoopExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LoopExpression::Infinite { block } => write!(f, "loop {}", block),
            LoopExpression::While { condition, block } => {
                write!(f, "while {} {}", condition, block)
            }
            LoopExpression::For {
                identifier,
                iterator,
                block,
            } => write!(f, "for {} in {} {}", identifier, iterator, block),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructExpression {
    // The unit struct expression is omitted because it is redundant with identifier expressions
    // The tuple struct expression is omitted because it is redundant with call expression
    Fields {
        name: Node<Identifier>,
        fields: Vec<(Node<Identifier>, Expression)>,
    },
}

impl Display for StructExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructExpression::Fields { name, fields } => {
                write!(f, "{} {{", name)?;

                for (index, (field, value)) in fields.iter().enumerate() {
                    write!(f, "{}: {}", field, value)?;

                    if index < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")
            }
        }
    }
}
