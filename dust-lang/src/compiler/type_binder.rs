use smallvec::SmallVec;
use tracing::{debug, info};

use crate::{
    compiler::error::CompileError,
    dust_error::{ErrorKind, InternalError},
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind, DeclarationMembers, ModuleKind},
        type_graph::{TypeId, TypeMembers, TypeNode},
    },
    source::{Source, SourceFileId},
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    file_id: SourceFileId,

    source: &'a Source<'a>,

    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<ErrorKind>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        file_id: SourceFileId,
        source: &'a Source,
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
    ) -> Self {
        Self {
            source,
            file_id,
            syntax,
            resolver,
            errors,
        }
    }
}

impl SyntaxVisitor for TypeBinder<'_> {
    type RootOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = TypeId;
    type TypeOutput = TypeId;
    type PathOutput = TypeId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, ErrorKind> {
        debug!("Visting root");

        let children = node.children()?;

        for child in children {
            self.visit_item(child);
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visting module item");

        let mut children = module_item.children()?;
        let module_name = children.expect_next()?;
        let module_body = children.next();

        if let Some(module_body) = module_body {
            for child in module_body.children()? {
                self.visit_item(child);
            }
        } else {
            let module_declaration_id = *self.resolver.get_declaration_binding(&module_name.id)?;
            let module_declaration = self
                .resolver
                .declarations
                .get_declaration(module_declaration_id)?;
            let module_file_id = if let DeclarationKind::Module {
                kind: ModuleKind::File { file_id },
                ..
            } = module_declaration.kind
            {
                file_id
            } else {
                return Err(ErrorKind::Internal(
                    InternalError::ExpectedModuleDeclaration {
                        declaration_id: module_declaration_id,
                    },
                ));
            };
            let module_root = self.syntax.get_tree(module_file_id)?.root()?;
            let mut module_type_binder = TypeBinder::new(
                module_file_id,
                self.source,
                self.syntax,
                self.resolver,
                self.errors,
            );

            module_type_binder.visit_root(module_root)?;
        }

        Ok(())
    }

    fn visit_function_item(&mut self, function_item: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visting function item");

        let (_function_name, function_expression) = function_item.binary_children()?;

        let function_declaration_id = *self.resolver.get_declaration_binding(&function_item.id)?;
        let function_type_id = self.visit_function_expression(function_expression, ())?;

        self.resolver
            .declarations
            .set_declaration_type(function_declaration_id, function_type_id);

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visting use item");

        Ok(())
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visting struct item");

        let (struct_name, struct_fields) = node.binary_children()?;

        let mut fields = SmallVec::<[DeclarationId; 8]>::new();

        for field in struct_fields.children()? {
            let (field_name, field_type) = field.binary_children()?;

            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_type_id = self.visit_type(field_type)?;

            self.resolver
                .declarations
                .set_declaration_type(field_declaration_id, field_type_id);
            fields.push(field_declaration_id);
        }

        let declaration_id = *self.resolver.get_declaration_binding(&struct_name.id)?;
        let struct_type = TypeNode::Struct {
            declaration_id,
            type_arguments: TypeMembers::default(),
        };
        let struct_type_id = self.resolver.types.add_type(struct_type);

        self.resolver
            .declarations
            .set_declaration_type(declaration_id, struct_type_id);

        Ok(())
    }

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        debug!("Visting enum item");

        let mut children = node.children()?;
        let enum_name = children.expect_next()?;
        let enum_variants = children.expect_next()?;

        let mut variants = SmallVec::<[DeclarationId; 8]>::new();

        for variant in enum_variants.children()? {
            debug!("Visting enum variant");

            let (variant_name, variant_type) = variant.binary_children()?;

            let variant_declaration_id =
                *self.resolver.get_declaration_binding(&variant_name.id)?;
            let variant_type_id = self.visit_type(variant_type)?;

            self.resolver
                .declarations
                .set_declaration_type(variant_declaration_id, variant_type_id);
            variants.push(variant_declaration_id);
        }

        let declaration_id = *self.resolver.get_declaration_binding(&enum_name.id)?;
        let enum_type = TypeNode::Enum {
            declaration_id,
            type_arguments: TypeMembers::default(),
        };
        let enum_type_id = self.resolver.types.add_type(enum_type);

        self.resolver
            .declarations
            .set_declaration_type(declaration_id, enum_type_id);

        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting expression statement");

        self.visit_expression(node.child()?, ())?;

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting let statement");

        let mut children = node.children()?;
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
        self.resolver.add_type_binding(node.id, TypeId::UNIT);
        self.resolver
            .declarations
            .set_declaration_type(declaration_id, expression_type_id);

        Ok(())
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting binary assignment statement");

        let (path, expression) = node.binary_children()?;

        let path_type = {
            let raw = self.visit_path(path, false)?;

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
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting reassignment statement");

        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.child()?;

        let path_type = self.visit_path(path, false)?;
        let expression_type = self.visit_expression(expression, ())?;

        self.resolver
            .unify_types(path_type, Some(path), expression_type, expression)?;
        self.resolver.add_type_binding(path.id, path_type);
        self.resolver
            .add_type_binding(expression.id, expression_type);
        self.resolver.add_type_binding(node.id, TypeId::UNIT);

        Ok(())
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting boolean expression");

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting byte expression");

        self.resolver.add_type_binding(node.id, TypeId::BYTE);

        Ok(TypeId::BYTE)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting character expression");

        self.resolver.add_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting float expression");

        self.resolver.add_type_binding(node.id, TypeId::FLOAT);

        Ok(TypeId::FLOAT)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting integer expression");

        self.resolver.add_type_binding(node.id, TypeId::INTEGER);

        Ok(TypeId::INTEGER)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting string expression");

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(TypeId::STRING)
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting list expression");

        let children = node.children()?;

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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.binary_children()?;

        let list_type_id = {
            let raw = self.visit_expression(list_expression, input)?;

            self.resolver.infer_type(raw)?
        };
        let index_type_id = {
            let raw = self.visit_expression(index_expression, input)?;

            self.resolver.infer_type(raw)?
        };

        if index_type_id != TypeId::INTEGER {
            return Err(ErrorKind::Compile(CompileError::ExpectedIntegerIndex {
                found: index_type_id,
                position: index_expression.position(),
            }));
        }

        let list_type = *self.resolver.types.get_type(list_type_id)?;
        let element_type = match list_type {
            TypeNode::List { element_type } => {
                self.resolver.add_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(ErrorKind::Compile(CompileError::CannotIndex {
                    type_id: list_type_id,
                    position: list_expression.position(),
                }));
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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting path expression");
        debug_assert_eq!(path_expression.kind(), SyntaxKind::PathExpression);

        let type_id = self.visit_path(path_expression.child()?, true)?;

        self.resolver.add_type_binding(path_expression.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting struct expression");

        let (_struct_name, fields) = node.binary_children()?;

        let declaration_id = *self.resolver.get_declaration_binding(&node.id)?;
        let declared_struct_type_id = *self
            .resolver
            .declarations
            .get_declaration_type(&declaration_id)?;

        for field in fields.children()? {
            let (field_name, field_expression) = field.binary_children()?;

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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting block expression");

        let children = node.children()?;

        let mut block_type_id = TypeId::UNIT;

        for child in children {
            let child_type = if child.is_item() {
                self.visit_item(child);

                TypeId::UNIT
            } else if child.is_statement() {
                self.visit_statement(child);

                TypeId::UNIT
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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting if expression");

        let mut children = node.children()?;

        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.resolver.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(ErrorKind::Compile(
                CompileError::ExpectedBooleanExpression {
                    found: condition_type,
                    node_kind: node.kind(),
                    position: condition.position(),
                },
            ));
        }

        let then_type = self.visit_block_expression(then_block, ())?;

        if let Some(else_block) = else_block {
            let else_type = self.visit_else_expression(else_block, ())?;

            self.resolver.unify_types(
                then_type,
                Some(then_block),
                else_type,
                else_block.child()?,
            )?;
        }

        self.resolver.add_type_binding(node.id, then_type);

        Ok(then_type)
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting else expression");

        self.visit_block_expression(node.child()?, ())
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting comparison binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting logical binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, input)?;

            self.resolver.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, input)?;

            self.resolver.infer_type(raw)?
        };

        if left_type != TypeId::BOOLEAN {
            return Err(ErrorKind::Compile(
                CompileError::ExpectedBooleanExpression {
                    found: left_type,
                    node_kind: left_expression.kind(),
                    position: left_expression.position(),
                },
            ));
        }

        if right_type != TypeId::BOOLEAN {
            return Err(ErrorKind::Compile(
                CompileError::ExpectedBooleanExpression {
                    found: right_type,
                    node_kind: right_expression.kind(),
                    position: right_expression.position(),
                },
            ));
        }

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting unary negation expression");

        let expression = node.child()?;
        let child_type = {
            let raw = self.visit_expression(expression, ())?;

            self.resolver.infer_type(raw)?
        };

        match child_type {
            TypeId::BOOLEAN | TypeId::BYTE | TypeId::FLOAT | TypeId::INTEGER => {
                self.resolver.add_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => Err(ErrorKind::Compile(CompileError::CannotApplyUnaryOperator {
                operator: node.kind(),
                operand_type: self.resolver.get_small_type(child_type, &expression)?,
                operand_position: expression.position(),
            })),
        }
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting while expression");

        let (condition, body) = node.binary_children()?;

        let condition_type = {
            let raw = self.visit_expression(condition, ())?;

            self.resolver.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(ErrorKind::Compile(
                CompileError::ExpectedBooleanExpression {
                    found: condition_type,
                    node_kind: condition.kind(),
                    position: condition.position(),
                },
            ));
        }

        self.visit_block_expression(body, ())?;

        Ok(TypeId::UNIT)
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting function expression");

        let (signature, body) = node.binary_children()?;
        let mut signature_children = signature.children()?;
        let value_parameters = signature_children.expect_next()?;
        let return_type = signature_children.next();

        let mut value_parameter_types = SmallVec::<[TypeId; 8]>::new();

        for parameter_node in value_parameters.children()? {
            let (parameter_name, parameter_type) = parameter_node.binary_children()?;

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
                TypeId::UNIT
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
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting call expression");

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.children()?;

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
            return Err(ErrorKind::Compile(CompileError::ExpectedFunctionType {
                found: callee_type,
                position: callee.position(),
            }));
        };

        let expected_parameters = self
            .resolver
            .types
            .get_type_members(value_parameters)?
            .iter()
            .copied()
            .collect::<SmallVec<[TypeId; 8]>>();

        if arguments.len() != expected_parameters.len() {
            return Err(ErrorKind::Compile(CompileError::ExpectedArguments {
                function_type: callee_type,
                found_position: callee.position(),
                expected_count: expected_parameters.len(),
                found_count: arguments.len(),
            }));
        }

        for (argument, expected_type_id) in arguments.zip(expected_parameters.into_iter()) {
            let argument_type = self.visit_expression(argument, ())?;

            self.resolver
                .unify_types(expected_type_id, None, argument_type, argument)?;
        }

        self.resolver.add_type_binding(node.id, return_type_id);

        Ok(return_type_id)
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind> {
        match node.kind() {
            SyntaxKind::AnyType => Ok(self.resolver.types.create_inferred_type()),
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::ByteType => Ok(TypeId::BYTE),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::FloatType => Ok(TypeId::FLOAT),
            SyntaxKind::IntegerType => Ok(TypeId::INTEGER),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_node = node.child()?;
                let element_type_id = self.visit_type(element_type_node)?;
                let list_type_id = self.resolver.types.add_type(TypeNode::List {
                    element_type: element_type_id,
                });

                Ok(list_type_id)
            }
            SyntaxKind::FunctionType => {
                let type_node = todo!();
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
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_path(
        &mut self,
        path: SyntaxReader,
        local: bool,
    ) -> Result<Self::PathOutput, ErrorKind> {
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
