use std::{collections::HashMap, sync::OnceLock};

use crate::{BuiltInFunction, Context, ContextData, Function, Identifier, Value};

static CORE_LIBRARY: OnceLock<Context> = OnceLock::new();

pub fn core_library<'a>() -> &'a Context {
    CORE_LIBRARY.get_or_init(|| {
        Context::with_data(HashMap::from([
            (
                Identifier::new("to_string"),
                (
                    ContextData::VariableValue(Value::function(Function::BuiltIn(
                        BuiltInFunction::ToString,
                    ))),
                    0,
                ),
            ),
            (
                Identifier::new("read_line"),
                (
                    ContextData::VariableValue(Value::function(Function::BuiltIn(
                        BuiltInFunction::ReadLine,
                    ))),
                    0,
                ),
            ),
            (
                Identifier::new("write_line"),
                (
                    ContextData::VariableValue(Value::function(Function::BuiltIn(
                        BuiltInFunction::WriteLine,
                    ))),
                    0,
                ),
            ),
        ]))
    })
}
