#[cfg(test)]
mod tests;

use smallvec::SmallVec;
use tracing::debug;

use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind, DeclarationMembers, ModuleKind},
        type_graph::{TypeId, TypeMembers, TypeNode},
    },
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxVisitor},
};

#[derive(Debug)]
pub struct TypeBinder<'a> {
    syntax: &'a Syntax,

    resolver: &'a mut Resolver,

    errors: &'a mut Vec<ErrorKind>,
}

impl<'a> TypeBinder<'a> {
    pub fn new(
        syntax: &'a Syntax,
        resolver: &'a mut Resolver,
        errors: &'a mut Vec<ErrorKind>,
    ) -> Self {
        Self {
            syntax,
            resolver,
            errors,
        }
    }

    pub fn infer_type(&self, type_id: TypeId) -> Result<TypeId, CompileError> {
        if let TypeNode::Inferred {
            resolved: Some(resolved),
            ..
        } = self.resolver.types.get_type(type_id)?
        {
            self.infer_type(*resolved)
        } else {
            Ok(type_id)
        }
    }

    pub fn unify_types(
        &mut self,
        left: TypeId,
        left_syntax: Option<SyntaxReader>,
        right: TypeId,
        right_syntax: SyntaxReader,
    ) -> Result<(), CompileError> {
        if left == right {
            return Ok(());
        }

        let left_inferred = self.infer_type(left)?;
        let right_inferred = self.infer_type(right)?;

        if left_inferred == right_inferred {
            return Ok(());
        }

        self.unify_inferred_types(left_inferred, left_syntax, right_inferred, right_syntax)
    }

    pub fn unify_inferred_types<'b>(
        &'b mut self,
        left: TypeId,
        left_syntax: Option<SyntaxReader<'b>>,
        right: TypeId,
        right_syntax: SyntaxReader<'b>,
    ) -> Result<(), CompileError> {
        let left_type_node = *self.resolver.types.get_type(left)?;
        let right_type_node = *self.resolver.types.get_type(right)?;

        match (left_type_node, right_type_node) {
            (
                TypeNode::Inferred {
                    inferred_id,
                    resolved: None,
                },
                _,
            ) => {
                let left_node = self.resolver.types.get_type_mut(left)?;

                *left_node = TypeNode::Inferred {
                    inferred_id,
                    resolved: Some(right),
                };

                Ok(())
            }
            (
                _,
                TypeNode::Inferred {
                    inferred_id,
                    resolved: None,
                },
            ) => {
                let right_node = self.resolver.types.get_type_mut(right)?;

                *right_node = TypeNode::Inferred {
                    inferred_id,
                    resolved: Some(left),
                };

                Ok(())
            }
            (
                TypeNode::List {
                    element_type_id: left_element_type,
                },
                TypeNode::List {
                    element_type_id: right_element_type,
                },
            ) => self.unify_types(
                left_element_type,
                left_syntax,
                right_element_type,
                right_syntax,
            ),
            (
                TypeNode::Function {
                    type_parameters: _left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type,
                },
                TypeNode::Function {
                    type_parameters: _right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type,
                },
            ) => {
                let left_value_types = self
                    .resolver
                    .types
                    .get_type_members(left_value_parameters)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_value_types = self
                    .resolver
                    .types
                    .get_type_members(right_value_parameters)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();

                for (left_type_id, right_type_id) in
                    left_value_types.into_iter().zip(right_value_types)
                {
                    self.unify_types(left_type_id, left_syntax, right_type_id, right_syntax)?;
                }

                self.unify_types(
                    left_return_type,
                    left_syntax,
                    right_return_type,
                    right_syntax,
                )?;

                Ok(())
            }
            (
                TypeNode::Struct {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                    ..
                },
                TypeNode::Struct {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                    ..
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children()?.next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()?
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    return Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    });
                }

                let left_args = self
                    .resolver
                    .types
                    .get_type_members(left_type_arguments)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();
                let right_args = self
                    .resolver
                    .types
                    .get_type_members(right_type_arguments)?
                    .iter()
                    .copied()
                    .collect::<SmallVec<[TypeId; 8]>>();

                for (left_arg, right_arg) in left_args.iter().zip(right_args.iter()) {
                    self.unify_types(*left_arg, left_syntax, *right_arg, right_syntax)?;
                }

                Ok(())
            }
            (left_type_node, right_type_node) => {
                if left_type_node == right_type_node {
                    Ok(())
                } else {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children()?.next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()?
                        .next_back()
                        .unwrap_or(right_syntax)
                        .position();

                    Err(CompileError::TypeConflict {
                        expected_type: left,
                        expected_position,
                        found_type: right,
                        found_position,
                    })
                }
            }
        }
    }
}

