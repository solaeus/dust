use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Context, Identifier, Span, Type};

use super::{Expression, Node};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    ExpressionNullified(Node<Expression>),
    Let(Node<Let>),
    StructDefinition(Node<StructDefinition>),
}

impl Statement {
    pub fn struct_definition(struct_definition: StructDefinition, position: Span) -> Self {
        Statement::StructDefinition(Node::new(struct_definition, position))
    }

    pub fn return_type(&self, context: &Context) -> Option<Type> {
        match self {
            Statement::Expression(expression) => expression.return_type(context),
            Statement::ExpressionNullified(expression_node) => {
                expression_node.inner.return_type(context)
            }
            Statement::Let(_) => None,
            Statement::StructDefinition(_) => None,
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Statement::Expression(expression) => expression.position(),
            Statement::ExpressionNullified(expression_node) => expression_node.position,
            Statement::Let(r#let) => r#let.position,
            Statement::StructDefinition(definition) => definition.position,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::Expression(expression) => write!(f, "{}", expression),
            Statement::ExpressionNullified(expression) => write!(f, "{}", expression),
            Statement::Let(r#let) => write!(f, "{}", r#let),
            Statement::StructDefinition(definition) => write!(f, "{}", definition),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Let {
    pub identifier: Node<Identifier>,
    pub value: Node<Expression>,
}

impl Display for Let {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "let {} = {}", self.identifier, self.value)
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
