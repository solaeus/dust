use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Span, Value};

use super::{Node, Statement};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Expression {
    WithBlock(Node<Box<ExpressionWithBlock>>),
    WithoutBlock(Node<Box<ExpressionWithoutBlock>>),
}

impl Expression {
    pub fn operator(operator_expression: OperatorExpression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(operator_expression)),
            position,
        ))
    }

    pub fn range(start: Expression, end: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Range(Range { start, end })),
            position,
        ))
    }

    pub fn call(invoker: Expression, arguments: Vec<Expression>, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Call(CallExpression {
                invoker,
                arguments,
            })),
            position,
        ))
    }

    pub fn field_access(container: Expression, field: Node<Identifier>, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::FieldAccess(FieldAccess {
                container,
                field,
            })),
            position,
        ))
    }

    pub fn tuple_access(tuple: Expression, index: Node<usize>, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::TupleAccess(TupleAccess {
                tuple,
                index,
            })),
            position,
        ))
    }

    pub fn assignment(assignee: Expression, value: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::Assignment { assignee, value },
            )),
            position,
        ))
    }

    pub fn comparison(
        left: Expression,
        operator: Node<ComparisonOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::Comparison {
                    left,
                    operator,
                    right,
                },
            )),
            position,
        ))
    }

    pub fn compound_assignment(
        assignee: Expression,
        operator: Node<MathOperator>,
        modifier: Expression,
        position: Span,
    ) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::CompoundAssignment {
                    assignee,
                    operator,
                    modifier,
                },
            )),
            position,
        ))
    }

    pub fn math(
        left: Expression,
        operator: Node<MathOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(OperatorExpression::Math {
                left,
                operator,
                right,
            })),
            position,
        ))
    }

    pub fn negation(expression: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::Negation(expression),
            )),
            position,
        ))
    }

    pub fn not(expression: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(OperatorExpression::Not(
                expression,
            ))),
            position,
        ))
    }

    pub fn logic(
        left: Expression,
        operator: Node<LogicOperator>,
        right: Expression,
        position: Span,
    ) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::Logic {
                    left,
                    operator,
                    right,
                },
            )),
            position,
        ))
    }

    pub fn error_propagation(expression: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Operator(
                OperatorExpression::ErrorPropagation(expression),
            )),
            position,
        ))
    }

    pub fn infinite_loop(block: Node<Block>, position: Span) -> Self {
        Expression::WithBlock(Node::new(
            Box::new(ExpressionWithBlock::Loop(Loop::Infinite { block })),
            position,
        ))
    }

    pub fn while_loop(condition: Expression, block: Node<Block>, position: Span) -> Self {
        Expression::WithBlock(Node::new(
            Box::new(ExpressionWithBlock::Loop(Loop::While { condition, block })),
            position,
        ))
    }

    pub fn for_loop(
        identifier: Node<Identifier>,
        iterator: Expression,
        block: Node<Block>,
        position: Span,
    ) -> Self {
        Expression::WithBlock(Node::new(
            Box::new(ExpressionWithBlock::Loop(Loop::For {
                identifier,
                iterator,
                block,
            })),
            position,
        ))
    }

    pub fn block(block: Block, position: Span) -> Self {
        Expression::WithBlock(Node::new(
            Box::new(ExpressionWithBlock::Block(block)),
            position,
        ))
    }

    pub fn grouped(expression: Expression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Grouped(expression)),
            position,
        ))
    }

    pub fn r#struct(struct_expression: StructExpression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Struct(struct_expression)),
            position,
        ))
    }

    pub fn identifier(identifier: Identifier, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Identifier(identifier)),
            position,
        ))
    }

    pub fn list(list_expression: ListExpression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::List(list_expression)),
            position,
        ))
    }

    pub fn list_index(list_index: ListIndex, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::ListIndex(list_index)),
            position,
        ))
    }

    pub fn r#if(r#if: If, position: Span) -> Self {
        Expression::WithBlock(Node::new(Box::new(ExpressionWithBlock::If(r#if)), position))
    }

    pub fn literal(literal: LiteralExpression, position: Span) -> Self {
        Expression::WithoutBlock(Node::new(
            Box::new(ExpressionWithoutBlock::Literal(literal)),
            position,
        ))
    }

    pub fn as_identifier(&self) -> Option<&Identifier> {
        if let Expression::WithoutBlock(Node {
            inner: expression_without_block,
            ..
        }) = self
        {
            if let ExpressionWithoutBlock::Identifier(identifier) =
                expression_without_block.as_ref()
            {
                Some(identifier)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Expression::WithBlock(expression_node) => expression_node.position,
            Expression::WithoutBlock(expression_node) => expression_node.position,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::WithBlock(expression) => write!(f, "{}", expression),
            Expression::WithoutBlock(expression) => write!(f, "{}", expression),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExpressionWithBlock {
    Block(Block),
    Loop(Loop),
    If(If),
}

impl Display for ExpressionWithBlock {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ExpressionWithBlock::Block(block) => write!(f, "{}", block),
            ExpressionWithBlock::Loop(r#loop) => write!(f, "{}", r#loop),
            ExpressionWithBlock::If(r#if) => write!(f, "{}", r#if),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExpressionWithoutBlock {
    Call(CallExpression),
    List(ListExpression),
    Literal(LiteralExpression),
    Identifier(Identifier),
    Operator(OperatorExpression),
    Struct(StructExpression),
    Grouped(Expression),
    FieldAccess(FieldAccess),
    ListIndex(ListIndex),
    Range(Range),
    TupleAccess(TupleAccess),
}

impl Display for ExpressionWithoutBlock {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ExpressionWithoutBlock::Call(call_expression) => write!(f, "{}", call_expression),
            ExpressionWithoutBlock::List(list) => write!(f, "{}", list),
            ExpressionWithoutBlock::Literal(literal) => write!(f, "{}", literal),
            ExpressionWithoutBlock::Identifier(identifier) => write!(f, "{}", identifier),
            ExpressionWithoutBlock::Operator(expression) => write!(f, "{}", expression),
            ExpressionWithoutBlock::Struct(struct_expression) => write!(f, "{}", struct_expression),
            ExpressionWithoutBlock::Grouped(expression) => write!(f, "({})", expression),
            ExpressionWithoutBlock::FieldAccess(field_access) => write!(f, "{}", field_access),
            ExpressionWithoutBlock::ListIndex(list_index) => write!(f, "{}", list_index),
            ExpressionWithoutBlock::Range(range) => write!(f, "{}", range),
            ExpressionWithoutBlock::TupleAccess(tuple_access) => write!(f, "{}", tuple_access),
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
pub enum If {
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
    If(Box<If>),
}

impl Display for ElseExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ElseExpression::Block(block) => write!(f, "{}", block),
            ElseExpression::If(r#if) => write!(f, "{}", r#if),
        }
    }
}

impl Display for If {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            If::If {
                condition,
                if_block,
            } => {
                write!(f, "if {} {}", condition, if_block)
            }
            If::IfElse {
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
