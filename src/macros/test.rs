use crate::{Macro, MacroInfo, Result, Value};

pub struct Assert;

impl Macro for Assert {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "assert",
            description: "Panic if a boolean is false.",
            group: "test",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let boolean = argument.as_boolean()?;

        assert!(boolean);

        Ok(Value::Empty)
    }
}

pub struct AssertEqual;

impl Macro for AssertEqual {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "assert_equal",
            description: "Panic if two values do not match.",
            group: "test",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let arguments = argument.as_fixed_len_list(2)?;
        assert_eq!(arguments[0], arguments[1]);

        Ok(Value::Empty)
    }
}
