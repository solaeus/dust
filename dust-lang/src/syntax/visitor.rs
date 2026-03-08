use crate::{
    compiler::error::CompileError,
    error::ErrorKind,
    syntax::{SyntaxKind, SyntaxReader},
};

pub trait SyntaxVisitor {
    type RootOutput;

    type StatementOutput;

    type ExpressionInput;

    type ExpressionOutput;

    type TypeOutput;

    type PathInput;

    type PathOutput;

    fn visit_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        match node.kind() {
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => self.visit_use_item(node),
            SyntaxKind::StructItem | SyntaxKind::PublicStructItem => self.visit_struct_item(node),
            SyntaxKind::EnumItem | SyntaxKind::PublicEnumItem => self.visit_enum_item(node),
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Option<Self::StatementOutput>, ErrorKind> {
        match node.kind() {
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => {
                self.visit_module_item(node).map(|_| None)
            }
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node).map(|_| None)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => {
                self.visit_use_item(node).map(|_| None)
            }
            SyntaxKind::StructItem | SyntaxKind::PublicStructItem => {
                self.visit_struct_item(node).map(|_| None)
            }
            SyntaxKind::EnumItem | SyntaxKind::PublicEnumItem => {
                self.visit_enum_item(node).map(|_| None)
            }
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node).map(Some)
            }
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node).map(Some),
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
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
            SyntaxKind::ByteExpression => self.visit_byte_expression(node, input),
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
            SyntaxKind::FunctionExpression => self.visit_function_expression(node, input),
            SyntaxKind::CallExpression => self.visit_call_expression(node, input),
            SyntaxKind::GroupedExpression => self.visit_expression(node.child()?, input),
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, ErrorKind>;

    fn visit_module_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_function_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_use_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_compound_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_comparison_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_logic_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind>;

    fn visit_path(
        &mut self,
        node: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, ErrorKind>;

    fn visit_simple_path(
        &mut self,
        node: SyntaxReader,
        input: Self::PathInput,
    ) -> Result<Self::PathOutput, ErrorKind>;
}
