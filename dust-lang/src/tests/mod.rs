macro_rules! test_case {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}

pub const FUNCTION_ITEM: &[u8] = test_case!("");

pub const LET_STATEMENT: &[u8] = test_case!("let x = 42;");

pub const LET_STATEMENT_WITH_TYPE: &[u8] = test_case!("let x: int = 42;");

pub const LET_MUT_STATEMENT: &[u8] = test_case!("let mut x = 42;");

pub const LET_MUT_STATEMENT_WITH_TYPE: &[u8] = test_case!("let mut x: int = 42;");

pub const REASSIGNMENT_STATEMENT: &[u8] = test_case!("x = 42;");

pub mod binary_assignment_statement {
    pub const ADD_ASSIGN: &[u8] = test_case!("x += 42;");
    pub const SUBTRACT_ASSIGN: &[u8] = test_case!("x -= 42;");
    pub const MULTIPLY_ASSIGN: &[u8] = test_case!("x *= 42;");
    pub const DIVIDE_ASSIGN: &[u8] = test_case!("x /= 42;");
    pub const MODULO_ASSIGN: &[u8] = test_case!("x %= 42;");
    pub const POWER_ASSIGN: &[u8] = test_case!("x ^= 42;");
}

pub mod value_expression {
    pub const BOOLEAN: &[u8] = test_case!("true");
    pub const BYTE: &[u8] = test_case!("0x2A");
    pub const CHARACTER: &[u8] = test_case!("'a'");
    pub const FLOAT: &[u8] = test_case!("3.14");
    pub const INTEGER: &[u8] = test_case!("42");
    pub const STRING: &[u8] = test_case!("\"Hello, world!\"");
    pub const LIST: &[u8] = test_case!("[1, 2, 3]");
    pub const FUNCTION: &[u8] = test_case!("fn() {}");
}

pub mod unary_expression {
    pub const NEGATION: &[u8] = test_case!("-x");
    pub const LOGICAL_NOT: &[u8] = test_case!("!x");
}

pub mod binary_expression {
    pub const ADDITION: &[u8] = test_case!("x + y");
    pub const SUBTRACTION: &[u8] = test_case!("x - y");
    pub const MULTIPLICATION: &[u8] = test_case!("x * y");
    pub const DIVISION: &[u8] = test_case!("x / y");
    pub const MODULO: &[u8] = test_case!("x % y");
    pub const POWER: &[u8] = test_case!("x ^ y");

    pub const EQUAL: &[u8] = test_case!("x == y");
    pub const NOT_EQUAL: &[u8] = test_case!("x != y");
    pub const LESS_THAN: &[u8] = test_case!("x < y");
    pub const LESS_THAN_OR_EQUAL: &[u8] = test_case!("x <= y");
    pub const GREATER_THAN: &[u8] = test_case!("x > y");
    pub const GREATER_THAN_OR_EQUAL: &[u8] = test_case!("x >= y");

    pub const LOGICAL_AND: &[u8] = test_case!("x && y");
    pub const LOGICAL_OR: &[u8] = test_case!("x || y");
}

pub const GROUPED_EXPRESSION: &[u8] = test_case!("(x + y)");

pub mod block_expression {
    pub const EMPTY: &[u8] = test_case!("{}");
    pub const ITEM: &[u8] = test_case!("{ fn foo() {} }");
    pub const STATEMENT: &[u8] = test_case!("{ let x = 42; }");
    pub const EXPRESSION: &[u8] = test_case!("{ x + y }");
    pub const MIXED: &[u8] = test_case!("{ fn foo() {} let x = 42; x + y }");
}

pub mod if_expression {
    pub const IF: &[u8] = test_case!("if condition { x + y }");
    pub const IF_ELSE: &[u8] = test_case!("if condition { x + y } else { x - y }");
    pub const IF_ELSE_IF: &[u8] =
        test_case!("if left { x + y } else if right { x - y } else { x * y }");
}
