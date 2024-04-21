use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    Value,
};

use super::{AbstractNode, Action, Block, Expression, Type, WithPos, WithPosition};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Expression>),
    Map(Vec<(Identifier, Option<WithPosition<Type>>, Expression)>),
    Range(Range<i64>),
    String(String),
    Structure {
        name: WithPosition<Identifier>,
        fields: Vec<(Identifier, Expression)>,
    },
    ParsedFunction {
        type_arguments: Vec<WithPosition<Type>>,
        parameters: Vec<(Identifier, WithPosition<Type>)>,
        return_type: WithPosition<Type>,
        body: WithPosition<Block>,
    },
}

impl AbstractNode for ValueNode {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(items) => {
                let mut item_types = Vec::with_capacity(items.len());

                for expression in items {
                    item_types.push(
                        expression
                            .expected_type(context)?
                            .with_position(expression.position()),
                    );
                }

                Type::ListExact(item_types)
            }
            ValueNode::Map(_) => Type::Map,
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::ParsedFunction {
                parameters,
                return_type,
                ..
            } => Type::Function {
                parameter_types: parameters
                    .iter()
                    .map(|(_, r#type)| r#type.clone())
                    .collect(),
                return_type: Box::new(return_type.clone()),
            },
            ValueNode::Structure {
                name,
                fields: expressions,
            } => {
                let mut types = Vec::with_capacity(expressions.len());

                for (identifier, expression) in expressions {
                    let r#type = expression.expected_type(context)?;

                    types.push((
                        identifier.clone(),
                        WithPosition {
                            node: r#type,
                            position: expression.position(),
                        },
                    ));
                }

                Type::Structure {
                    name: name.node.clone(),
                    fields: types,
                }
            }
        };

        Ok(r#type)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let ValueNode::Map(map_assignments) = self {
            for (_identifier, r#type, expression) in map_assignments {
                if let Some(expected_type) = r#type {
                    let actual_type = expression.expected_type(context)?;

                    expected_type.node.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position(),
                            expected_position: expected_type.position,
                        }
                    })?;
                }
            }

            return Ok(());
        }

        if let ValueNode::ParsedFunction {
            type_arguments,
            parameters,
            return_type,
            body,
        } = self
        {
            let function_context = Context::new();

            function_context.inherit_types_from(context)?;

            for r#type in type_arguments {
                if let Type::Argument(identifier) = &r#type.node {
                    function_context.set_type(identifier.clone(), r#type.node.clone())?;
                }
            }

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

            return Ok(());
        }

        if let ValueNode::Structure {
            name,
            fields: expressions,
        } = self
        {
            if !context.contains(&name.node)? {
                return Err(ValidationError::VariableNotFound {
                    identifier: name.node.clone(),
                    position: name.position,
                });
            }

            if let Some(Type::Structure {
                name: _,
                fields: types,
            }) = context.get_type(&name.node)?
            {
                for ((_, expression), (_, expected_type)) in expressions.iter().zip(types.iter()) {
                    let actual_type = expression.expected_type(context)?;

                    expected_type.node.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position(),
                            expected_position: expected_type.position,
                        }
                    })?
                }
            }
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let expression_position = expression.position();
                    let action = expression.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        WithPosition {
                            node: value,
                            position: expression_position,
                        }
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    value_list.push(value);
                }

                Value::list(value_list)
            }
            ValueNode::Map(property_list) => {
                let mut property_map = BTreeMap::new();

                for (identifier, _type, expression) in property_list {
                    let expression_position = expression.position();
                    let action = expression.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    property_map.insert(identifier, value);
                }

                Value::map(property_map)
            }
            ValueNode::Range(range) => Value::range(range),
            ValueNode::String(string) => Value::string(string),
            ValueNode::ParsedFunction {
                type_arguments,
                parameters,
                return_type,
                body,
            } => Value::function(type_arguments, parameters, return_type, body),
            ValueNode::Structure {
                name,
                fields: expressions,
            } => {
                let mut fields = Vec::with_capacity(expressions.len());

                for (identifier, expression) in expressions {
                    let expression_position = expression.position();
                    let action = expression.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    fields.push((identifier, value));
                }

                Value::structure(name, fields)
            }
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
        use self::ValueNode::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left.cmp(right),
            (Boolean(_), _) => Ordering::Greater,
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
                ParsedFunction {
                    type_arguments: left_type_arguments,
                    parameters: left_parameters,
                    return_type: left_return,
                    body: left_body,
                },
                ParsedFunction {
                    type_arguments: right_type_arguments,
                    parameters: right_parameters,
                    return_type: right_return,
                    body: right_body,
                },
            ) => {
                let parameter_cmp = left_parameters.cmp(right_parameters);

                if parameter_cmp.is_eq() {
                    let return_cmp = left_return.cmp(right_return);

                    if return_cmp.is_eq() {
                        let type_argument_cmp = left_type_arguments.cmp(right_type_arguments);

                        if type_argument_cmp.is_eq() {
                            left_body.cmp(right_body)
                        } else {
                            type_argument_cmp
                        }
                    } else {
                        return_cmp
                    }
                } else {
                    parameter_cmp
                }
            }
            (ParsedFunction { .. }, _) => Ordering::Greater,
            (
                Structure {
                    name: left_name,
                    fields: left_fields,
                },
                Structure {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    left_fields.cmp(right_fields)
                } else {
                    name_cmp
                }
            }
            (Structure { .. }, _) => Ordering::Greater,
        }
    }
}
