use dust_lang::{
    abstract_tree::Type,
    error::{Error, TypeCheckError, ValidationError},
    *,
};

#[test]
fn set_and_get_variable() {
    assert_eq!(interpret("foobar = true; foobar"), Ok(Value::boolean(true)));
}

#[test]
fn set_variable_with_type() {
    assert_eq!(
        interpret("foobar: bool = true; foobar"),
        Ok(Value::boolean(true))
    );
}

#[test]
fn set_variable_with_type_error() {
    assert_eq!(
        interpret("foobar: str = true"),
        Err(vec![Error::Validation {
            error: ValidationError::TypeCheck(TypeCheckError {
                actual: Type::Boolean,
                expected: Type::String
            }),
            span: (0..18).into()
        }])
    );
}