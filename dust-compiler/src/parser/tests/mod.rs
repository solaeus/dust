mod as_expression;
mod assignment_expression;
mod binary_assignment_expressions;
mod binary_expressions;
mod block_expression;
mod call_expression;
mod const_item;
mod enum_item;
mod field_access_expression;
mod function_item;
mod grouped_expression;
mod if_expression;
mod impl_item;
mod index_expression;
mod let_statement;
mod loop_expressions;
mod method_call_expression;
mod module_item;
mod path_expression;
mod precedence;
mod range_expression;
mod struct_expression;
mod struct_item;
mod trait_item;
mod type_item;
mod type_notation;
mod unary_expressions;
mod unit_struct_item;
mod use_item;
mod value_expressions;

#[macro_export]
macro_rules! function_wrapper {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}
