use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Function, Identifier, Map, Result, Statement, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    type_definition: Option<TypeDefinition>,
    operator: AssignmentOperator,
    statement: Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
}

impl AbstractTree for Assignment {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "assignment", node)?;

        let identifier_node = node.child_by_field_name("identifier").unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node, context)?;

        let type_node = node.child_by_field_name("type");
        let type_definition = if let Some(type_node) = type_node {
            Some(TypeDefinition::from_syntax_node(
                source, type_node, context,
            )?)
        } else {
            None
        };

        let operator_node = node
            .child_by_field_name("assignment_operator")
            .unwrap()
            .child(0)
            .unwrap();
        let operator = match operator_node.kind() {
            "=" => AssignmentOperator::Equal,
            "+=" => AssignmentOperator::PlusEqual,
            "-=" => AssignmentOperator::MinusEqual,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "=, += or -=",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let statement_node = node.child_by_field_name("statement").unwrap();
        let statement = Statement::from_syntax_node(source, statement_node, context)?;

        if let Some(type_definition) = &type_definition {
            let statement_type = statement.expected_type(context)?;

            match operator {
                AssignmentOperator::Equal => {
                    type_definition.abstract_check(
                        &statement_type,
                        context,
                        statement_node,
                        source,
                    )?;
                }
                AssignmentOperator::PlusEqual => {
                    let identifier_type = identifier.expected_type(context)?;

                    type_definition.abstract_check(
                        &identifier_type,
                        context,
                        type_node.unwrap(),
                        source,
                    )?;

                    let type_definition = if let Type::List(item_type) = type_definition.inner() {
                        TypeDefinition::new(item_type.as_ref().clone())
                    } else {
                        type_definition.clone()
                    };

                    type_definition.abstract_check(
                        &statement_type,
                        context,
                        statement_node,
                        source,
                    )?;
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        }

        Ok(Assignment {
            identifier,
            type_definition,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let key = self.identifier.inner();
        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(mut previous_value) = context.variables()?.get(key).cloned() {
                    if let Ok(list) = previous_value.as_list() {
                        let item_type = if let Some(type_defintion) = &self.type_definition {
                            if let Type::List(item_type) = type_defintion.inner() {
                                TypeDefinition::new(item_type.as_ref().clone())
                            } else {
                                TypeDefinition::new(Type::Empty)
                            }
                        } else if let Some(first) = list.items().first() {
                            first.r#type(context)?
                        } else {
                            TypeDefinition::new(Type::Any)
                        };

                        item_type.runtime_check(&value.r#type(context)?, context)?;
                    }

                    previous_value += value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(mut previous_value) = context.variables()?.get(key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::Equal => value,
        };

        let new_value = if let Some(type_definition) = &self.type_definition {
            let new_value_type = new_value.r#type(context)?;

            type_definition.runtime_check(&new_value_type, context)?;

            if let Value::Function(function) = new_value {
                Value::Function(Function::new(
                    function.parameters().clone(),
                    function.body().clone(),
                ))
            } else {
                new_value
            }
        } else {
            new_value
        };

        context.variables_mut()?.insert(key.clone(), new_value);

        Ok(Value::Empty)
    }

    fn expected_type(&self, _context: &Map) -> Result<TypeDefinition> {
        Ok(TypeDefinition::new(Type::Empty))
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, List, Value};

    #[test]
    fn simple_assignment() {
        let test = evaluate("x = 1 x").unwrap();

        assert_eq!(Value::Integer(1), test);
    }

    #[test]
    fn simple_assignment_with_type() {
        let test = evaluate("x <int> = 1 x").unwrap();

        assert_eq!(Value::Integer(1), test);
    }

    #[test]
    fn list_add_assign() {
        let test = evaluate(
            "
            x <list int> = []
            x += 1
            x
        ",
        )
        .unwrap();

        assert_eq!(Value::List(List::with_items(vec![Value::Integer(1)])), test);
    }

    #[test]
    fn function_assignment() {
        let test = evaluate(
            "
            foobar <fn str -> str> = |text| { 'hi' }
            foobar
        ",
        )
        .unwrap();

        assert_eq!(Value::String("hi".to_string()), test);
    }
}
