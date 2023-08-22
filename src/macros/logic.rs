use crate::{Error, Macro, MacroInfo, Result, Value, ValueType};


pub struct Assert;

impl Macro for Assert {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "assert",
            description: "Panic if a boolean is false.",
            group: "test",
            inputs: vec![
                ValueType::Boolean,
                ValueType::Function
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let boolean = argument.as_boolean()?;

        if boolean {        
            Ok(Value::Empty)
        } else {
            Err(Error::AssertFailed)
        }

    }
}

pub struct AssertEqual;

impl Macro for AssertEqual {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "assert_equal",
            description: "Panic if two values do not match.",
            group: "test",
            inputs: vec![ValueType::List(vec![ValueType::Any, ValueType::Any])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let arguments = argument.as_fixed_len_list(2)?;

        if arguments[0] == arguments[1] {
        Ok(Value::Empty)
        } else {
            Err(Error::AssertEqualFailed { expected: arguments[0].clone(), actual: arguments[1].clone() })
        }

    }
}

pub struct If;

impl Macro for If {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "if",
            description: "Evaluates the first argument. If true, it does the second argument. If false, it does the third argument",            
            group: "general",
            inputs: vec![
                ValueType::List(vec![
                    ValueType::Boolean,
                    ValueType::Any,
                    ValueType::Any
                ]), 
                ValueType::List(vec![
                    ValueType::Function, 
                    ValueType::Any,
                    ValueType::Any
                ])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_fixed_len_list(3)?;
        let (condition, if_true, if_false) = (&argument[0], &argument[1], &argument[2]);

        let condition_is_true = if let Ok(boolean) = condition.as_boolean() {
            boolean
        } else if let Ok(function) = condition.as_function() {
            function.run()?.as_boolean()?
        } else {
            return Err(Error::TypeError {
                expected: &[ValueType::Boolean, ValueType::Function],
                actual: condition.clone(),
            });
        };

        let should_yield = if condition_is_true { if_true } else { if_false };

        if let Ok(function) = should_yield.as_function() {
            function.run()
        } else {
            Ok(should_yield.clone())
        }
    }
}

pub struct Loop;

impl Macro for Loop {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "loop",
            description: "Repeats a function until the program ends.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let function = argument.as_function()?;

        function.run()?;

        Loop.run(argument)
    }
}

pub struct While;

impl Macro for While {
    fn info(&self) -> MacroInfo<'static> {
        todo!()
    }

    fn run(&self, _argument: &Value) -> Result<Value> {
        todo!()
    }
}
