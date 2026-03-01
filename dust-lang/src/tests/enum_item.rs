pub const EMPTY_VARIANT: &[u8] = b"enum Foo { Bar }";
pub const TUPLE_VARIANT: &[u8] = b"enum Foo { Bar(int, int) }";
pub const FIELDS_VARIANT: &[u8] = b"enum Foo { Bar { x: int, y: int } }";
pub const MIXED_VARIANTS: &[u8] = b"enum Foo { Bar, Baz(int), Qux { x: int } }";
pub const TYPE_PARAMETERS: &[u8] = b"enum Foo<A, B, C> { Bar }";