impl SyntaxVisitor for TypeBinder<'_> {
    type RootOutput = ();
    type StatementOutput = ();
    type ExpressionInput = ();
    type ExpressionOutput = TypeId;
    type TypeOutput = TypeId;
    type PathInput = ();
    type PathOutput = TypeId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug!("Visting root");

        let children = node.children()?;

        for child in children {
            match self.visit_item(child) {
                Ok(()) => {}
                Err(error) => {
                    self.errors.push(ErrorKind::Compile(error));
                }
            }
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visting module item");

        let mut children = module_item.children()?;
        let module_name = children.expect_next()?;
        let module_body = children.next();

        if let Some(module_body) = module_body {
            for child in module_body.children()? {
                match self.visit_item(child) {
                    Ok(()) => {}
                    Err(error) => {
                        self.errors.push(ErrorKind::Compile(error));
                    }
                }
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
                return Err(CompileError::ExpectedModuleDeclaration(
                    module_declaration_id,
                ));
            };
            let module_root = self.syntax.get_tree(module_file_id)?.root()?;

            self.visit_root(module_root)?;
        }

        Ok(())
    }

    fn visit_function_item(&mut self, function_item: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visting function item");

        let (function_name, function_expression) = function_item.binary_children()?;

        let function_declaration_id = *self.resolver.get_declaration_binding(&function_name.id)?;
        let function_type_id = self.visit_function_expression(function_expression, None)?;

        self.resolver
            .declarations
            .set_declaration_type(function_declaration_id, function_type_id);

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visting use item");

        Ok(())
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visting struct item");

        let (struct_name, struct_fields) = node.binary_children()?;

        for [field_name, field_type] in struct_fields.children()?.array_chunks::<2>() {
            let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
            let field_type_id = self.visit_type(field_type)?;

            self.resolver
                .declarations
                .set_declaration_type(field_declaration_id, field_type_id);
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

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), CompileError> {
        debug!("Visting enum item");

        let mut children = node.children()?;
        let enum_name = children.expect_next()?;
        let enum_variants = children.expect_next()?;

        let mut variants = SmallVec::<[DeclarationId; 8]>::new();

        for variant in enum_variants.children()? {
            debug!("Visting enum variant");

            let mut variant_children = variant.children()?;
            let variant_name = variant_children.expect_next()?;
            let variant_fields = variant_children.next();

            let variant_declaration_id =
                *self.resolver.get_declaration_binding(&variant_name.id)?;
            let variant_type_id = if let Some(variant_fields) = variant_fields {
                self.resolver.types.add_type(TypeNode::Struct {
                    declaration_id: variant_declaration_id,
                    type_arguments: TypeMembers::default(),
                })
            } else {
                TypeId::UNIT
            };

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

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting let statement");

        let mut children = node.children()?;
        let path = children.expect_next()?;
        let expression = children.expect_next()?;
        let type_notation = children.next();

        let expression_type_id = self.visit_expression(expression, None)?;

        if let Some(type_notation) = type_notation {
            let explicit_type = self.visit_type(type_notation)?;

            match self.unify_types(
                explicit_type,
                Some(type_notation),
                expression_type_id,
                expression,
            ) {
                Ok(()) => {}
                Err(error) => {
                    self.errors.push(ErrorKind::Compile(error));

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

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting expression statement");

        self.visit_expression(node.child()?, None)?;

        Ok(())
    }

    fn visit_compound_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting binary assignment statement");

        let (left, right) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left, None)?;

            self.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right, None)?;

            self.infer_type(raw)?
        };

        let is_character_concatenation = node.kind() == SyntaxKind::AdditionAssignmentExpression
            && ((left_type == TypeId::STRING && right_type == TypeId::CHARACTER)
                || (left_type == TypeId::CHARACTER && right_type == TypeId::STRING)
                || (left_type == TypeId::CHARACTER && right_type == TypeId::CHARACTER));

        if is_character_concatenation {
            return Ok(TypeId::UNIT);
        }

        if let Err(error) = self.unify_inferred_types(left_type, Some(left), right_type, right) {
            self.errors.push(ErrorKind::Compile(error));
        }

        Ok(TypeId::UNIT)
    }

    fn visit_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting reassignment statement");

        let (left, right) = node.binary_children()?;

        let left_type = self.visit_expression(left, None)?;
        let right_type = self.visit_expression(right, None)?;

        self.unify_types(left_type, Some(left), right_type, right)?;
        self.resolver.add_type_binding(left.id, left_type);
        self.resolver.add_type_binding(right.id, right_type);
        self.resolver.add_type_binding(node.id, TypeId::UNIT);

        Ok(TypeId::UNIT)
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting boolean expression");

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting byte expression");

        self.resolver.add_type_binding(node.id, TypeId::U_8);

        Ok(TypeId::U_8)
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting character expression");

        self.resolver.add_type_binding(node.id, TypeId::CHARACTER);

        Ok(TypeId::CHARACTER)
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting float expression");

        self.resolver.add_type_binding(node.id, TypeId::F_64);

        Ok(TypeId::F_64)
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting integer expression");

        self.resolver.add_type_binding(node.id, TypeId::I_64);

        Ok(TypeId::I_64)
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting string expression");

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(TypeId::STRING)
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting list expression");

        let children = node.children()?;

        let mut first_type = None;

        for child in children {
            let child_type = self.visit_expression(child, None)?;

            if let Some((previous_type, previous)) = first_type {
                self.unify_types(previous_type, Some(previous), child_type, child)?;
            } else {
                first_type = Some((child_type, child));
            }
        }

        let element_type_id = if let Some((element_type, _)) = first_type {
            element_type
        } else {
            self.resolver.types.create_inferred_type()
        };
        let list_type = self
            .resolver
            .types
            .add_type(TypeNode::List { element_type_id });

        self.resolver.add_type_binding(node.id, list_type);

        Ok(list_type)
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.binary_children()?;

        let list_type_id = {
            let raw = self.visit_expression(list_expression, input)?;

            self.infer_type(raw)?
        };
        let index_type_id = {
            let raw = self.visit_expression(index_expression, input)?;

            self.infer_type(raw)?
        };

        if index_type_id != TypeId::U_64 {
            return Err(CompileError::ExpectedIntegerIndex {
                found: index_type_id,
                position: index_expression.position(),
            });
        }

        let list_type = *self.resolver.types.get_type(list_type_id)?;
        let element_type = match list_type {
            TypeNode::List {
                element_type_id: element_type,
            } => {
                self.resolver.add_type_binding(node.id, element_type);

                element_type
            }
            _ => {
                return Err(CompileError::CannotIndex {
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
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting path expression");

        let declaration_id = self.resolver.get_declaration_binding(&path_expression.id)?;
        let declaration = self
            .resolver
            .declarations
            .get_declaration(*declaration_id)?;

        let type_id = match declaration.kind {
            DeclarationKind::Local { .. }
            | DeclarationKind::Function { .. }
            | DeclarationKind::NativeFunction(_) => *self
                .resolver
                .declarations
                .get_declaration_type(declaration_id)?,
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: path_expression.kind(),
                    position: path_expression.position(),
                });
            }
        };

        self.resolver.add_type_binding(path_expression.id, type_id);

        Ok(type_id)
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting struct expression");

        let (struct_name, fields) = node.binary_children()?;

        let declaration_id = *self.resolver.get_declaration_binding(&struct_name.id)?;
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
            let actual_field_type_id = self.visit_expression(field_expression, None)?;

            self.unify_types(
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
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting block expression");

        let children = node.children()?;

        let mut block_type_id = TypeId::UNIT;

        for child in children {
            let child_type = if child.is_statement() {
                match self.visit_statement(child) {
                    Ok(_) => {}
                    Err(error) => {
                        self.errors.push(ErrorKind::Compile(error));
                    }
                }

                TypeId::UNIT
            } else {
                self.visit_expression(child, None)?
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
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting if expression");

        let mut children = node.children()?;

        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let condition_type = {
            let raw = self.visit_expression(condition, None)?;

            self.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                    found: condition_type,
                    node_kind: node.kind(),
                    position: condition.position(),
                },
            );
        }

        let then_type = self.visit_block_expression(then_block, None)?;

        if let Some(else_block) = else_block {
            let else_type = self.visit_block_expression(else_block, None)?;

            self.unify_types(then_type, Some(then_block), else_type, else_block.child()?)?;
        }

        self.resolver.add_type_binding(node.id, then_type);

        Ok(then_type)
    }

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, None)?;

            self.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, None)?;

            self.infer_type(raw)?
        };

        let is_character_concatenation = matches!(
            node.kind(),
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression
                if (left_type == TypeId::STRING && right_type == TypeId::CHARACTER)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::STRING)
            || (left_type == TypeId::CHARACTER && right_type == TypeId::CHARACTER)
        );

        let math_expression_type = if is_character_concatenation {
            TypeId::STRING
        } else {
            self.unify_inferred_types(
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

    fn visit_comparison_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting comparison binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = self.visit_expression(left_expression, None)?;
        let right_type = self.visit_expression(right_expression, None)?;

        self.unify_types(
            left_type,
            Some(left_expression),
            right_type,
            right_expression,
        )?;
        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_logic_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting logical binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_type = {
            let raw = self.visit_expression(left_expression, input)?;

            self.infer_type(raw)?
        };
        let right_type = {
            let raw = self.visit_expression(right_expression, input)?;

            self.infer_type(raw)?
        };

        if left_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                    found: left_type,
                    node_kind: left_expression.kind(),
                    position: left_expression.position(),
                },
            );
        }

        if right_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                    found: right_type,
                    node_kind: right_expression.kind(),
                    position: right_expression.position(),
                },
            );
        }

        self.resolver.add_type_binding(node.id, TypeId::BOOLEAN);

        Ok(TypeId::BOOLEAN)
    }

    fn visit_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting unary negation expression");

        let expression = node.child()?;
        let child_type = {
            let raw = self.visit_expression(expression, None)?;

            self.infer_type(raw)?
        };

        match child_type {
            TypeId::BOOLEAN
            | TypeId::U_8
            | TypeId::I_8
            | TypeId::U_16
            | TypeId::I_16
            | TypeId::U_32
            | TypeId::I_32
            | TypeId::U_64
            | TypeId::I_64
            | TypeId::U_128
            | TypeId::I_128
            | TypeId::F_32
            | TypeId::F_64 => {
                self.resolver.add_type_binding(node.id, child_type);

                Ok(child_type)
            }
            _ => Err(CompileError::CannotApplyOperator {
                operator: node.kind(),
                type_id: child_type,
                operand_position: expression.position(),
            }),
        }
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting while expression");

        let (condition, body) = node.binary_children()?;

        let condition_type = {
            let raw = self.visit_expression(condition, None)?;

            self.infer_type(raw)?
        };

        if condition_type != TypeId::BOOLEAN {
            return Err(CompileError::ExpectedBooleanExpression {
                    found: condition_type,
                    node_kind: condition.kind(),
                    position: condition.position(),
                },
            );
        }

        self.visit_block_expression(body, None)?;

        Ok(TypeId::UNIT)
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting function expression");

        let (signature, body) = node.binary_children()?;
        let mut signature_children = signature.children()?;
        let parameters = signature_children.expect_next()?;
        let return_type = signature_children.next();
        let mut parameter_children = parameters.children()?;
        let value_parameters = parameter_children.expect_next()?;
        let type_parameters = parameter_children.next();

        let mut value_parameter_type_ids = SmallVec::<[TypeId; 8]>::new();

        for [parameter_name, parameter_type] in value_parameters.children()?.array_chunks::<2>() {
            let parameter_declaration_id =
                *self.resolver.get_declaration_binding(&parameter_name.id)?;
            let parameter_type_id = self.visit_type(parameter_type)?;

            value_parameter_type_ids.push(parameter_type_id);
            self.resolver
                .declarations
                .set_declaration_type(parameter_declaration_id, parameter_type_id);
        }

        let type_parameters = if let Some(type_parameters) = type_parameters {
            let mut type_parameter_declaration_ids = SmallVec::<[DeclarationId; 8]>::new();

            for type_parameter in type_parameters.children()? {
                let type_parameter_declaration_id =
                    *self.resolver.get_declaration_binding(&type_parameter.id)?;
                let type_parameter_type_id = self.resolver.types.create_inferred_type();

                type_parameter_declaration_ids.push(type_parameter_declaration_id);
                self.resolver
                    .declarations
                    .set_declaration_type(type_parameter_declaration_id, type_parameter_type_id);
            }

            self.resolver
                .declarations
                .add_declaration_members(&type_parameter_declaration_ids)
        } else {
            DeclarationMembers::default()
        };

        let value_parameters = self
            .resolver
            .types
            .add_type_members(&value_parameter_type_ids);
        let expected_return_type_id = {
            if let Some(return_type_node) = return_type {
                let raw = self.visit_type(return_type_node)?;

                self.infer_type(raw)?
            } else {
                TypeId::UNIT
            }
        };
        let actual_return_type_id = self.visit_block_expression(body, None)?;

        self.unify_types(
            expected_return_type_id,
            return_type,
            actual_return_type_id,
            body,
        )?;

        let function_type_id = self.resolver.types.add_type(TypeNode::Function {
            type_parameters,
            value_parameters,
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
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting call expression");

        let (callee, arguments_list) = node.binary_children()?;
        let arguments = arguments_list.children()?;

        let callee_type = {
            let raw = self.visit_expression(callee, None)?;

            self.infer_type(raw)?
        };

        let TypeNode::Function {
            value_parameters,
            return_type_id,
            ..
        } = *self.resolver.types.get_type(callee_type)?
        else {
            return Err(CompileError::ExpectedFunctionType {
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
            return Err(CompileError::ExpectedArguments {
                function_type: callee_type,
                found_position: callee.position(),
                expected_count: expected_parameters.len(),
                found_count: arguments.len(),
            });
        }

        for (argument, expected_type_id) in arguments.zip(expected_parameters.into_iter()) {
            let argument_type = self.visit_expression(argument, None)?;

            self.unify_types(expected_type_id, None, argument_type, argument)?;
        }

        self.resolver.add_type_binding(node.id, return_type_id);

        Ok(return_type_id)
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        match node.kind() {
            SyntaxKind::AnyType => Ok(self.resolver.types.create_inferred_type()),
            SyntaxKind::BooleanType => Ok(TypeId::BOOLEAN),
            SyntaxKind::U8Type => Ok(TypeId::U_8),
            SyntaxKind::CharacterType => Ok(TypeId::CHARACTER),
            SyntaxKind::F64Type => Ok(TypeId::F_64),
            SyntaxKind::I64Type => Ok(TypeId::I_64),
            SyntaxKind::StringType => Ok(TypeId::STRING),
            SyntaxKind::ListType => {
                let element_type_node = node.child()?;
                let element_type_id = self.visit_type(element_type_node)?;
                let list_type_id = self
                    .resolver
                    .types
                    .add_type(TypeNode::List { element_type_id });

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
            _ => Err(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            }),
        }
    }

    fn visit_path(&mut self, path: SyntaxReader, _: ()) -> Result<Self::PathOutput, CompileError> {
        debug!("Visting path");
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        self.resolver
            .get_declaration_binding(&path.id)
            .and_then(|declaration_id| {
                self.resolver
                    .declarations
                    .get_declaration_type(declaration_id)
            })
            .copied()
            .map_err(CompileError::from)
    }

    fn visit_simple_path(
        &mut self,
        simple_path: SyntaxReader,
        _: (),
    ) -> Result<Self::PathOutput, CompileError> {
        debug!("Visiting simple path");
        debug_assert_eq!(simple_path.kind(), SyntaxKind::SimplePath);

        self.resolver
            .get_declaration_binding(&simple_path.id)
            .and_then(|declaration_id| {
                self.resolver
                    .declarations
                    .get_declaration_type(declaration_id)
            })
            .copied()
            .map_err(CompileError::from)
    }
}
