#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

mod constant_values;
mod let_statement;

#[macro_export]
macro_rules! assert_program_eq {
    (
        $code: literal,
        prototypes: [$($prototype: expr,)*],
        return_type: $return_type: expr
    ) => {
        use $crate::{compiler::Compiler, source::{Source, SourceCode}};

        let mut source = Source::new();

        source.add_code(SourceCode::validated_borrowed("test", $code));

        let program = Compiler::new(source).compile(None).unwrap();
        let mut prototypes = program.prototypes.iter();

        assert_eq!(program.return_type(), &$return_type);

        $(
            let Some(next_prototype) = prototypes.next() else {
                panic!("Fewer prototypes than expected");
            };

            assert_eq!(next_prototype, &$prototype);
        )*
    };
}
