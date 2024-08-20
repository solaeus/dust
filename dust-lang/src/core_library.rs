use std::{collections::HashMap, sync::OnceLock};

use crate::{BuiltInFunction, Context, ContextData, Function, Identifier, Value};

static CORE_LIBRARY: OnceLock<Context> = OnceLock::new();

pub fn core_library<'a>() -> &'a Context {
    CORE_LIBRARY.get_or_init(|| {
        Context::with_data(HashMap::from([
            (
                Identifier::new("to_string"),
                (
                    ContextData::VariableValue(Value::Function(Function::BuiltIn(
                        BuiltInFunction::ToString,
                    ))),
                    (0, 0),
                ),
            ),
            (
                Identifier::new("is_even"),
                (
                    ContextData::VariableValue(Value::Function(Function::BuiltIn(
                        BuiltInFunction::IsEven,
                    ))),
                    (0, 0),
                ),
            ),
            (
                Identifier::new("is_odd"),
                (
                    ContextData::VariableValue(Value::Function(Function::BuiltIn(
                        BuiltInFunction::IsOdd,
                    ))),
                    (0, 0),
                ),
            ),
            (
                Identifier::new("length"),
                (
                    ContextData::VariableValue(Value::Function(Function::BuiltIn(
                        BuiltInFunction::Length,
                    ))),
                    (0, 0),
                ),
            ),
        ]))
    })
}
