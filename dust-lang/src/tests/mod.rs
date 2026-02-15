pub const FUNCTION_ITEM: &[u8] = br"
    fn main() {}
";

pub const LET_STATEMENT: &[u8] = br"
    fn main() {
        let x = 42;
    }
";

pub const REASSIGNMENT_STATEMENT: &[u8] = br"
    fn main() {
        x = 42;
    }
";

pub const BINARY_ASSIGNMENT_STATEMENT: &[u8] = br"
    fn main() {
        x += 42;
    }
";
