use std::{cmp::Ordering, ops::RangeInclusive};

use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Expression, Format, Function, FunctionNode,
    Identifier, List, Type, Value, TypeDefinition, MapNode,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ValueNode {
    Boolean(String),
    Float(String),
    Function(Function),
    Integer(String),
    String(String),
    List(Vec<Expression>),
    Map(MapNode),
    Range(RangeInclusive<i64>),
    Struct {
        name: Identifier,
        properties: MapNode,
    },
    Enum {
        name: Identifier,
        variant: Identifier,
        expression: Option<Box<Expression>>,
    },
}

impl AbstractTree for ValueNode {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("value", node)?;

        let child = node.child(0).unwrap();
        let value_node = match child.kind() {
            "boolean" => ValueNode::Boolean(source[child.byte_range()].to_string()),
            "float" => ValueNode::Float(source[child.byte_range()].to_string()),
            "function" => {
                let function_node = FunctionNode::from_syntax(child, source, context)?;

                ValueNode::Function(Function::ContextDefined(function_node))
            }
            "integer" => ValueNode::Integer(source[child.byte_range()].to_string()),
            "string" => {
                let without_quotes = child.start_byte() + 1..child.end_byte() - 1;

                ValueNode::String(source[without_quotes].to_string())
            }
            "list" => {
                let mut expressions = Vec::new();

                for index in 1..child.child_count() - 1 {
                    let current_node = child.child(index).unwrap();

                    if current_node.is_named() {
                        let expression = Expression::from_syntax(current_node, source, context)?;

                        expressions.push(expression);
                    }
                }

                ValueNode::List(expressions)
            }
            "map" => {
                ValueNode::Map(MapNode::from_syntax(child, source, context)?)
            }
            "range" => {
                let mut split = source[child.byte_range()].split("..");
                let start = split.next().unwrap().parse().unwrap();
                let end = split.next().unwrap().parse().unwrap();

                ValueNode::Range(start..=end)
            }
            "enum_instance" => {
                let name_node = child.child(0).unwrap();
                let name = Identifier::from_syntax(name_node, source, context)?;
                
                let variant_node = child.child(2).unwrap();
                let variant = Identifier::from_syntax(variant_node, source, context)?;
             
                let expression = if let Some(expression_node) = child.child(4) {                
                    Some(Box::new(Expression::from_syntax(expression_node, source, context)?))
                } else {
                    None
                };

                ValueNode::Enum { name, variant , expression  }                
            }
            "struct_instance" => {
                let name_node = child.child(0).unwrap();
                let name = Identifier::from_syntax(name_node, source, context)?;

                let properties_node = child.child(2).unwrap();
                let properties = MapNode::from_syntax(properties_node, source, context)?;

                ValueNode::Struct
                {
                    name,
                    properties
                }
            }
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected:
                        "string, integer, float, boolean, range, list, map, option, function, struct or enum"
                            .to_string(),
                    actual: child.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(value_node)
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Function(function) => function.r#type(),
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::String(_) => Type::String,
            ValueNode::List(expressions) => {
                let mut item_types = Vec::new();

                for expression in expressions {
                    let expression_type = expression.expected_type(context)?;

                    item_types.push(expression_type);
                }

                Type::ListExact(item_types)
            }
            ValueNode::Map(map_node) => map_node.expected_type(context)?,
            ValueNode::Struct { name, .. }  => {
                Type::custom(name.clone(), Vec::with_capacity(0))
            }
            ValueNode::Range(_) => Type::Range,
            ValueNode::Enum { name, variant, expression: _ } => {
                let types: Vec<Type> = if let Some(type_definition) = context.get_definition(name)? {
                    if let TypeDefinition::Enum(enum_definition) = type_definition {
                        let types = enum_definition.variants().into_iter().find_map(|(identifier, types)| {
                            if identifier == variant {
                                Some(types.clone())
                            } else {
                                None
                            }
                        });

                        if let Some(types) = types {
                            types 
                        } else {
                            return Err(ValidationError::VariableIdentifierNotFound(variant.clone()));
                        }
                         
                    } else {
                        return Err(ValidationError::ExpectedEnumDefintion { actual: type_definition.clone() });
                    }
                } else {
                     return Err(ValidationError::VariableIdentifierNotFound(name.clone()));
                };

                Type::custom(name.clone(), types.clone())
                
            },
        };

