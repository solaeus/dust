use core::fmt;
use std::{
    fmt::{Display, Formatter},
    io::stdin,
    thread,
    time::Duration,
};

use rand::{thread_rng, Rng};

use crate::{
    abstract_tree::{Action, Type},
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

pub const BUILT_IN_FUNCTIONS: [BuiltInFunction; 5] = [
    BuiltInFunction::IntParse,
    BuiltInFunction::IntRandomRange,
    BuiltInFunction::ReadLine,
    BuiltInFunction::WriteLine,
    BuiltInFunction::Sleep,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    IntParse,
    IntRandomRange,
    ReadLine,
    WriteLine,
    Sleep,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::IntParse => "parse",
            BuiltInFunction::IntRandomRange => "random_range",
            BuiltInFunction::ReadLine => "read_line",
            BuiltInFunction::WriteLine => "write_line",
            BuiltInFunction::Sleep => "sleep",
        }
    }

    pub fn as_value(self) -> Value {
        Value::built_in_function(self)
    }

    pub fn call(&self, arguments: Vec<Value>, context: &Context) -> Result<Action, RuntimeError> {
        match self {
            BuiltInFunction::IntParse => {
                let string = arguments.get(0).unwrap();

                if let ValueInner::String(_string) = string.inner().as_ref() {
                    // let integer = string.parse();

                    todo!()

                    // Ok(Action::Return(Value::integer(integer)))
                } else {
                    let mut actual = Vec::with_capacity(arguments.len());

                    for value in arguments {
                        let r#type = value.r#type(context)?;

                        actual.push(r#type);
                    }

                    Err(RuntimeError::ValidationFailure(
                        ValidationError::WrongArguments {
                            expected: vec![Type::String],
                            actual,
                        },
                    ))
                }
            }
            BuiltInFunction::IntRandomRange => {
                let range = arguments.get(0).unwrap();

                if let ValueInner::Range(range) = range.inner().as_ref() {
                    let random = thread_rng().gen_range(range.clone());

                    Ok(Action::Return(Value::integer(random)))
                } else {
                    panic!("Built-in function cannot have a non-function type.")
                }
            }
            BuiltInFunction::ReadLine => {
                let mut input = String::new();

                stdin().read_line(&mut input)?;

                Ok(Action::Return(Value::string(input)))
            }
            BuiltInFunction::WriteLine => {
                println!("{}", arguments[0]);

                Ok(Action::None)
            }
            BuiltInFunction::Sleep => {
                if let ValueInner::Integer(milliseconds) = arguments[0].inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(Action::None)
            }
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BuiltInFunction::IntParse => write!(f, "(input : int) : str {{ *MAGIC* }}"),
            BuiltInFunction::IntRandomRange => write!(f, "(input: range) : int {{ *MAGIC* }}"),
            BuiltInFunction::ReadLine => write!(f, "() : str {{ *MAGIC* }}"),
            BuiltInFunction::WriteLine => write!(f, "(to_output : any) : none {{ *MAGIC* }}"),
            BuiltInFunction::Sleep => write!(f, "(milliseconds : int) : none {{ *MAGIC* }}"),
        }
    }
}
