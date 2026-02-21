use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::error::{DustError, InternalError},
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationMembers},
        type_graph::{TypeId, TypeMembers, TypeNode},
    },
    source::{Position, Source, SourceFileId},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    file_id: SourceFileId,

    source: &'a Source<'a>,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<DustError>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: &'a Source,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<DustError>,
    ) -> Self {
        Self {
            source,
            file_id,
            syntax,
            resolver,
            errors,
        }
    }

    pub fn bind(mut self) -> Result<(), DustError> {
        let main_root = self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .ok_or(DustError::Internal(
                InternalError::MissingSyntaxTree(SourceFileId::MAIN),
            ))?
            .root()
            .ok_or(DustError::Internal(
                InternalError::MissingSyntaxNode(SyntaxId::ROOT),
            ))?;

        self.visit_root(main_root)
    }
}

impl SyntaxVisitor for TypeBinder<'_> {
    type RootOutput = ();
    type ItemOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = TypeId;
    type TypeOutput = TypeId;
    type PathOutput = TypeId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, DustError> {
        debug!("Visting root");

        let children = node.children();

        for child in children {
            self.visit_item(child)?;
        }

        Ok(())
    }

    fn visit_module_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, DustError> {
        debug!("Visting module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        function_item: SyntaxReader,
    ) -> Result<Self::ItemOutput, DustError> {
        debug!("Visting function item");

        let function_expression = function_item.expect_right_child()?;

        let function_declaration_id = *self.resolver.get_declaration_binding(&function_item.id)?;
        let function_type_id = self.visit_function_expression(function_expression, ())?;

        info!(
            "Declaring type of function \"{}\" as `{}`",
            self.resolver
                .declarations
                .get_declaration(function_declaration_id)
                .and_then(|declaration| self.resolver.symbols.get_symbol(&declaration.symbol_id))
                .unwrap_or("anonymous"),
            self.resolver
                .get_full_type(function_type_id, self.source)
                .map(|r#type| r#type.to_string())
                .unwrap_or_else(|_| "unknown type".to_string())
        );

        self.resolver
            .declarations
            .set_declaration_type(function_declaration_id, function_type_id);

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, DustError> {
        debug!("Visting use item");

        todo!()
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError> {
        debug!("Visting struct item");

        let (struct_name, struct_fields_list) = node.expect_binary_children()?;
        let struct_fields = struct_fields_list.expect_multiple_children()?;

        let mut fields = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields {
            let (field_name, field_type) = field.expect_binary_children()?;

            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_type_id = self.visit_type(field_type)?;

            self.resolver
                .declarations
                .set_declaration_type(field_declaration_id, field_type_id);
            fields.push(field_declaration_id);
        }

        let declaration_id = *self.resolver.get_declaration_binding(&struct_name.id)?;
        let fields = self.resolver.declarations.add_declaration_members(&fields);
        let struct_type = TypeNode::Struct {
            declaration_id,
            fields,
            generics: DeclarationMembers::default(),
        };
        let struct_type_id = self.resolver.types.add_type(struct_type);

        self.resolver
            .declarations
            .set_declaration_type(declaration_id, struct_type_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError> {
        debug!("Visting expression statement");

        self.visit_expression(node.expect_left_child()?, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError> {
        debug!("Visting let statement");

        let mut children = node.children();
        let path = children.expect_next()?;
        let expression = children.expect_next()?;
        let type_notation = children.next();

        let expression_type_id = self.visit_expression(expression, ())?;

        if let Some(type_notation) = type_notation {
            let explicit_type = self.visit_type(type_notation)?;

            match self.resolver.unify_types(
                explicit_type,
                Some(type_notation),
                expression_type_id,
                expression,
            ) {
                Ok(()) => {}
                Err(error) => {
                    self.errors.push(error);

                    return Ok(());
                }
            }
        }

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;

        self.resolver
            .add_type_binding(expression.id, expression_type_id);
        self.resolver.add_type_binding(node.id, TypeId::NONE);
        self.resolver
            .declarations
            .set_declaration_type(declaration_id, expression_type_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError> {
        debug!("Visting binary assignment statement");

        let (path, expression) = node.expect_binary_children()?;

        let path_type = {
            let raw = self.visit_path(path)?;

            self.resolver.infer_type(raw)?
        };
        let expression_type = {
            let raw = self.visit_expression(expression, ())?;

            self.resolver.infer_type(raw)?
        };

        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionAssignmentStatement
                if (path_type == TypeId::STRING && expression_type == TypeId::CHARACTER)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::STRING)
            || (path_type == TypeId::CHARACTER && expression_type == TypeId::CHARACTER)
        );

        let unified =
            self.resolver
                .unify_inferred_types(path_type, Some(path), expression_type, expression);

        if unified.is_err() && is_character_concatenation {
            self.resolver.add_type_binding(path.id, TypeId::STRING);
            self.resolver
                .add_type_binding(expression.id, TypeId::CHARACTER);

            return Ok(());
        }

        match unified {
            Ok(()) => {}
            Err(error) => {
                self.errors.push(error);

                return Ok(());
            }
        }

        Ok(())
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError> {
        debug!("Visting reassignment statement");

        let (path, expression_statement) = node.expect_binary_children()?;
        let expression = expression_statement.expect_left_child()?;

        let path_type = self.visit_path(path)?;
        let expression_type = self.visit_expression(expression, ())?;

        self.resolver
            .unify_types(path_type, Some(path), expression_type, expression)?;
        self.resolver.add_type_binding(path.id, path_type);
        self.resolver
            .add_type_binding(expression.id, expression_type);
        self.resolver.add_type_binding(node.id, TypeId::NONE);

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting boolean expression");

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting byte expression");

        self.resolver.add_type_binding(node.id, TypeId::BYTE);

        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting character expression");

        self.resolver.add_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting float expression");

        self.resolver.add_type_binding(node.id, TypeId::FLOAT);

        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting integer expression");

        self.resolver.add_type_binding(node.id, TypeId::INTEGER);

        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting string expression");

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(TypeId::STRING)
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting list expression");

        let children = node.expect_multiple_children()?;

        let mut first_type = None;

        for child in children {
            let child_type = self.visit_expression(child, ())?;

            if let Some((previous_type, previous)) = first_type {
                self.resolver
                    .unify_types(previous_type, Some(previous), child_type, child)?;
            } else {
                first_type = Some((child_type, child));
            }
        }

        let element_type = if let Some((element_type, _)) = first_type {
            element_type
        } else {
            self.resolver.types.create_inferred_type()
        };
        let list_type = self
            .resolver
            .types
            .add_type(TypeNode::List { element_type });

        self.resolver.add_type_binding(node.id, list_type);

        Ok(list_type)
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.expect_binary_children()?;

        let list_type_id = {
            let raw = self.visit_expression(list_expression, input)?;

            self.resolver.infer_type(raw)?
        };
        let index_type_id = {
            let raw = self.visit_expression(index_expression, input)?;

            self.resolver.infer_type(raw)?
        };

        if index_type_id != TypeId::INTEGER {
            return Err(DustError::ExpectedIntegerIndex {
                found: index_type_id,
                position: index_expression.position(),
            });
        }

        let list_type = *self.resolver.types.get_type(list_type_id)?;
        let element_type = match list_type {
            TypeNode::List { element_type } => {
                self.resolver.add_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(DustError::CannotIndex {
                    type_id: list_type_id,
                    position: list_expression.position(),
                });
            }
        };

        self.resolver.add_type_binding(node.id, element_type);
        self.resolver
            .add_type_binding(list_expression.id, list_type_id);

        Ok(element_type)
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting path expression");
        debug_assert_eq!(path_expression.kind(), SyntaxKind::PathExpression);

        let type_id = self.visit_path(path_expression.expect_left_child()?)?;

        self.resolver.add_type_binding(path_expression.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting struct expression");

        let fields = node.expect_right_child()?.expect_multiple_children()?;

        let declaration_id = *self.resolver.get_declaration_binding(&node.id)?;
        let declared_struct_type_id = *self
            .resolver
            .declarations
            .get_declaration_type(&declaration_id)?;

        for field in fields {
            let (field_name, field_expression) = field.expect_binary_children()?;

            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let declared_field_type_id = *self
                .resolver
                .declarations
                .get_declaration_type(&field_declaration_id)?;
            let actual_field_type_id = self.visit_expression(field_expression, ())?;

            self.resolver.unify_types(
                declared_field_type_id,
                Some(field_name),
                actual_field_type_id,
                field_expression,
            )?;
            self.resolver
                .add_type_binding(field_expression.id, declared_field_type_id);
        }

        self.resolver
            .add_type_binding(node.id, declared_struct_type_id);

        Ok(declared_struct_type_id)
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting block expression");

        let children = node.children();

        let mut block_type_id = TypeId::NONE;

        for child in children {
            let child_type = if child.is_item() {
                self.visit_item(child)?;

                TypeId::NONE
            } else if child.is_statement() {
                self.visit_statement(child)?;

                TypeId::NONE
            } else {
                self.visit_expression(child, ())?
            };
            block_type_id = child_type;

            self.resolver.add_type_binding(child.id, child_type);
        }

        self.resolver.add_type_binding(node.id, block_type_id);

        Ok(block_type_id)
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting if expression");

        let mut children = node.expect_multiple_children()?;

        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.resolver.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(DustError::ExpectedBooleanExpression {
                found: condition_type,
                node_kind: node.kind(),
                position: condition.position(),
            });
        }

        let then_type = self.visit_block_expression(then_block, ())?;

        if let Some(else_block) = else_block {
            let else_type = self.visit_else_expression(else_block, ())?;

            self.resolver.unify_types(
                then_type,
                Some(then_block),
                else_type,
                else_block.expect_left_child()?,
            )?;
        }

        self.resolver.add_type_binding(node.id, then_type);

        Ok(then_type)
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting else expression");

        self.visit_block_expression(node.expect_left_child()?, ())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, ())?;

            self.resolver.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, ())?;

            self.resolver.infer_type(raw)?
        };

        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentStatement
                if (left_type == TypeId::STRING && right_type == TypeId::CHARACTER)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::STRING)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::CHARACTER)
        );

        let math_expression_type = if is_character_concatenation {
            TypeId::STRING
        } else {
            self.resolver.unify_inferred_types(
                left_type,
                Some(left_expression),
                right_type,
                right_expression,
            )?;

            left_type
        };

        self.resolver
            .add_type_binding(node.id, math_expression_type);

        Ok(math_expression_type)
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting comparison binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_type = self.visit_expression(left_expression, ())?;
        let right_type = self.visit_expression(right_expression, ())?;

        self.resolver.unify_types(
            left_type,
            Some(left_expression),
            right_type,
            right_expression,
        )?;
        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting logical binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, input)?;

            self.resolver.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, input)?;

            self.resolver.infer_type(raw)?
        };

        if left_type != TypeId::BOOLEAN {
            return Err(DustError::ExpectedBooleanExpression {
                found: left_type,
                node_kind: left_expression.kind(),
                position: left_expression.position(),
            });
        }

        if right_type != TypeId::BOOLEAN {
            return Err(DustError::ExpectedBooleanExpression {
                found: right_type,
                node_kind: right_expression.kind(),
                position: right_expression.position(),
            });
        }

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting unary negation expression");

        let expression = node.expect_left_child()?;
        let child_type = {
            let raw = self.visit_expression(expression, ())?;

            self.resolver.infer_type(raw)?
        };

        match child_type {
            TypeId::BOOLEAN | TypeId::BYTE | TypeId::FLOAT | TypeId::INTEGER => {
                self.resolver.add_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => Err(DustError::CannotApplyOperator {
                operator: node.kind(),
                type_id: child_type,
                position: expression.position(),
            }),
        }
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting while expression");

        let (condition, body) = node.expect_binary_children()?;

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.resolver.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(DustError::ExpectedBooleanExpression {
                found: condition_type,
                node_kind: condition.kind(),
                position: condition.position(),
            });
        }

        self.visit_block_expression(body, ())?;

        Ok(TypeId::NONE)
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting function expression");

        let (signature, body) = node.expect_binary_children()?;
        let value_parameters_list = signature.expect_left_child()?;
        let value_parameters = value_parameters_list.children();
        let return_type = signature.right_child()?;

        let mut value_parameter_types = SmallVec::<[TypeId; 8]>::new();

        for parameter_node in value_parameters {
            let parameter_name = parameter_node.expect_left_child()?;
            let parameter_type = parameter_node.expect_right_child()?;

            let parameter_declaration_id =
                *self.resolver.get_declaration_binding(&parameter_name.id)?;
            let parameter_type_id = self.visit_type(parameter_type)?;

            value_parameter_types.push(parameter_type_id);
            self.resolver
                .declarations
                .set_declaration_type(parameter_declaration_id, parameter_type_id);
        }

        let value_parameter_children = self.resolver.types.add_type_members(&value_parameter_types);
        let expected_return_type_id = {
            if let Some(return_type_node) = return_type {
                let raw = self.visit_type(return_type_node)?;

                self.resolver.infer_type(raw)?
            } else {
                TypeId::NONE
            }
        };
        let actual_return_type_id = self.visit_block_expression(body, ())?;

        self.resolver.unify_types(
            expected_return_type_id,
            return_type,
            actual_return_type_id,
            body,
        )?;

        let function_type_id = self.resolver.types.add_type(TypeNode::Function {
            type_parameters: DeclarationMembers::default(),
            value_parameters: value_parameter_children,
            return_type_id: expected_return_type_id,
        });

        self.resolver.add_type_binding(node.id, function_type_id);
        self.resolver
            .add_type_binding(body.id, expected_return_type_id);

        Ok(function_type_id)
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        debug!("Visting call expression");

        let (callee, arguments_list) = node.expect_binary_children()?;
        let arguments = arguments_list.expect_multiple_children()?;

        let callee_type = {
            let raw = self.visit_expression(callee, ())?;

            self.resolver.infer_type(raw)?
        };

        let TypeNode::Function {
            value_parameters,
            return_type_id,
            ..
        } = *self.resolver.types.get_type(callee_type)?
        else {
            return Err(DustError::ExpectedFunctionType {
                found: callee_type,
                position: callee.position(),
            });
        };

        let expected_parameters = self
            .resolver
            .types
            .get_type_members(value_parameters)?
            .iter()
            .copied()
            .collect::<SmallVec<[TypeId; 8]>>();

        if arguments.len() != expected_parameters.len() {
            return Err(DustError::ExpectedArguments {
                function_type: callee_type,
                found_position: callee.position(),
                expected_count: expected_parameters.len(),
                found_count: arguments.len(),
            });
        }

        for (argument, expected_type_id) in arguments.zip(expected_parameters.into_iter()) {
            let argument_type = self.visit_expression(argument, ())?;

            self.resolver
                .unify_types(expected_type_id, None, argument_type, argument)?;
        }

        self.resolver.add_type_binding(node.id, return_type_id);

        Ok(return_type_id)
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, DustError> {
        match node.kind() {
            SyntaxKind::AnyType => Ok(self.resolver.types.create_inferred_type()),
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Ok(TypeId::BYTE),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::FloatType => Ok(TypeId::FLOAT),
            SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_node = node.expect_left_child()?;
                let element_type_id = self.visit_type(element_type_node)?;
                let list_type_id = self.resolver.types.add_type(TypeNode::List {
                    element_type: element_type_id,
                });

                Ok(list_type_id)
            }
            SyntaxKind::FunctionType => {
                let type_node = {
                    let type_node_value_parameters = if node.has_left_child() {
                        let value_parameters =
                            node.expect_left_child()?.expect_multiple_children()?;
                        let mut value_parameter_ids = SmallVec::<[TypeId; 4]>::new();

                        for value_parameter in value_parameters {
                            let type_id = if value_parameter.id == SyntaxId::NONE {
                                TypeId::NONE
                            } else {
                                self.visit_type(value_parameter)?
                            };

                            value_parameter_ids.push(type_id);
                        }

                        self.resolver.types.add_type_members(&value_parameter_ids)
                    } else {
                        TypeMembers::default()
                    };

                    let return_type_id = if node.has_right_child() {
                        self.visit_type(node.expect_right_child()?)?
                    } else {
                        TypeId::NONE
                    };

                    TypeNode::Function {
                        type_parameters: DeclarationMembers::default(),
                        value_parameters: type_node_value_parameters,
                        return_type_id,
                    }
                };
                let function_type_id = self.resolver.types.add_type(type_node);

                Ok(function_type_id)
            }
            SyntaxKind::TypePath => {
                let declaration_id = self.resolver.get_declaration_binding(&node.id)?;
                let type_id = self
                    .resolver
                    .declarations
                    .get_declaration_type(declaration_id)?;

                Ok(*type_id)
            }
            _ => Err(DustError::Internal(
                InternalError::UnimplementedSyntaxFeature(node.kind()),
            )),
        }
    }

    fn visit_path(&mut self, path: SyntaxReader) -> Result<Self::PathOutput, DustError> {
        debug!("Visting path");

        self.resolver
            .get_declaration_binding(&path.id)
            .and_then(|declaration_id| {
                self.resolver
                    .declarations
                    .get_declaration_type(declaration_id)
            })
            .copied()
    }
}
