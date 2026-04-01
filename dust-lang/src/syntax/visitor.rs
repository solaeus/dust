use crate::{
    compiler::error::CompileError,
    syntax::{node::SyntaxKind, reader::SyntaxReader},
};

pub trait SyntaxVisitor {
    type RootOutput;

    type StatementOutput;

    type ExpressionInput;

    type ExpressionOutput;

    type TypeOutput;

    type PathInput;

    type PathOutput;

    fn visit_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        match reader.node.kind {
            SyntaxKind::ModuleItem => self.visit_module_item(reader),
            SyntaxKind::FunctionItem => self.visit_function_item(reader),
            SyntaxKind::UseItem => self.visit_use_item(reader),
            SyntaxKind::StructItem => self.visit_struct_item(reader),
            SyntaxKind::EnumItem => self.visit_enum_item(reader),
            SyntaxKind::ConstItem => self.visit_const_item(reader),
            SyntaxKind::TypeItem => self.visit_type_item(reader),
            SyntaxKind::ImplItem => self.visit_impl_item(reader),
            SyntaxKind::ImplTraitItem => self.visit_impl_trait_item(reader),
            SyntaxKind::TraitItem => self.visit_trait_item(reader),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: reader.node.kind,
                position: reader.position(),
            }),
        }
    }

    fn visit_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Option<Self::StatementOutput>, CompileError> {
        match reader.node.kind {
            SyntaxKind::ModuleItem => self.visit_module_item(reader).map(|_| None),
            SyntaxKind::FunctionItem => self.visit_function_item(reader).map(|_| None),
            SyntaxKind::UseItem => self.visit_use_item(reader).map(|_| None),
            SyntaxKind::StructItem => self.visit_struct_item(reader).map(|_| None),
            SyntaxKind::EnumItem => self.visit_enum_item(reader).map(|_| None),
            SyntaxKind::ConstItem => self.visit_const_item(reader).map(|_| None),
            SyntaxKind::TypeItem => self.visit_type_item(reader).map(|_| None),
            SyntaxKind::ImplItem => self.visit_impl_item(reader).map(|_| None),
            SyntaxKind::ImplTraitItem => self.visit_impl_trait_item(reader).map(|_| None),
            SyntaxKind::TraitItem => self.visit_trait_item(reader).map(|_| None),
            SyntaxKind::LetStatement => self.visit_let_statement(reader).map(Some),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(reader).map(Some),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: reader.node.kind,
                position: reader.position(),
            }),
        }
    }

    fn visit_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        match reader.node.kind {
            SyntaxKind::AssignmentExpression => self.visit_assignment_expression(reader),
            SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentAssignmentExpression => {
                self.visit_compound_assignment_expression(reader)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(reader, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(reader, input),
            SyntaxKind::HexadecimalIntegerExpression => self.visit_byte_expression(reader, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(reader, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(reader, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(reader, input),
            SyntaxKind::StringExpression => self.visit_string_expression(reader, input),
            SyntaxKind::ArrayExpression => self.visit_array_expression(reader, input),
            SyntaxKind::ArrayRepeatExpression => self.visit_array_repeat_expression(reader, input),
            SyntaxKind::IndexExpression => self.visit_index_expression(reader, input),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.visit_range_expression(reader, input)
            }
            SyntaxKind::StructExpression => self.visit_struct_expression(reader, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_expression(reader, input),
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                self.visit_comparison_expression(reader, input)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logic_expression(reader, input)
            }
            SyntaxKind::NotExpression => self.visit_not_expression(reader, input),
            SyntaxKind::GroupedExpression => self.visit_grouped_expression(reader, input),
            SyntaxKind::BlockExpression => self.visit_block_expression(reader, input),
            SyntaxKind::IfExpression => self.visit_if_expression(reader, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(reader, input),
            SyntaxKind::CallExpression => self.visit_call_expression(reader, input),
            SyntaxKind::FieldAccessExpression => self.visit_field_access_expression(reader, input),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: reader.node.kind,
                position: reader.position(),
            }),
        }
    }

    fn visit_root(&mut self, reader: SyntaxReader) -> Result<Self::RootOutput, CompileError>;

    fn visit_module_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_function_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_use_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_struct_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_enum_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_const_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_type_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_impl_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError>;

    fn visit_expression_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError>;

    fn visit_let_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError>;

    fn visit_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_compound_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_boolean_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_byte_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_character_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_float_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_integer_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_string_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_array_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_range_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_path_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_block_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_if_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_math_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_logic_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_negation_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_not_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_while_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_type(&mut self, reader: SyntaxReader) -> Result<Self::TypeOutput, CompileError>;

    fn visit_path(
        &mut self,
        reader: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError>;

    fn visit_simple_path(
        &mut self,
        reader: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError>;
}
