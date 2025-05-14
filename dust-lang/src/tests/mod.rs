macro_rules! create_vm_test {
    ($title: ident, $snippet: expr, $expected: expr) => {
        #[test]
        fn $title() {
            assert_eq!($expected, crate::panic_vm::run($snippet).unwrap());
        }
    };
}

pub mod values;
