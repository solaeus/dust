use std::{
    fmt::{self, Display, Formatter},
    io::stdin,
    thread,
    time::Duration,
};

use rand::{thread_rng, Rng};

use crate::{
    abstract_tree::{Action, Identifier, Type, WithPosition},
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BuiltInFunction {
    name: WithPosition<Identifier>,
}

impl BuiltInFunction {
    pub fn new(name: WithPosition<Identifier>) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &Identifier {
        &self.name.node
    }

    pub fn as_value(self) -> Value {
        Value::built_in_function(self)
    }

    pub fn r#type(&self) -> Type {
        match self.name.node.as_str() {
            "WRITE_LINE" => Type::Function {
                parameter_types: vec![Type::String],
                return_type: Box::new(Type::None),
            },
            _ => {
                todo!()
            }
        }
    }

    pub fn call(&self, arguments: Vec<Value>, context: &Context) -> Result<Action, RuntimeError> {
        match self.name.node.as_str() {
            "INT_PARSE" => {
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
            "INT_RANDOM_RANGE" => {
                let range = arguments.get(0).unwrap();

                if let ValueInner::Range(range) = range.inner().as_ref() {
                    let random = thread_rng().gen_range(range.clone());

                    Ok(Action::Return(Value::integer(random)))
                } else {
                    panic!("Built-in function cannot have a non-function type.")
                }
            }
            "READ_LINE" => {
                let mut input = String::new();

                stdin().read_line(&mut input)?;

                Ok(Action::Return(Value::string(input)))
            }
            "WRITE_LINE" => {
                println!("{}", arguments[0]);

                Ok(Action::None)
            }
            "SLEEP" => {
                if let ValueInner::Integer(milliseconds) = arguments[0].inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(Action::None)
            }
            _ => {
                todo!()
            }
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name.node)
    }
}
