use crate::{
    dust_error::{DustError, InternalError},
    syntax::{SyntaxKind, SyntaxReader},
};

pub trait SyntaxVisitor {
    type RootOutput;

    type ItemOutput;

    type StatementOutput;

    type ExpressionInput;

    type ExpressionOutput;

    type TypeOutput;

    type PathOutput;

    fn visit_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError> {
        match node.kind() {
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => self.visit_use_item(node),
            SyntaxKind::StructItem | SyntaxKind::PublicStructItem => self.visit_struct_item(node),
            _ => Err(DustError::Internal(InternalError::UnimplementedFeature(
                node.kind(),
            ))),
        }
    }

    fn visit_statement(&mut self, node: SyntaxReader) -> Result<Self::StatementOutput, DustError> {
        match node.kind() {
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node)
            }
            SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::ExponentAssignmentStatement => {
                self.visit_binary_assignment_statement(node)
            }
            _ => Err(DustError::Internal(InternalError::UnimplementedFeature(
                node.kind(),
            ))),
        }
    }

    fn visit_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError> {
        match node.kind() {
            SyntaxKind::PathExpression => self.visit_path_expression(node, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(node, input),
            SyntaxKind::ByteExpression => self.visit_byte_expression(node, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(node, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(node, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node, input),
            SyntaxKind::StringExpression => self.visit_string_expression(node, input),
            SyntaxKind::ListExpression => self.visit_list_expression(node, input),
            SyntaxKind::ListIndexExpression => self.visit_index_expression(node, input),
            SyntaxKind::StructExpression => self.visit_struct_expression(node, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_binary_expression(node, input),
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                self.visit_comparison_binary_expression(node, input)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logical_binary_expression(node, input)
            }
            SyntaxKind::BlockExpression => self.visit_block_expression(node, input),
            SyntaxKind::IfExpression => self.visit_if_expression(node, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(node, input),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node, input),
            SyntaxKind::CallExpression => self.visit_call_expression(node, input),
            SyntaxKind::GroupedExpression => {
                self.visit_expression(node.expect_left_child()?, input)
            }
            _ => Err(DustError::Internal(InternalError::UnimplementedFeature(
                node.kind(),
            ))),
        }
    }

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, DustError>;

    fn visit_module_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError>;

    fn visit_function_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError>;

    fn visit_use_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError>;

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<Self::ItemOutput, DustError>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError>;

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError>;

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError>;

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, DustError>;

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, DustError>;

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, DustError>;

    fn visit_path(&mut self, node: SyntaxReader) -> Result<Self::PathOutput, DustError>;
}
