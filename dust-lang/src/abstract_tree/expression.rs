use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Span, Value};

use super::{Node, Statement};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Expression {
    Block(Node<Box<Block>>),
    Call(Node<Box<CallExpression>>),
    FieldAccess(Node<Box<FieldAccess>>),
    Grouped(Node<Box<Expression>>),
    Identifier(Node<Identifier>),
    If(Node<Box<IfExpression>>),
    List(Node<Box<ListExpression>>),
    ListIndex(Node<Box<ListIndex>>),
    Literal(Node<Box<LiteralExpression>>),
    Loop(Node<Box<Loop>>),
    Operator(Node<Box<OperatorExpression>>),
    Range(Node<Box<Range>>),
    Struct(Node<Box<StructExpression>>),
    TupleAccess(Node<Box<TupleAccess>>),
}

impl Expression {
    pub fn operator(operator_expression: OperatorExpression, position: Span) -> Self {
        Self::Operator(Node::new(Box::new(operator_expression), position))
    }

    pub fn range(start: Expression, end: Expression, position: Span) -> Self {
        Self::Range(Node::new(Box::new(Range { start, end }), position))
    }

    pub fn call(invoker: Expression, arguments: Vec<Expression>, position: Span) -> Self {
        Self::Call(Node::new(
            Box::new(CallExpression { invoker, arguments }),
            position,
        ))
    }

    pub fn field_access(container: Expression, field: Node<Identifier>, position: Span) -> Self {
        Self::FieldAccess(Node::new(
            Box::new(FieldAccess { container, field }),
            position,
        ))
    }

    pub fn tuple_access(tuple: Expression, index: Node<usize>, position: Span) -> Self {
        Self::TupleAccess(Node::new(Box::new(TupleAccess { tuple, index }), position))
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

    pub fn infinite_loop(block: Node<Block>, position: Span) -> Self {
        Self::Loop(Node::new(Box::new(Loop::Infinite { block }), position))
    }

    pub fn while_loop(condition: Expression, block: Node<Block>, position: Span) -> Self {
        Self::Loop(Node::new(
            Box::new(Loop::While { condition, block }),
            position,
        ))
    }

    pub fn for_loop(
        identifier: Node<Identifier>,
        iterator: Expression,
        block: Node<Block>,
        position: Span,
    ) -> Self {
        Self::Loop(Node::new(
            Box::new(Loop::For {
                identifier,
                iterator,
                block,
            }),
            position,
        ))
    }

    pub fn block(block: Block, position: Span) -> Self {
        Self::Block(Node::new(Box::new(block), position))
    }

    pub fn grouped(expression: Expression, position: Span) -> Self {
        Self::Grouped(Node::new(Box::new(expression), position))
    }

    pub fn r#struct(struct_expression: StructExpression, position: Span) -> Self {
        Self::Struct(Node::new(Box::new(struct_expression), position))
    }

    pub fn identifier(identifier: Identifier, position: Span) -> Self {
        Self::Identifier(Node::new(identifier, position))
    }

    pub fn list(list_expression: ListExpression, position: Span) -> Self {
        Self::List(Node::new(Box::new(list_expression), position))
    }

    pub fn list_index(list_index: ListIndex, position: Span) -> Self {
        Self::ListIndex(Node::new(Box::new(list_index), position))
    }

