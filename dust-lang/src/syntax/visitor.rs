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

    fn visit_item(&mut self, node: SyntaxReader) -> Result<(), CompileError> {
        match node.kind() {
            SyntaxKind::ModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem => self.visit_function_item(node),
            SyntaxKind::UseItem => self.visit_use_item(node),
            SyntaxKind::StructItem => self.visit_struct_item(node),
            SyntaxKind::EnumItem => self.visit_enum_item(node),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            }),
        }
    }

    fn visit_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Option<Self::StatementOutput>, CompileError> {
        match node.kind() {
            SyntaxKind::ModuleItem => self.visit_module_item(node).map(|_| None),
            SyntaxKind::FunctionItem => self.visit_function_item(node).map(|_| None),
            SyntaxKind::UseItem => self.visit_use_item(node).map(|_| None),
            SyntaxKind::StructItem => self.visit_struct_item(node).map(|_| None),
            SyntaxKind::EnumItem => self.visit_enum_item(node).map(|_| None),
            SyntaxKind::LetStatement => self.visit_let_statement(node).map(Some),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node).map(Some),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            }),
        }
    }

    fn visit_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        match node.kind() {
            SyntaxKind::AssignmentExpression => self.visit_assignment_expression(node),
            SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentAssignmentExpression => {
                self.visit_compound_assignment_expression(node)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(node, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(node, input),
            SyntaxKind::HexadecimalIntegerExpression => self.visit_byte_expression(node, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(node, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(node, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node, input),
            SyntaxKind::StringExpression => self.visit_string_expression(node, input),
            SyntaxKind::ListExpression => self.visit_list_expression(node, input),
            SyntaxKind::IndexExpression => self.visit_index_expression(node, input),
            SyntaxKind::StructExpression => self.visit_struct_expression(node, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_expression(node, input),
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                self.visit_comparison_expression(node, input)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logic_expression(node, input)
            }
            SyntaxKind::BlockExpression => self.visit_block_expression(node, input),
            SyntaxKind::IfExpression => self.visit_if_expression(node, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(node, input),
            SyntaxKind::CallExpression => self.visit_call_expression(node, input),
            SyntaxKind::GroupedExpression => self.visit_expression(node.single_child()?, input),
            _ => Err(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            }),
        }
    }

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, CompileError>;

    fn visit_module_item(&mut self, node: SyntaxReader) -> Result<(), CompileError>;

    fn visit_function_item(&mut self, node: SyntaxReader) -> Result<(), CompileError>;

    fn visit_use_item(&mut self, node: SyntaxReader) -> Result<(), CompileError>;

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), CompileError>;

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), CompileError>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError>;

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError>;

    fn visit_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_compound_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_comparison_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_logic_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError>;

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, CompileError>;

    fn visit_path(
        &mut self,
        node: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError>;

    fn visit_simple_path(
        &mut self,
        node: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError>;
}
