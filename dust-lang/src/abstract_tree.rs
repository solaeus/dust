//! In-memory representation of a Dust program.
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{BuiltInFunction, Context, Identifier, Span, StructType, Type, Value};

/// In-memory representation of a Dust program.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbstractSyntaxTree {
    pub nodes: VecDeque<Node<Statement>>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node<T> {
    pub inner: T,
    pub position: Span,
}

impl<T> Node<T> {
    pub fn new(inner: T, position: Span) -> Self {
        Self { inner, position }
    }
}

impl<T: Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    Assignment {
        identifier: Node<Identifier>,
        operator: Node<AssignmentOperator>,
        value: Box<Node<Statement>>,
    },

    // A sequence of statements
    Block(Vec<Node<Statement>>),

    // Assignment, logic, math and comparison expressions with two operands
    BinaryOperation {
        left: Box<Node<Statement>>,
        operator: Node<BinaryOperator>,
        right: Box<Node<Statement>>,
    },

    // Logic and math expressions with one operand
    UnaryOperation {
        operator: Node<UnaryOperator>,
        operand: Box<Node<Statement>>,
    },

    // Type definitions
    StructDefinition(StructDefinition),

    // Function calls and type instantiation
    BuiltInFunctionCall {
        function: BuiltInFunction,
        type_arguments: Option<Vec<Node<Statement>>>,
        value_arguments: Option<Vec<Node<Statement>>>,
    },
    Invokation {
        invokee: Box<Node<Statement>>,
        type_arguments: Option<Vec<Node<Statement>>>,
        value_arguments: Option<Vec<Node<Statement>>>,
    },
    StructInstantiation(StructInstantiation),

    // Loops
    While {
        condition: Box<Node<Statement>>,
        body: Box<Node<Statement>>,
    },

    // Control flow
    If {
        condition: Box<Node<Statement>>,
        body: Box<Node<Statement>>,
    },
    IfElse {
        condition: Box<Node<Statement>>,
        if_body: Box<Node<Statement>>,
        else_body: Box<Node<Statement>>,
    },
    IfElseIf {
        condition: Box<Node<Statement>>,
        if_body: Box<Node<Statement>>,
        else_ifs: Vec<(Node<Statement>, Node<Statement>)>,
    },
    IfElseIfElse {
        condition: Box<Node<Statement>>,
        if_body: Box<Node<Statement>>,
        else_ifs: Vec<(Node<Statement>, Node<Statement>)>,
        else_body: Box<Node<Statement>>,
    },

    // Identifier expression
    Identifier(Identifier),

    // Value collection expressions
    List(Vec<Node<Statement>>),
    Map(Vec<(Node<Identifier>, Node<Statement>)>),

    // Hard-coded value
    Constant(Value),

    // A statement that always returns None. Created with a semicolon, it causes the preceding
    // statement to return None. This is analagous to the semicolon or unit type in Rust.
    Nil(Box<Node<Statement>>),
}

