use dust_lang::*;

#[test]
fn define_enum() {
    interpret(
        "
        enum FooBar(F) {
            Foo(F),
            Bar,    
        }

        foo = FooBar::Foo(1)
        foo
        ",
    )
    .unwrap();
}
