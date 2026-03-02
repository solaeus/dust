pub const EMPTY: &[u8] = b"fn foo() {}";
pub const VALUE_PARAMETERS: &[u8] = b"fn foo(x: int, y: bool) {}";
pub const TYPE_PARAMETERS: &[u8] = b"fn foo<A, B, C>() {}";
pub const RETURN_TYPE: &[u8] = b"fn foo() -> int {}";
pub const MIXED: &[u8] = b"fn foo<A, B, C>(x: A, y: B) -> C {}";
