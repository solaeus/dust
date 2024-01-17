use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, AssignmentOperator, Error, Format, Identifier, Map, Result, Statement,
    SyntaxNode, SyntaxPosition, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    type_definition: Option<TypeDefinition>,
    operator: AssignmentOperator,
    statement: Statement,

    syntax_position: SyntaxPosition,
}

impl AbstractTree for Assignment {
    fn from_syntax(syntax_node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "assignment", syntax_node)?;

        let child_count = syntax_node.child_count();

        let identifier_node = syntax_node.child(0).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let type_node = syntax_node.child(1).unwrap();
        let type_definition = if type_node.kind() == "type_definition" {
            Some(TypeDefinition::from_syntax(type_node, source, context)?)
        } else {
            None
        };

        let operator_node = syntax_node.child(child_count - 2).unwrap();
        let operator = AssignmentOperator::from_syntax(operator_node, source, context)?;

        let statement_node = syntax_node.child(child_count - 1).unwrap();
        let statement = Statement::from_syntax(statement_node, source, context)?;

        if let AssignmentOperator::Equal = operator {
            context.set_type(
                identifier.inner().clone(),
                statement.expected_type(context)?,
            )?;
        }

        Ok(Assignment {
            identifier,
            type_definition,
            operator,
            statement,
            syntax_position: syntax_node.range().into(),
        })
    }

    fn check_type(&self, source: &str, context: &Map) -> Result<()> {
        let actual_type = self.statement.expected_type(context)?;

        if let Some(type_definition) = &self.type_definition {
            match self.operator {
                AssignmentOperator::Equal => {
                    type_definition
                        .inner()
                        .check(&actual_type)
                        .map_err(|error| error.at_source_position(source, self.syntax_position))?;
                }
                AssignmentOperator::PlusEqual => {
                    if let Type::List(item_type) = type_definition.inner() {
                        item_type.check(&actual_type).map_err(|error| {
                            error.at_source_position(source, self.syntax_position)
                        })?;
                    } else {
                        type_definition
                            .inner()
                            .check(&self.identifier.expected_type(context)?)
                            .map_err(|error| {
                                error.at_source_position(source, self.syntax_position)
                            })?;
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        } else {
            match self.operator {
                AssignmentOperator::Equal => {}
                AssignmentOperator::PlusEqual => {
                    if let Type::List(item_type) = self.identifier.expected_type(context)? {
                        item_type.check(&actual_type).map_err(|error| {
                            error.at_source_position(source, self.syntax_position)
                        })?;
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        }

        self.statement
            .check_type(source, context)
            .map_err(|error| error.at_source_position(source, self.syntax_position))?;

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let key = self.identifier.inner();
        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some((mut previous_value, _)) = context.variables()?.get(key).cloned() {
                    previous_value += value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some((mut previous_value, _)) = context.variables()?.get(key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::Equal => value,
        };

        context.set(key.clone(), new_value)?;

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
    }
}

impl Format for Assignment {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.identifier.format(output, indent_level);

        if let Some(type_definition) = &self.type_definition {
            type_definition.format(output, indent_level);
        }

        output.push(' ');
        self.operator.format(output, indent_level);
        output.push(' ');

        self.statement.format(output, 0);
    }
}
