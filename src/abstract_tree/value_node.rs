use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::EnumInstance,
    Value,
};

use super::{AbstractTree, Action, Block, Expression, Identifier, Type, WithPosition};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode {
    Boolean(bool),
    EnumInstance {
        name: Identifier,
        variant: Identifier,
        type_arguments: Option<Vec<WithPosition<Type>>>,
        expression: Box<WithPosition<Expression>>,
    },
    Float(f64),
    Integer(i64),
    List(Vec<WithPosition<Expression>>),
    Map(
        Vec<(
            Identifier,
            Option<WithPosition<Type>>,
            WithPosition<Expression>,
        )>,
    ),
    Range(Range<i64>),
    String(String),
    Function {
        parameters: Vec<(Identifier, WithPosition<Type>)>,
        return_type: WithPosition<Type>,
        body: WithPosition<Block>,
    },
}

impl AbstractTree for ValueNode {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,

            ValueNode::EnumInstance {
                name,
                variant: _,
                type_arguments,
                expression: _,
            } => {
                if let Some(r#type) = context.get_type(name)? {
                    r#type
                } else {
                    Type::None
                }
            }
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(items) => {
                let mut item_types = Vec::with_capacity(items.len());

                for expression in items {
                    item_types.push(expression.node.expected_type(context)?);
                }

                Type::ListExact(item_types)
            }
            ValueNode::Map(_) => Type::Map,
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::Function {
                parameters,
                return_type,
                ..
            } => Type::Function {
                parameter_types: parameters
                    .into_iter()
                    .map(|(_, r#type)| r#type.node.clone())
                    .collect(),
                return_type: Box::new(return_type.node.clone()),
            },
        };

        Ok(r#type)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let ValueNode::Map(map_assignments) = self {
            for (_identifier, r#type, expression) in map_assignments {
                if let Some(expected_type) = r#type {
                    let actual_type = expression.node.expected_type(context)?;

                    expected_type.node.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position,
                            expected_position: expected_type.position,
                        }
                    })?;
                }
            }
        }

        if let ValueNode::Function {
            parameters,
            return_type,
            body,
        } = self
        {
            let function_context = Context::new();

            function_context.inherit_types_from(context)?;

            for (identifier, r#type) in parameters {
                function_context.set_type(identifier.clone(), r#type.node.clone())?;
            }

            body.node.validate(&function_context)?;

            let actual_return_type = body.node.expected_type(&function_context)?;

            return_type
                .node
                .check(&actual_return_type)
                .map_err(|conflict| ValidationError::TypeCheck {
                    conflict,
                    actual_position: body.position,
                    expected_position: return_type.position,
                })?;
        }

        if let ValueNode::EnumInstance {
            name,
            variant,
            type_arguments,
            expression,
        } = self
        {
            let r#type = self.expected_type(context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::EnumInstance {
                name,
                variant,
                type_arguments: _,
                expression,
            } => {
                let action = expression.node.run(_context)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    todo!()
                };

                Value::enum_instance(EnumInstance::new(name, variant, value))
            }
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let action = expression.node.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression.position),
                        ));
                    };

                    value_list.push(value);
                }

                Value::list(value_list)
            }
            ValueNode::Map(property_list) => {
                let mut property_map = BTreeMap::new();

                for (identifier, _type, expression) in property_list {
                    let action = expression.node.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression.position),
                        ));
                    };

                    property_map.insert(identifier, value);
                }

                Value::map(property_map)
            }
            ValueNode::Range(range) => Value::range(range),
            ValueNode::String(string) => Value::string(string),
            ValueNode::Function {
                parameters,
                return_type,
                body,
            } => Value::function(parameters, return_type, body),
        };

        Ok(Action::Return(value))
    }
}

impl Eq for ValueNode {}

impl PartialOrd for ValueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        use ValueNode::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left.cmp(right),
            (Boolean(_), _) => Ordering::Greater,
            (
                EnumInstance {
                    name: left_name,
                    variant: left_variant,
                    type_arguments: left_types,
                    expression: left_expression,
                },
                EnumInstance {
                    name: right_name,
                    variant: right_variant,
                    type_arguments: right_types,
                    expression: right_expression,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    let variant_cmp = left_variant.cmp(right_variant);

                    if variant_cmp.is_eq() {
                        let type_cmp = left_types.cmp(right_types);

                        if type_cmp.is_eq() {
                            left_expression.cmp(right_expression)
                        } else {
                            type_cmp
                        }
                    } else {
                        variant_cmp
                    }
                } else {
                    name_cmp
                }
            }
            (EnumInstance { .. }, _) => Ordering::Greater,
            (Float(left), Float(right)) => left.total_cmp(right),
            (Float(_), _) => Ordering::Greater,
            (Integer(left), Integer(right)) => left.cmp(right),
            (Integer(_), _) => Ordering::Greater,
            (List(left), List(right)) => left.cmp(right),
            (List(_), _) => Ordering::Greater,
            (Map(left), Map(right)) => left.cmp(right),
            (Map(_), _) => Ordering::Greater,
            (Range(left), Range(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp.is_eq() {
                    left.end.cmp(&right.end)
                } else {
                    start_cmp
                }
            }
            (Range(_), _) => Ordering::Greater,
            (String(left), String(right)) => left.cmp(right),
            (String(_), _) => Ordering::Greater,
            (
                Function {
                    parameters: left_parameters,
                    return_type: left_return,
                    body: left_body,
                },
                Function {
                    parameters: right_parameters,
                    return_type: right_return,
                    body: right_body,
                },
            ) => {
                let parameter_cmp = left_parameters.cmp(right_parameters);

                if parameter_cmp.is_eq() {
                    let return_cmp = left_return.cmp(right_return);

                    if return_cmp.is_eq() {
                        left_body.cmp(right_body)
                    } else {
                        return_cmp
                    }
                } else {
                    parameter_cmp
                }
            }
            (Function { .. }, _) => Ordering::Greater,
        }
    }
}
