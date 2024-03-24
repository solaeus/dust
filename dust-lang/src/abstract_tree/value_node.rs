use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{
    built_in_functions::BuiltInFunction,
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractNode, Action, Block, Expression, Identifier, Type, WithPos, WithPosition};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode {
    Boolean(bool),
    BuiltInFunction(BuiltInFunction),
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
    Structure {
        name: Identifier,
        fields: Vec<(Identifier, WithPosition<Expression>)>,
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
                            .node
                            .expected_type(context)?
                            .with_position(expression.position),
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
                    let r#type = expression.node.expected_type(context)?;

                    types.push((
                        identifier.clone(),
                        WithPosition {
                            node: r#type,
                            position: expression.position,
                        },
                    ));
                }

                Type::Structure {
                    name: name.clone(),
                    fields: types,
                }
            }
            ValueNode::BuiltInFunction(built_in_function) => built_in_function.r#type(),
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
            if let Type::Structure {
                name: _,
                fields: types,
            } = name.expected_type(context)?
            {
                for ((_, expression), (_, expected_type)) in expressions.iter().zip(types.iter()) {
                    let actual_type = expression.node.expected_type(context)?;

                    expected_type.node.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position,
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
                    let action = expression.node.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        WithPosition {
                            node: value,
                            position: expression.position,
                        }
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
                    let action = expression.node.run(_context)?;
                    let value = if let Action::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression.position),
                        ));
                    };

                    fields.push((identifier, value));
                }

                Value::structure(name, fields)
            }
            ValueNode::BuiltInFunction(built_in_function) => {
                Value::built_in_function(built_in_function)
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
            (BuiltInFunction(_), BuiltInFunction(_)) => todo!(),
            (BuiltInFunction(_), _) => todo!(),
        }
    }
}
