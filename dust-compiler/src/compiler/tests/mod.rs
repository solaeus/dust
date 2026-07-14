#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

mod array_expression;
mod assignment_expression;
mod block_expression;
mod call_expression;
mod comparison_expressions;
mod constant_values;
mod group_expression;
mod hexadecimal_expression;
mod if_expression;
mod let_statement;
mod logic_expressions;
mod math_expressions;
mod method_call_expression;
mod path_expression;
mod range_expression;
mod struct_expression;
mod unary_expressions;
mod while_expression;

#[macro_export]
macro_rules! assert_program_eq {
    ($code: literal, prototypes: $prototypes: expr, return_type: $return_type: expr) => {
        use $crate::{
            compiler::Compiler,
            source::{Source, SourceCode},
        };

        let mut source = Source::new();

        source.add_code(SourceCode::validated("test", $code));

        let program = Compiler::new(source)
            .compile("test_program".to_string())
            .unwrap();

        assert_eq!(program.prototypes(), &$prototypes);
        assert_eq!(program.return_type(), &$return_type);
    };
}
