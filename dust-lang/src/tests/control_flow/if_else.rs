use crate::{Value, run};

mod boolean_returns {
    use super::*;

    #[test]
    fn if_true_else() {
        let source = "if true { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else() {
        let source = "if false { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else() {
        let source = "if true { 42 } else if true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else() {
        let source = "if true { 42 } else if false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else() {
        let source = "if false { 0 } else if true { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else() {
        let source = "if false { 0 } else if false { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_true_else() {
        let source = "if true { 42 } else if true { 0 } else if true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_false_else() {
        let source = "if true { 42 } else if true { 0 } else if false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else_if_true_else() {
        let source = "if true { 42 } else if false { 0 } else if true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else_if_false_else() {
        let source = "if true { 42 } else if false { 0 } else if false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_true_else() {
        let source = "if false { 0 } else if true { 42 } else if true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_false_else() {
        let source = "if false { 0 } else if true { 42 } else if false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_true_else() {
        let source = "if false { 0 } else if false { 0 } else if true { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else() {
        let source = "if false { 0 } else if false { 0 } else if false { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }
}

mod comparison_returns {
    use super::*;

    #[test]
    fn if_true_else() {
        let source = "if 42 == 42 { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else() {
        let source = "if 42 != 42 { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else() {
        let source = "if 42 != 42 { 0 } else if 42 > 0 { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else() {
        let source = "if 42 != 42 { 0 } else if 0 > 42 { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_true_else() {
        let source = "if 0 > 42 { 1 } else if 0 > 42 { 2 } else if 50 > 42 { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else() {
        let source = "if 0 > 42 { 1 } else if 0 > 42 { 2 } else if 42 > 50 { 3 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else() {
        let source = "if 100 > 50 { 42 } else if 0 > 42 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else() {
        let source = "if 100 > 50 { 42 } else if 50 > 0 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_false_else() {
        let source = "if 0 > 42 { 1 } else if 42 > 0 { 42 } else if 42 > 50 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_true_else() {
        let source = "if 0 > 42 { 1 } else if 42 > 0 { 42 } else if 50 > 42 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else_if_false_else() {
        let source = "if 42 > 0 { 42 } else if 0 > 42 { 0 } else if 42 > 50 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_false_else() {
        let source = "if 42 > 0 { 42 } else if 50 > 0 { 0 } else if 42 > 50 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_true_else() {
        let source = "if 42 > 0 { 42 } else if 50 > 0 { 0 } else if 100 > 50 { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else_if_true_else() {
        let source = "if 0 > 42 { 1 } else if 0 > 42 { 2 } else if 42 > 50 { 3 } else if 50 > 42 { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else_if_false_else() {
        let source = "if 0 > 42 { 1 } else if 0 > 42 { 2 } else if 42 > 50 { 3 } else if 42 > 50 { 4 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }
}

mod logic_returns {
    use super::*;

    #[test]
    fn if_true_else() {
        let source = "if true && true { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else() {
        let source = "if true && false { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else() {
        let source = "if false && true { 0 } else if true || false { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else() {
        let source = "if false || false { 0 } else if false && true { 0 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_true_else() {
        let source = "if false && true { 1 } else if false || false { 2 } else if true && true { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else() {
        let source = "if false || false { 1 } else if false && false { 2 } else if false || false { 3 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else() {
        let source = "if true || false { 42 } else if false && true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else() {
        let source = "if true && true { 42 } else if true || false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_false_else() {
        let source = "if false || false { 1 } else if true && true { 42 } else if false || true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_true_else_if_true_else() {
        let source = "if false && false { 1 } else if true || false { 42 } else if true && true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_false_else_if_false_else() {
        let source = "if true || false { 42 } else if false && true { 0 } else if false || false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_false_else() {
        let source = "if true && true { 42 } else if true || false { 0 } else if false && true { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_true_else_if_true_else_if_true_else() {
        let source = "if true || true { 42 } else if true && true { 0 } else if true || false { 0 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else_if_true_else() {
        let source = "if false && false { 1 } else if false || false { 2 } else if false && false { 3 } else if true || false { 42 } else { 0 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }

    #[test]
    fn if_false_else_if_false_else_if_false_else_if_false_else() {
        let source = "if false || false { 1 } else if false && false { 2 } else if false || false { 3 } else if false && false { 4 } else { 42 }";
        let return_value = Some(Value::integer(42));

        assert_eq!(return_value, run(source).unwrap());
    }
}
