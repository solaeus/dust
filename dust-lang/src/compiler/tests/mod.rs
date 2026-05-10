#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

mod constant_values;
mod let_statement;

#[macro_export]
macro_rules! assert_program_eq {
    (
        $code: literal,
        prototypes: $prototypes: expr,
        return_type: $return_type: expr
    ) => {
        use $crate::{
            compiler::Compiler,
            source::{Source, SourceCode},
        };

        let mut source = Source::new();

        source.add_code(SourceCode::from_str("test", $code));

        let program = Compiler::new(source).compile(None).unwrap();

        assert_eq!(program.prototypes, $prototypes);
        assert_eq!(program.return_type(), &$return_type);
    };
}
