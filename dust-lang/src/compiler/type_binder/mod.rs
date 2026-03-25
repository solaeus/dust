use smallvec::SmallVec;

use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    resolver::{
        Resolver,
        declarations::Definition,
        types::{Type, TypeId},
    },
    syntax::{
        Syntax, components::FunctionItem, node::SyntaxKind, reader::SyntaxReader,
        visitor::SyntaxVisitor,
    },
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
        if let Type::Inferred {
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
                Type::Inferred {
                    inferred_id,
                    resolved: None,
                },
                _,
            ) => {
                let left_node = self.resolver.types.get_type_mut(left)?;

                *left_node = Type::Inferred {
                    inferred_id,
                    resolved: Some(right),
                };

                Ok(())
            }
            (
                _,
                Type::Inferred {
                    inferred_id,
                    resolved: None,
                },
            ) => {
                let right_node = self.resolver.types.get_type_mut(right)?;

                *right_node = Type::Inferred {
                    inferred_id,
                    resolved: Some(left),
                };

                Ok(())
            }
            (
                Type::FunctionDefinition {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::FunctionDefinition {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                let left_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(left_declaration_id)?;
                let Definition::Function {
                    public: _,
                    type_parameters: left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type_id: left_return_type_id,
                } = left_declaration.definition
                else {
                    return Err(CompileError::ExpectedFunctionType {
                        found: left,
                        position: left_syntax.unwrap_or(right_syntax).position(),
                    });
                };
                let right_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(right_declaration_id)?;
                let Definition::Function {
                    public: _,
                    type_parameters: right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type_id: right_return_type_id,
                } = right_declaration.definition
                else {
                    return Err(CompileError::ExpectedFunctionType {
                        found: right,
                        position: right_syntax.position(),
                    });
                };

                Ok(())
            }
            (
                Type::Algebraic {
                    declaration_id: left_declaration_id,
                    type_arguments: left_type_arguments,
                },
                Type::Algebraic {
                    declaration_id: right_declaration_id,
                    type_arguments: right_type_arguments,
                },
            ) => {
                if left_declaration_id != right_declaration_id {
                    let expected_position = if let Some(left) = left_syntax {
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
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
                        left.children().next_back().map(|child| child.position())
                    } else {
                        None
                    };
                    let found_position = right_syntax
                        .children()
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

    fn visit_root(&mut self, reader: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug_assert!(reader.node.kind == SyntaxKind::Root);

        for item in reader.children() {
            match self.visit_item(item) {
                Ok(()) => {}
                Err(error) => self.errors.push(ErrorKind::Compile(error)),
            }
        }

        Ok(())
    }

    fn visit_module_item(&mut self, module_item: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_function_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        todo!()
    }

    fn visit_compound_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_comparison_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_logic_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_negation_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        todo!()
    }

    fn visit_path(&mut self, path: SyntaxReader, _: ()) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }

    fn visit_simple_path(
        &mut self,
        simple_path: SyntaxReader,
        _: (),
    ) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }
}