    pub fn r#if(r#if: IfExpression, position: Span) -> Self {
        Self::If(Node::new(Box::new(r#if), position))
    }

    pub fn literal(literal: LiteralExpression, position: Span) -> Self {
        Self::Literal(Node::new(Box::new(literal), position))
    }

    pub fn as_identifier(&self) -> Option<&Identifier> {
        if let Expression::Identifier(identifier) = self {
            Some(&identifier.inner)
        } else {
            None
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Expression::Block(block) => block.position,
            Expression::Call(call) => call.position,
            Expression::FieldAccess(field_access) => field_access.position,
            Expression::Grouped(grouped) => grouped.position,
            Expression::Identifier(identifier) => identifier.position,
            Expression::If(r#if) => r#if.position,
            Expression::List(list) => list.position,
            Expression::ListIndex(list_index) => list_index.position,
            Expression::Literal(literal) => literal.position,
            Expression::Loop(r#loop) => r#loop.position,
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
            Expression::Call(call) => write!(f, "{}", call.inner),
            Expression::FieldAccess(field_access) => write!(f, "{}", field_access.inner),
            Expression::Grouped(grouped) => write!(f, "({})", grouped.inner),
            Expression::Identifier(identifier) => write!(f, "{}", identifier.inner),
            Expression::If(r#if) => write!(f, "{}", r#if.inner),
            Expression::List(list) => write!(f, "{}", list.inner),
            Expression::ListIndex(list_index) => write!(f, "{}", list_index.inner),
            Expression::Literal(literal) => write!(f, "{}", literal.inner),
            Expression::Loop(r#loop) => write!(f, "{}", r#loop.inner),
            Expression::Operator(operator) => write!(f, "{}", operator.inner),
            Expression::Range(range) => write!(f, "{}", range),
            Expression::Struct(r#struct) => write!(f, "{}", r#struct.inner),
            Expression::TupleAccess(tuple_access) => write!(f, "{}", tuple_access),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TupleAccess {
    pub tuple: Expression,
    pub index: Node<usize>,
}

impl Display for TupleAccess {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.tuple, self.index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Range {
    pub start: Expression,
    pub end: Expression,
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ListIndex {
    pub list: Expression,
    pub index: Expression,
}

impl Display for ListIndex {
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
pub struct FieldAccess {
    pub container: Expression,
    pub field: Node<Identifier>,
}

impl Display for FieldAccess {
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
pub enum LiteralExpression {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    String(String),
    Value(Value),
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LiteralExpression::Boolean(boolean) => write!(f, "{}", boolean),
            LiteralExpression::Float(float) => write!(f, "{}", float),
            LiteralExpression::Integer(integer) => write!(f, "{}", integer),
            LiteralExpression::String(string) => write!(f, "{}", string),
            LiteralExpression::Value(value) => write!(f, "{}", value),
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
            (LiteralExpression::Boolean(left), LiteralExpression::Boolean(right)) => {
                left.cmp(right)
            }
            (LiteralExpression::Float(left), LiteralExpression::Float(right)) => {
                left.to_bits().cmp(&right.to_bits())
            }
            (LiteralExpression::Integer(left), LiteralExpression::Integer(right)) => {
                left.cmp(right)
            }
            (LiteralExpression::String(left), LiteralExpression::String(right)) => left.cmp(right),
            (LiteralExpression::Value(left), LiteralExpression::Value(right)) => left.cmp(right),
            _ => unreachable!(),
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
        if_block: Node<Block>,
    },
    IfElse {
        condition: Expression,
        if_block: Node<Block>,
        r#else: ElseExpression,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ElseExpression {
    Block(Node<Block>),
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
pub enum Block {
    Async(Vec<Statement>),
    Sync(Vec<Statement>),
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Block::Async(statements) => {
                writeln!(f, "async {{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        writeln!(f, " ")?;
                    }

                    writeln!(f, "{}", statement)?;
                }

                write!(f, " }}")
            }
            Block::Sync(statements) => {
                writeln!(f, "{{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        writeln!(f, " ")?;
                    }

                    writeln!(f, "{}", statement)?;
                }

                write!(f, " }}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Loop {
    Infinite {
        block: Node<Block>,
    },
    While {
        condition: Expression,
        block: Node<Block>,
    },
    For {
        identifier: Node<Identifier>,
        iterator: Expression,
        block: Node<Block>,
    },
}

impl Display for Loop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Loop::Infinite { block } => write!(f, "loop {}", block),
            Loop::While { condition, block } => write!(f, "while {} {}", condition, block),
            Loop::For {
                identifier,
                iterator,
                block,
            } => write!(f, "for {} in {} {}", identifier, iterator, block),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructExpression {
    // The tuple struct expression is omitted because it is redundant with call expression
    Unit {
        name: Node<Identifier>,
    },
    Fields {
        name: Node<Identifier>,
        fields: Vec<(Node<Identifier>, Expression)>,
    },
}

impl Display for StructExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructExpression::Unit { name } => write!(f, "{}", name),
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
