use std::{
    fmt::{self, Display, Formatter},
    io::stdin,
    thread,
    time::Duration,
};

use crate::{
    abstract_tree::{Action, Type, WithPos},
    context::Context,
    error::RuntimeError,
    value::ValueInner,
    Value,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    ReadLine,
    Sleep,
    WriteLine,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::ReadLine => todo!(),
            BuiltInFunction::Sleep => todo!(),
            BuiltInFunction::WriteLine => todo!(),
        }
    }

    pub fn as_value(self) -> Value {
        Value::built_in_function(self)
    }

    pub fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::WriteLine => Type::Function {
                parameter_types: vec![Type::String.with_position((0, 0))],
                return_type: Box::new(Type::None.with_position((0, 0))),
            },
            _ => {
                todo!()
            }
        }
    }

    pub fn call(&self, arguments: Vec<Value>, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            // "INT_RANDOM_RANGE" => {
            //     let range = arguments.get(0).unwrap();

            //     if let ValueInner::Range(range) = range.inner().as_ref() {
            //         let random = thread_rng().gen_range(range.clone());

            //         Ok(Action::Return(Value::integer(random)))
            //     } else {
            //         panic!("Built-in function cannot have a non-function type.")
            //     }
            // }
            BuiltInFunction::ReadLine => {
                let mut input = String::new();

                stdin().read_line(&mut input)?;

                Ok(Action::Return(Value::string(input)))
            }
            BuiltInFunction::Sleep => {
                if let ValueInner::Integer(milliseconds) = arguments[0].inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(Action::None)
            }
            BuiltInFunction::WriteLine => {
                println!("{}", arguments[0]);

                Ok(Action::None)
            }
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
