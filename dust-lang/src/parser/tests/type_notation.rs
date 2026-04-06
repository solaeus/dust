use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::SourceFileId,
};

#[test]
fn type_notations() {
    let cases = [
        "type Foo = bool;",
        "type Foo = char;",
        "type Foo = str;",
        "type Foo = i8;",
        "type Foo = i16;",
        "type Foo = i32;",
        "type Foo = i64;",
        "type Foo = i128;",
        "type Foo = isize;",
        "type Foo = u8;",
        "type Foo = u16;",
        "type Foo = u32;",
        "type Foo = u64;",
        "type Foo = u128;",
        "type Foo = usize;",
        "type Foo = f32;",
        "type Foo = f64;",
        "type Foo = !;",
        "type Foo = Bar;",
        "type Foo = Bar<i32>;",
        "type Foo = [i32; 3];",
        "type Foo = [i32];",
        "type Foo = (i32, bool);",
        "type Foo = fn(i32) -> bool;",
    ];

    for source in cases {
        let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(source.as_bytes()));
        let ParseResult { errors, .. } = parser.parse();

        assert!(errors.is_empty(), "{source}: {errors:#?}");
    }
}
