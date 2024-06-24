use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    Value,
};

use super::{
    AbstractNode, Block, BuiltInFunction, Evaluation, Expression, Type, TypeConstructor, WithPos,
    WithPosition,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueNode {
    Boolean(bool),
    BuiltInFunction(BuiltInFunction),
    EnumInstance {
        type_name: WithPosition<Identifier>,
        variant: WithPosition<Identifier>,
        content: Option<Vec<Expression>>,
    },
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
    Function(FunctionNode),
}

impl ValueNode {
    pub fn function(
        type_parameters: Option<Vec<Identifier>>,
        value_parameters: Option<Vec<(Identifier, TypeConstructor)>>,
        return_type: Option<TypeConstructor>,
        body: WithPosition<Block>,
    ) -> Self {
        ValueNode::Function(FunctionNode {
            type_parameters,
            value_parameters,
            return_type,
            body,
            context_template: Context::new(None),
        })
    }
}

impl AbstractNode for ValueNode {
    fn define_types(&self, outer_context: &Context) -> Result<(), ValidationError> {
        match self {
            ValueNode::EnumInstance { content, .. } => {
                if let Some(expressions) = content {
                    for expression in expressions {
                        expression.define_types(outer_context)?;
                    }
                }
            }
            ValueNode::List(expressions) => {
                for expression in expressions {
                    expression.define_types(outer_context)?;
                }
            }
            ValueNode::Map(fields) => {
                for (_, _, expression) in fields {
                    expression.define_types(outer_context)?;
                }
            }
            ValueNode::Structure { fields, .. } => {
                for (_, expression) in fields {
                    expression.define_types(outer_context)?;
                }
            }
            ValueNode::Function(FunctionNode {
                body,
                type_parameters,
                value_parameters,
                context_template,
                ..
            }) => {
                context_template.set_parent(outer_context.clone())?;

                if let Some(type_parameters) = type_parameters {
                    for identifier in type_parameters {
                        context_template.set_type(
                            identifier.clone(),
                            Type::Generic {
                                identifier: identifier.clone(),
                                concrete_type: None,
                            },
                        )?;
                    }
                }

                if let Some(value_parameters) = value_parameters {
                    for (identifier, type_constructor) in value_parameters {
                        let r#type = type_constructor.clone().construct(outer_context)?;

                        context_template.set_type(identifier.clone(), r#type)?;
                    }
                }

                body.node.define_types(context_template)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn validate(&self, context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        if let ValueNode::EnumInstance {
            type_name, variant, ..
        } = self
        {
            if let Some(Type::Enum { variants, .. }) = context.get_type(&type_name.node)? {
                if variants
                    .iter()
                    .find(|(identifier, _)| identifier == &variant.node)
                    .is_none()
                {
                    return Err(ValidationError::EnumVariantNotFound {
                        identifier: variant.node.clone(),
                        position: variant.position,
                    });
                }
            } else {
                return Err(ValidationError::EnumDefinitionNotFound {
                    identifier: type_name.node.clone(),
                    position: Some(type_name.position),
                });
            }
        }

        if let ValueNode::Map(map_assignments) = self {
            for (_identifier, constructor_option, expression) in map_assignments {
                expression.validate(context, _manage_memory)?;

                if let Some(constructor) = constructor_option {
                    let actual_type = if let Some(r#type) = expression.expected_type(context)? {
                        r#type
                    } else {
                        return Err(ValidationError::ExpectedExpression(expression.position()));
                    };
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

        if let ValueNode::Function(FunctionNode {
            return_type,
            body,
            context_template,
            ..
        }) = self
        {
            body.node.validate(&context_template, _manage_memory)?;

            let ((expected_return, expected_position), actual_return) =
                match (return_type, body.node.expected_type(&context_template)?) {
                    (Some(constructor), Some(r#type)) => (
                        (
                            constructor.construct(context_template)?,
                            constructor.position(),
                        ),
                        r#type,
                    ),
                    (None, Some(_)) => return Err(ValidationError::ExpectedValue(body.position)),
                    (Some(constructor), None) => {
                        return Err(ValidationError::ExpectedExpression(constructor.position()))
                    }
                    (None, None) => return Ok(()),
                };

            expected_return.check(&actual_return).map_err(|conflict| {
                ValidationError::TypeCheck {
                    conflict,
                    actual_position: body.position,
                    expected_position: Some(expected_position),
                }
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
                    let actual_type = if let Some(r#type) = expression.expected_type(context)? {
                        r#type
                    } else {
                        return Err(ValidationError::ExpectedExpression(expression.position()));
                    };

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
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::EnumInstance {
                type_name,
                variant,
                content: expressions,
            } => {
                let content = if let Some(expressions) = expressions {
                    let mut values = Vec::with_capacity(expressions.len());

                    for expression in expressions {
                        let position = expression.position();
                        let evaluation = expression.evaluate(context, manage_memory)?;

                        if let Some(Evaluation::Return(value)) = evaluation {
                            values.push(value);
                        } else {
                            return Err(RuntimeError::ValidationFailure(
                                ValidationError::ExpectedExpression(position),
                            ));
                        }
                    }
                    Some(values)
                } else {
                    None
                };

                Value::enum_instance(type_name.node, variant.node, content)
            }
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let position = expression.position();
                    let evaluation = expression.evaluate(context, manage_memory)?;
                    let value = if let Some(Evaluation::Return(value)) = evaluation {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedExpression(position),
                        ));
                    };

                    value_list.push(value);
                }

                Value::list(value_list)
            }
            ValueNode::Map(property_list) => {
                let mut property_map = BTreeMap::new();

                for (identifier, _type, expression) in property_list {
                    let position = expression.position();
                    let evaluation = expression.evaluate(context, manage_memory)?;
                    let value = if let Some(Evaluation::Return(value)) = evaluation {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedExpression(position),
                        ));
                    };

                    property_map.insert(identifier, value);
                }

                Value::map(property_map)
            }
            ValueNode::Range(range) => Value::range(range),
            ValueNode::String(string) => Value::string(string),
            ValueNode::Function(FunctionNode {
                type_parameters,
                value_parameters,
                return_type,
                body,
                context_template,
            }) => {
                let outer_context = context;
                let value_parameters = if let Some(value_parameters) = value_parameters {
                    let mut parameters = Vec::with_capacity(value_parameters.len());

                    for (identifier, constructor) in value_parameters {
                        let r#type = constructor.construct(&outer_context)?;

                        parameters.push((identifier, r#type));
                    }

                    Some(parameters)
                } else {
                    None
                };
                let return_type = if let Some(constructor) = return_type {
                    Some(constructor.construct(&outer_context)?)
                } else {
                    None
                };

                Value::function(
                    type_parameters,
                    value_parameters,
                    return_type,
                    body.node,
                    context_template,
                )
            }
            ValueNode::Structure {
                name,
                fields: expressions,
            } => {
                let mut fields = Vec::with_capacity(expressions.len());

                for (identifier, expression) in expressions {
                    let position = expression.position();
                    let evaluation = expression.evaluate(context, manage_memory)?;
                    let value = if let Some(Evaluation::Return(value)) = evaluation {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedExpression(position),
                        ));
                    };

                    fields.push((identifier.node, value));
                }

                Value::structure(name, fields)
            }
            ValueNode::BuiltInFunction(function) => Value::built_in_function(function),
        };

        Ok(Some(Evaluation::Return(value)))
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::EnumInstance { type_name, .. } => {
                if let Some(r#type) = context.get_type(&type_name.node)? {
                    r#type
                } else {
                    return Err(ValidationError::EnumDefinitionNotFound {
                        identifier: type_name.node.clone(),
                        position: Some(type_name.position),
                    });
                }
            }
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(expressions) => {
                let first_item = expressions.first().unwrap();
                let item_type = if let Some(r#type) = first_item.expected_type(context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedExpression(first_item.position()));
                };

                Type::List {
                    length: expressions.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueNode::Map(fields) => {
                let mut field_types = BTreeMap::new();

                for (identifier, constructor_option, expression) in fields {
                    let r#type = if let Some(constructor) = constructor_option {
                        constructor.construct(context)?
                    } else {
                        if let Some(r#type) = expression.expected_type(context)? {
                            r#type
                        } else {
                            return Err(ValidationError::CannotAssignToNone(expression.position()));
                        }
                    };

                    field_types.insert(identifier.clone(), r#type);
                }

                Type::Map(field_types)
            }
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::Function(FunctionNode {
                type_parameters,
                value_parameters,
                return_type,
                ..
            }) => {
                let value_parameters = if let Some(value_parameters) = value_parameters {
                    let mut parameters = Vec::with_capacity(value_parameters.len());

                    for (identifier, type_constructor) in value_parameters {
                        let r#type = type_constructor.clone().construct(&context)?;

                        parameters.push((identifier.clone(), r#type));
                    }

                    Some(parameters)
                } else {
                    None
                };

                let type_parameters = type_parameters.clone().map(|parameters| {
                    parameters
                        .iter()
                        .map(|identifier| identifier.clone())
                        .collect()
                });
                let return_type = if let Some(constructor) = return_type {
                    Some(Box::new(constructor.construct(context)?))
                } else {
                    None
                };

                Type::Function {
                    type_parameters,
                    value_parameters,
                    return_type,
                }
            }
            ValueNode::Structure {
                name,
                fields: expressions,
            } => {
                let mut types = Vec::with_capacity(expressions.len());

                for (identifier, expression) in expressions {
                    let r#type = if let Some(r#type) = expression.expected_type(context)? {
                        r#type
                    } else {
                        return Err(ValidationError::ExpectedExpression(expression.position()));
                    };

                    types.push((
                        identifier.clone(),
                        r#type.with_position(expression.position()),
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
            ValueNode::BuiltInFunction(built_in_function) => built_in_function.r#type(),
        };

        Ok(Some(r#type))
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
                EnumInstance {
                    type_name: left_name,
                    variant: left_variant,
                    content: left_content,
                },
                EnumInstance {
                    type_name: right_name,
                    variant: right_variant,
                    content: right_content,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    let variant_cmp = left_variant.cmp(right_variant);

                    if variant_cmp.is_eq() {
                        left_content.cmp(right_content)
                    } else {
                        variant_cmp
                    }
                } else {
                    name_cmp
                }
            }
            (EnumInstance { .. }, _) => Ordering::Greater,
            (
                Function(FunctionNode {
                    type_parameters: left_type_arguments,
                    value_parameters: left_parameters,
                    return_type: left_return,
                    body: left_body,
                    ..
                }),
                Function(FunctionNode {
                    type_parameters: right_type_arguments,
                    value_parameters: right_parameters,
                    return_type: right_return,
                    body: right_body,
                    ..
                }),
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
            (BuiltInFunction(left), BuiltInFunction(right)) => left.cmp(right),
            (BuiltInFunction(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionNode {
    type_parameters: Option<Vec<Identifier>>,
    value_parameters: Option<Vec<(Identifier, TypeConstructor)>>,
    return_type: Option<TypeConstructor>,
    body: WithPosition<Block>,
    #[serde(skip)]
    context_template: Context,
}

impl PartialEq for FunctionNode {
    fn eq(&self, other: &Self) -> bool {
        self.type_parameters == other.type_parameters
            && self.value_parameters == other.value_parameters
            && self.return_type == other.return_type
            && self.body == other.body
    }
}
