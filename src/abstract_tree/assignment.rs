use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, AssignmentOperator, Format, Identifier, SourcePosition, Statement, SyntaxNode,
    Type, TypeSpecification, Value,
};

/// Variable assignment, including add-assign and subtract-assign operations.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    type_specification: Option<TypeSpecification>,
    operator: AssignmentOperator,
    statement: Statement,
    syntax_position: SourcePosition,
}

impl AbstractTree for Assignment {
    fn from_syntax(
        syntax_node: SyntaxNode,
        source: &str,
        context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "assignment", syntax_node)?;

        let child_count = syntax_node.child_count();

        let identifier_node = syntax_node.child(0).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let type_node = syntax_node.child(1).unwrap();
        let type_specification = if type_node.kind() == "type_specification" {
            Some(TypeSpecification::from_syntax(type_node, source, context)?)
        } else {
            None
        };

        let operator_node = syntax_node.child(child_count - 2).unwrap();
        let operator = AssignmentOperator::from_syntax(operator_node, source, context)?;

        let statement_node = syntax_node.child(child_count - 1).unwrap();
        let statement = Statement::from_syntax(statement_node, source, context)?;

        Ok(Assignment {
            identifier,
            type_specification,
            operator,
            statement,
            syntax_position: syntax_node.range().into(),
        })
    }

    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError> {
        if let AssignmentOperator::Equal = self.operator {
            let r#type = if let Some(definition) = &self.type_specification {
                definition.inner().clone()
            } else {
                self.statement.expected_type(context)?
            };

            context.set_type(self.identifier.clone(), r#type)?;
        }

        if let Some(type_specification) = &self.type_specification {
            match self.operator {
                AssignmentOperator::Equal => {
                    let expected = type_specification.inner();
                    let actual = self.statement.expected_type(context)?;

                    if !expected.accepts(&actual) {
                        return Err(ValidationError::TypeCheck {
                            expected: expected.clone(),
                            actual,
                            position: self.syntax_position,
                        });
                    }
                }
                AssignmentOperator::PlusEqual => {
                    if let Type::List(expected) = type_specification.inner() {
                        let actual = self.identifier.expected_type(context)?;

                        if !expected.accepts(&actual) {
                            return Err(ValidationError::TypeCheck {
                                expected: expected.as_ref().clone(),
                                actual,
                                position: self.syntax_position,
                            });
                        }
                    } else {
                        let expected = type_specification.inner();
                        let actual = self.identifier.expected_type(context)?;

                        if !expected.accepts(&actual) {
                            return Err(ValidationError::TypeCheck {
                                expected: expected.clone(),
                                actual,
                                position: self.syntax_position,
                            });
                        }
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        } else {
            match self.operator {
                AssignmentOperator::Equal => {}
                AssignmentOperator::PlusEqual => {
                    if let Type::List(expected) = self.identifier.expected_type(context)? {
                        let actual = self.statement.expected_type(context)?;

                        if !expected.accepts(&actual) {
                            return Err(ValidationError::TypeCheck {
                                expected: expected.as_ref().clone(),
                                actual,
                                position: self.syntax_position,
                            });
                        }
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        }

        self.statement.validate(source, context)?;

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let right = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(left) = context.get_value(&self.identifier)? {
                    left.add(right)?
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableIdentifierNotFound(self.identifier.clone()),
                    ));
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(left) = context.get_value(&self.identifier)? {
                    left.subtract(right)?
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableIdentifierNotFound(self.identifier.clone()),
                    ));
                }
            }
            AssignmentOperator::Equal => right,
        };

        context.set_value(self.identifier.clone(), new_value)?;

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }
}

impl Format for Assignment {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.identifier.format(output, indent_level);

        if let Some(type_specification) = &self.type_specification {
            type_specification.format(output, indent_level);
        }

        output.push(' ');
        self.operator.format(output, indent_level);
        output.push(' ');

        self.statement.format(output, 0);
    }
}
