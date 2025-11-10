pub const IF_ELSE_TRUE: &str = r#"
if true {
    42
} else {
    0
}
"#;

pub const IF_ELSE_FALSE: &str = r#"
if false {
    0
} else {
    42
}
"#;

pub const IF_ELSE_LOGICAL_AND: &str = r#"
let a = true;
let b = true;

if a && b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_LOGICAL_OR: &str = r#"
let a = false;
let b = true;

if a || b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_EQUAL: &str = r#"
let a = 0;
let b = 0;

if a == b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_NOT_EQUAL: &str = r#"
let a = 0;
let b = 1;

if a != b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_LESS_THAN: &str = r#"
let a = 0;
let b = 1;

if a < b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_GREATER_THAN: &str = r#"
let a = 1;
let b = 0;

if a > b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_LESS_THAN_EQUAL: &str = r#"
let a = 0;
let b = 0;

if a <= b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_GREATER_THAN_EQUAL: &str = r#"
let a = 1;
let b = 0;

if a >= b {
    42
} else {
    0
}
"#;

pub const IF_ELSE_IF_CHAIN_END: &str = r#"
let a = 2;
let b = 1;

if a < b {
    0
} else if a == b {
    1
} else {
    42
}
"#;

pub const IF_ELSE_IF_CHAIN_MIDDLE: &str = r#"
let a = 1;
let b = 1;

if a < b {
    0
} else if a == b {
    42
} else {
    1
}
"#;

pub const IF_ELSE_NESTED: &str = r#"
let a = 1;
let b = 2;

if a < b {
    if b > a {
        42
    } else {
        0
    }
} else {
    0
}
"#;

pub const IF_ELSE_DOUBLE_NESTED: &str = r#"
let a = 1;
let b = 2;

if a < b {
    if b > a {
        if a != 0 {
            42
        } else {
            0
        }
    } else {
        0
    }
} else {
    0
}
"#;
