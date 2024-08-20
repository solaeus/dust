use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Context, ContextError, Identifier, Type};

use super::{AstError, Expression, Node, Span};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    ExpressionNullified(Node<Expression>),
    Let(Node<LetStatement>),
    StructDefinition(Node<StructDefinition>),
}

impl Statement {
    pub fn struct_definition(struct_definition: StructDefinition, position: Span) -> Self {
        Statement::StructDefinition(Node::new(struct_definition, position))
    }

    pub fn position(&self) -> Span {
        match self {
            Statement::Expression(expression) => expression.position(),
            Statement::ExpressionNullified(expression_node) => expression_node.position,
            Statement::Let(r#let) => r#let.position,
            Statement::StructDefinition(definition) => definition.position,
        }
    }

    pub fn return_type<'recovered>(
        &self,
        context: &'recovered Context,
    ) -> Result<Option<Type>, AstError> {
        match self {
            Statement::Expression(expression) => expression.return_type(context),
            Statement::ExpressionNullified(_) => Ok(None),
            Statement::Let(_) => Ok(None),
            Statement::StructDefinition(_) => Ok(None),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::Expression(expression) => write!(f, "{}", expression),
            Statement::ExpressionNullified(expression) => write!(f, "{};", expression),
            Statement::Let(r#let) => write!(f, "{};", r#let),
            Statement::StructDefinition(struct_definition) => match &struct_definition.inner {
                StructDefinition::Unit { name } => write!(f, "struct {};", name),
                StructDefinition::Tuple { name, items } => {
                    write!(f, "struct {name} {{ ")?;

                    for (index, item) in items.iter().enumerate() {
                        write!(f, "{}: {}", item, index)?;

                        if index < items.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }

                    write!(f, " }}")
                }
                StructDefinition::Fields { name, fields } => {
                    write!(f, "struct {name} {{ ")?;

                    for (index, (field, r#type)) in fields.iter().enumerate() {
                        write!(f, "{}: {}", field, r#type)?;

                        if index < fields.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }

                    write!(f, " }}")
                }
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LetStatement {
    Let {
        identifier: Node<Identifier>,
        value: Expression,
    },
    LetMut {
        identifier: Node<Identifier>,
        value: Expression,
    },
    LetType {
        identifier: Node<Identifier>,
        r#type: Node<Type>,
        value: Expression,
    },
    LetMutType {
        identifier: Node<Identifier>,
        r#type: Node<Type>,
        value: Expression,
    },
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LetStatement::Let { identifier, value } => {
                write!(f, "let {identifier} = {value}")
            }
            LetStatement::LetMut { identifier, value } => {
                write!(f, "let mut {identifier} = {value}")
            }
            LetStatement::LetType {
                identifier,
                r#type,
                value,
            } => {
                write!(f, "let {identifier}: {type} = {value}")
            }
            LetStatement::LetMutType {
                identifier,
                r#type,
                value,
            } => {
                write!(f, "let mut {identifier}: {type} = {value}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructDefinition {
    Unit {
        name: Node<Identifier>,
    },
    Tuple {
        name: Node<Identifier>,
        items: Vec<Node<Type>>,
    },
    Fields {
        name: Node<Identifier>,
        fields: Vec<(Node<Identifier>, Node<Type>)>,
    },
}

impl Display for StructDefinition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructDefinition::Unit { name } => write!(f, "struct {name}"),
            StructDefinition::Tuple {
                name,
                items: fields,
            } => {
                write!(f, "struct {name} {{")?;

                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{field}")?;
                }

                write!(f, "}}")
            }
            StructDefinition::Fields { name, fields } => {
                write!(f, "struct {name} {{")?;

                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{field_name}: {field_type}")?;
                }

                write!(f, "}}")
            }
        }
    }
}