        Ok(r#type)
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        match self {
            ValueNode::Function(function) => {
                if let Function::ContextDefined(function_node) = function {
                    function_node.validate(_source, context)?;
                }
            }
            ValueNode::Map(map_node) => map_node.validate(_source, context)?,
            ValueNode::Enum { name, expression, .. } => {
                name.validate(_source, context)?;

                if let Some(expression) = expression {
                    expression.validate(_source, context)?;
                }
            }
            _ => {},
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(value_source) => Value::Boolean(value_source.parse().unwrap()),
            ValueNode::Float(value_source) => {
                let float = value_source.parse()?;

                Value::Float(float)
            }
            ValueNode::Function(function) => Value::Function(function.clone()),
            ValueNode::Integer(value_source) => Value::Integer(value_source.parse().unwrap()),
            ValueNode::String(value_source) => Value::string(value_source.clone()),
            ValueNode::List(expressions) => {
                let mut values = Vec::with_capacity(expressions.len());

                for node in expressions {
                    let value = node.run(source, context)?;

                    values.push(value);
                }

                Value::List(List::with_items(values))
            }
            ValueNode::Map(map_node) => map_node.run(source, context)?,
            ValueNode::Range(range) => Value::Range(range.clone()),
            ValueNode::Struct { name, properties } => {
                let instance = if let Some(definition) = context.get_definition(name)? {
                    if let TypeDefinition::Struct(struct_definition) = definition {
                        struct_definition.instantiate(properties, source, context)?
                    } else {
                        return Err(RuntimeError::ValidationFailure(ValidationError::ExpectedStructDefintion { actual: definition.clone() }))
                    }
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::TypeDefinitionNotFound(name.clone())
                    ));
                };

                Value::Struct(instance)

            }
            ValueNode::Enum { name, variant, expression } => {
                let value = if let Some(expression) = expression {
                    expression.run(source, context)?
                } else {
                    Value::none()
                };
                let instance = if let Some(definition) = context.get_definition(name)? {
                    if let TypeDefinition::Enum(enum_defintion) = definition {
                        enum_defintion.instantiate(variant.clone(), Some(value))
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedEnumDefintion {
                                actual: definition.clone()
                            }
                        ));
                    }
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::TypeDefinitionNotFound(name.clone())
                    ));
                };

                Value::Enum(instance)                
            },
        };

        Ok(value)
    }
}

impl Format for ValueNode {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            ValueNode::Boolean(source) | ValueNode::Float(source) | ValueNode::Integer(source) => {
                output.push_str(source)
            }
            ValueNode::String(source) => {
                output.push('\'');
                output.push_str(source);
                output.push('\'');
            }
            ValueNode::Function(function) => function.format(output, indent_level),
            ValueNode::List(expressions) => {
                output.push('[');

                for expression in expressions {
                    expression.format(output, indent_level);
                }

                output.push(']');
            }
            ValueNode::Map(map_node) => map_node.format(output, indent_level),
            ValueNode::Struct { name, properties } => {
                name.format(output, indent_level);
                properties.format(output, indent_level);
            }
            ValueNode::Range(_) => todo!(),
            ValueNode::Enum { ..  } => todo!(),
        }
    }
}

impl Ord for ValueNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ValueNode::Boolean(left), ValueNode::Boolean(right)) => left.cmp(right),
            (ValueNode::Boolean(_), _) => Ordering::Greater,
            (ValueNode::Float(left), ValueNode::Float(right)) => left.cmp(right),
            (ValueNode::Float(_), _) => Ordering::Greater,
            (ValueNode::Function(left), ValueNode::Function(right)) => left.cmp(right),
            (ValueNode::Function(_), _) => Ordering::Greater,
            (ValueNode::Integer(left), ValueNode::Integer(right)) => left.cmp(right),
            (ValueNode::Integer(_), _) => Ordering::Greater,
            (ValueNode::String(left), ValueNode::String(right)) => left.cmp(right),
            (ValueNode::String(_), _) => Ordering::Greater,
            (ValueNode::List(left), ValueNode::List(right)) => left.cmp(right),
            (ValueNode::List(_), _) => Ordering::Greater,
            (ValueNode::Map(left), ValueNode::Map(right)) => left.cmp(right),
            (ValueNode::Map(_), _) => Ordering::Greater,
            (ValueNode::Struct{ name: left_name, properties: left_properties }, ValueNode::Struct {name: right_name, properties: right_properties} ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    left_properties.cmp(right_properties)
                } else {
                    name_cmp
                }
            },
            (ValueNode::Struct {..}, _) => Ordering::Greater,
            (
                ValueNode::Enum {
                    name: left_name, variant: left_variant, expression: left_expression
                },
                ValueNode::Enum {
                    name: right_name, variant: right_variant, expression: right_expression
                }
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    let variant_cmp = left_variant.cmp(right_variant);

                    if variant_cmp.is_eq() {
                        left_expression.cmp(right_expression)
                    } else {
                        variant_cmp
                    }
                } else {
                    name_cmp
                }
            },
            (ValueNode::Enum { .. }, _) => Ordering::Greater,
            (ValueNode::Range(left), ValueNode::Range(right)) => left.clone().cmp(right.clone()),
            (ValueNode::Range(_), _) => Ordering::Less,
        }
    }
}

impl PartialOrd for ValueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
