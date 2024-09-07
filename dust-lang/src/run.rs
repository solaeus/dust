use crate::{parse, DustError, Value, Vm};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;

    let mut vm = Vm::new(chunk);

    vm.interpret()
        .map_err(|error| DustError::Runtime { error, source })
}

#[cfg(test)]
mod tests {
    use super::*;

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