impl Statement {
    pub fn expected_type(&self, context: &Context) -> Option<Type> {
        match self {
            Statement::Assignment { .. } => None,
            Statement::Block(nodes) => nodes.last().unwrap().inner.expected_type(context),
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => match operator.inner {
                BinaryOperator::Add
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
                | BinaryOperator::Multiply
                | BinaryOperator::Subtract => Some(left.inner.expected_type(context)?),

                BinaryOperator::Equal
                | BinaryOperator::Greater
                | BinaryOperator::GreaterOrEqual
                | BinaryOperator::Less
                | BinaryOperator::LessOrEqual
                | BinaryOperator::And
                | BinaryOperator::Or => Some(Type::Boolean),

                BinaryOperator::FieldAccess => {
                    let left_type = left.inner.expected_type(context)?;

                    if let Type::Map(properties) = left_type {
                        let key = match &right.inner {
                            Statement::Identifier(identifier) => identifier,
                            _ => return None,
                        };

                        properties.get(key).cloned()
                    } else {
                        None
                    }
                }
                BinaryOperator::ListIndex => {
                    let left_type = left.inner.expected_type(context)?;

                    if let Type::List { item_type, .. } = left_type {
                        Some(*item_type)
                    } else {
                        None
                    }
                }
            },
            Statement::BuiltInFunctionCall { function, .. } => function.expected_return_type(),
            Statement::Constant(value) => Some(value.r#type()),
            Statement::Invokation {
                invokee: function, ..
            } => function.inner.expected_type(context),
            Statement::Identifier(identifier) => context.get_type(identifier),
            Statement::If { .. } => None,
            Statement::IfElse { if_body, .. } => if_body.inner.expected_type(context),
            Statement::IfElseIf { .. } => None,
            Statement::IfElseIfElse { if_body, .. } => if_body.inner.expected_type(context),
            Statement::List(nodes) => {
                let item_type = nodes.first().unwrap().inner.expected_type(context)?;

                Some(Type::List {
                    item_type: Box::new(item_type),
                    length: nodes.len(),
                })
            }
            Statement::Map(nodes) => {
                let mut types = BTreeMap::new();

                for (identifier, item) in nodes {
                    types.insert(identifier.inner.clone(), item.inner.expected_type(context)?);
                }

                Some(Type::Map(types))
            }
            Statement::Nil(_) => None,
            Statement::UnaryOperation { operator, operand } => match operator.inner {
                UnaryOperator::Negate => Some(operand.inner.expected_type(context)?),
                UnaryOperator::Not => Some(Type::Boolean),
            },
            Statement::StructDefinition(_) => None,
            Statement::StructInstantiation(struct_instantiation) => match struct_instantiation {
                StructInstantiation::Tuple { name, fields } => {
                    Some(Type::Struct(StructType::Tuple {
                        name: name.inner.clone(),
                        fields: fields
                            .iter()
                            .map(|field| field.inner.expected_type(context))
                            .collect::<Option<Vec<Type>>>()?,
                    }))
                }
            },
            Statement::While { .. } => None,
        }
    }

    pub fn block_statements_mut(&mut self) -> Option<&mut Vec<Node<Statement>>> {
        match self {
            Statement::Block(statements) => Some(statements),
            _ => None,
        }
    }

    pub fn map_properties_mut(&mut self) -> Option<&mut Vec<(Node<Identifier>, Node<Statement>)>> {
        match self {
            Statement::Map(properties) => Some(properties),
            _ => None,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assignment {
                identifier,
                operator,
                value,
            } => {
                write!(f, "{identifier} {operator} {value}")
            }
            Statement::Block(statements) => {
                write!(f, "{{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "{statement}")?;
                }

                write!(f, " }}")
            }
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let operator = match operator.inner {
                    BinaryOperator::FieldAccess => return write!(f, "{left}.{right}"),
                    BinaryOperator::ListIndex => return write!(f, "{left}[{right}]"),
                    BinaryOperator::Add => "+",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterOrEqual => ">=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessOrEqual => "<=",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };

                write!(f, "{left} {operator} {right}")
            }
            Statement::BuiltInFunctionCall {
                function,
                type_arguments: type_parameters,
                value_arguments: value_parameters,
            } => {
                write!(f, "{function}")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for (i, type_parameter) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{type_parameter}")?;
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (i, value_parameter) in value_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value_parameter}")?;
                    }
                }

                write!(f, ")")
            }
            Statement::Constant(value) => write!(f, "{value}"),
            Statement::Invokation {
                invokee: function,
                type_arguments: type_parameters,
                value_arguments: value_parameters,
            } => {
                write!(f, "{function}")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for (i, type_parameter) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{type_parameter}")?;
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (i, value_parameter) in value_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value_parameter}")?;
                    }
                }

                write!(f, ")")
            }
            Statement::Identifier(identifier) => write!(f, "{identifier}"),
            Statement::If { condition, body } => {
                write!(f, "if {condition} {body}")
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                write!(f, "if {condition} {if_body} else {else_body}")
            }
            Statement::IfElseIf {
                condition,
                if_body,
                else_ifs,
            } => {
                write!(f, "if {condition} {if_body}")?;

                for (condition, body) in else_ifs {
                    write!(f, " else if {condition} {body}")?;
                }

                Ok(())
            }
            Statement::IfElseIfElse {
                condition,
                if_body,
                else_ifs,
                else_body,
            } => {
                write!(f, "if {condition} {if_body}")?;

                for (condition, body) in else_ifs {
                    write!(f, " else if {condition} {body}")?;
                }

                write!(f, " else {else_body}")
            }
            Statement::List(nodes) => {
                write!(f, "[")?;

                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{node}")?;
                }

                write!(f, "]")
            }
            Statement::Map(nodes) => {
                write!(f, "{{")?;

                for (i, (identifier, node)) in nodes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{identifier} = {node}")?;
                }

                write!(f, "}}")
            }
            Statement::Nil(node) => write!(f, "{node};"),
            Statement::UnaryOperation { operator, operand } => {
                let operator = match operator.inner {
                    UnaryOperator::Negate => "-",
                    UnaryOperator::Not => "!",
                };

                write!(f, "{operator}{operand}")
            }
            Statement::StructDefinition(struct_definition) => {
                write!(f, "{struct_definition}")
            }
            Statement::StructInstantiation(struct_instantiation) => {
                write!(f, "{struct_instantiation}")
            }
            Statement::While { condition, body } => {
                write!(f, "while {condition} {body}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
}

impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let operator = match self {
            AssignmentOperator::Assign => "=",
            AssignmentOperator::AddAssign => "+=",
            AssignmentOperator::SubtractAssign => "-=",
        };

        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Accessors
    FieldAccess,
    ListIndex,

    // Math
    Add,
    Divide,
    Modulo,
    Multiply,
    Subtract,

    // Comparison
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,

    // Logic
    And,
    Or,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructDefinition {
    Unit {
        name: Node<Identifier>,
    },
    Tuple {
        name: Node<Identifier>,
        fields: Vec<Node<Type>>,
    },
}

impl Display for StructDefinition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructDefinition::Unit { name } => write!(f, "struct {name}"),
            StructDefinition::Tuple { name, fields } => {
                write!(f, "struct {name} {{")?;

                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{field}")?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructInstantiation {
    // The Unit variant is absent because unit structs are instantiated without any fields
    Tuple {
        name: Node<Identifier>,
        fields: Vec<Node<Statement>>,
    },
}

impl Display for StructInstantiation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructInstantiation::Tuple { name, fields } => {
                write!(f, "{name}(")?;

                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{field}")?;
                }

                write!(f, ")")
            }
        }
    }
}
