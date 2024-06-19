use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    Value,
};

use super::{
    Block, Evaluate, Evaluation, ExpectedType, Expression, Type, TypeConstructor, WithPosition,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueNode {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Expression>),
    Map(Vec<(Identifier, Option<TypeConstructor>, Expression)>),
    Range(Range<i64>),
    String(String),
    Structure {
        name: WithPosition<Identifier>,
        fields: Vec<(WithPosition<Identifier>, Expression)>,
    },
    Function {
        type_parameters: Option<Vec<Identifier>>,
        value_parameters: Vec<(Identifier, TypeConstructor)>,
        return_type: TypeConstructor,
        body: WithPosition<Block>,
    },
}

impl Evaluate for ValueNode {
    fn validate(&self, context: &mut Context, _manage_memory: bool) -> Result<(), ValidationError> {
        if let ValueNode::Map(map_assignments) = self {
            for (_identifier, constructor_option, expression) in map_assignments {
                expression.validate(context, _manage_memory)?;

                if let Some(constructor) = constructor_option {
                    let actual_type = expression.expected_type(context)?;
                    let exprected_type = constructor.clone().construct(&context)?;

                    exprected_type.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position(),
                            expected_position: Some(constructor.position()),
                        }
                    })?;
                }
            }

            return Ok(());
        }

        if let ValueNode::Function {
            type_parameters,
            value_parameters,
            return_type,
            body,
        } = self
        {
            let mut function_context = context.create_child();

            if let Some(type_parameters) = type_parameters {
                for identifier in type_parameters {
                    function_context.set_type(
                        identifier.clone(),
                        Type::Generic {
                            identifier: identifier.clone(),
                            concrete_type: None,
                        },
                    )?;
                }
            }

            for (identifier, type_constructor) in value_parameters {
                let r#type = type_constructor.clone().construct(&function_context)?;

                function_context.set_type(identifier.clone(), r#type)?;
            }

            body.node.validate(&mut function_context, _manage_memory)?;

            let actual_return_type = body.node.expected_type(&mut function_context)?;

            return_type
                .clone()
                .construct(&function_context)?
                .check(&actual_return_type)
                .map_err(|conflict| ValidationError::TypeCheck {
                    conflict,
                    actual_position: body.position,
                    expected_position: Some(return_type.position()),
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

                    expected_type.check(&actual_type).map_err(|conflict| {
                        ValidationError::TypeCheck {
                            conflict,
                            actual_position: expression.position(),
                            expected_position: None,
                        }
                    })?
                }
            }
        }

        Ok(())
    }

    fn evaluate(
        self,
        context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let expression_position = expression.position();
                    let evaluation = expression.evaluate(context, _manage_memory)?;
                    let value = if let Evaluation::Return(value) = evaluation {
                        value
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
                    let action = expression.evaluate(context, _manage_memory)?;
                    let value = if let Evaluation::Return(value) = action {
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
            ValueNode::Function {
                type_parameters,
                value_parameters: constructors,
                return_type,
                body,
            } => {
                let type_parameters =
                    type_parameters.map(|parameter_list| parameter_list.into_iter().collect());
                let mut value_parameters = Vec::with_capacity(constructors.len());

                for (identifier, constructor) in constructors {
                    let r#type = constructor.construct(&context)?;

                    value_parameters.push((identifier, r#type));
                }

                let return_type = return_type.construct(&context)?;

                Value::function(type_parameters, value_parameters, return_type, body.node)
            }
            ValueNode::Structure {
                name,
                fields: expressions,
            } => {
                let mut fields = Vec::with_capacity(expressions.len());

                for (identifier, expression) in expressions {
                    let expression_position = expression.position();
                    let action = expression.evaluate(context, _manage_memory)?;
                    let value = if let Evaluation::Return(value) = action {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::InterpreterExpectedReturn(expression_position),
                        ));
                    };

                    fields.push((identifier.node, value));
                }

                Value::structure(name, fields)
            }
        };

        Ok(Evaluation::Return(value))
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
                Function {
                    type_parameters: left_type_arguments,
                    value_parameters: left_parameters,
                    return_type: left_return,
                    body: left_body,
                },
                Function {
                    type_parameters: right_type_arguments,
                    value_parameters: right_parameters,
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
            (Function { .. }, _) => Ordering::Greater,
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

impl ExpectedType for ValueNode {
    fn expected_type(&self, context: &mut Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(items) => {
                let item_type = items.first().unwrap().expected_type(context)?;

                Type::List {
                    length: items.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueNode::Map(_) => Type::Map,
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::Function {
                type_parameters,
                value_parameters,
                return_type,
                ..
            } => {
                let mut value_parameter_types = Vec::with_capacity(value_parameters.len());

                for (_, type_constructor) in value_parameters {
                    let r#type = type_constructor.clone().construct(&context)?;

                    value_parameter_types.push(r#type);
                }

                let type_parameters = type_parameters.clone().map(|parameters| {
                    parameters
                        .iter()
                        .map(|identifier| identifier.clone())
                        .collect()
                });
                let return_type = return_type.clone().construct(&context)?;

                Type::Function {
                    type_parameters,
                    value_parameters: value_parameter_types,
                    return_type: Box::new(return_type),
                }
            }
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
                    fields: types
                        .into_iter()
                        .map(|(identifier, r#type)| (identifier.node, r#type.node))
                        .collect(),
                }
            }
        };

        Ok(r#type)
    }
}
