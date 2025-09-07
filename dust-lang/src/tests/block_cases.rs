pub const EMPTY_BLOCK: &str = "{}";
pub const BLOCK_EXPRESSION: &str = "{ 42 }";
pub const BLOCK_STATEMENT: &str = "{ let a: int = 42; }";
pub const BLOCK_STATEMENT_AND_EXPRESSION: &str = "{ let a: int = 42; a + 1 }";

pub const PARENT_SCOPE_ACCESS: &str = r#"
{
    let a: int = 42;
    { a }
}
"#;
pub const NESTED_PARRENT_SCOPE_ACCESS: &str = r#"
{
    let a: int = 41;
    {
        let b: int = 1;
        {
            a + b
        }
    }
}
"#;

pub const SCOPE_SHADOWING: &str = r#"
{
    let a: int = 42;
    {
        let a: int = 43;
        a
    }
}
"#;
pub const SCOPE_DESHADOWING: &str = r#"
{
    let a: int = 42;
    {
        let b: int = 1;
    }
    a
}
"#;
