use crate::{parse, DustError, Value, Vm};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;

    let mut vm = Vm::new(chunk);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_variables() {
        let source = "let foo = 21; let bar = 21; foo + bar";
        let result = run(source);

        assert_eq!(result, Ok(Some(Value::integer(42))));
    }

    #[test]
    fn variable() {
        let source = "let foo = 42; foo";
        let result = run(source);

        assert_eq!(result, Ok(Some(Value::integer(42))));
    }

    #[test]
    fn integer() {
        let source = "42";
        let result = run(source);

        assert_eq!(result, Ok(Some(Value::integer(42))));
    }

    #[test]
    fn addition() {
        let source = "21 + 21";
        let result = run(source);

        assert_eq!(result, Ok(Some(Value::integer(42))));
    }
}
