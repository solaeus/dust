//! Runtime values used by the VM.
mod abstract_list;
mod concrete_value;
mod function;
mod range_value;

pub use abstract_list::AbstractList;
pub use concrete_value::{ConcreteValue, DustString};
pub use function::Function;
pub use range_value::RangeValue;
use serde::{Deserialize, Serialize};

use std::fmt::{self, Debug, Display, Formatter};

use crate::{vm::Record, Type};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    Concrete(ConcreteValue),

    #[serde(skip)]
    AbstractList(AbstractList),

    #[serde(skip)]
    Function(Function),
}

impl Value {
    pub fn as_boolean(&self) -> Option<&bool> {
        if let Value::Concrete(ConcreteValue::Boolean(value)) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Value::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let Value::Concrete(ConcreteValue::String(value)) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Concrete(concrete_value) => concrete_value.r#type(),
            Value::AbstractList(AbstractList { item_type, .. }) => {
                Type::List(Box::new(item_type.clone()))
            }
            Value::Function(Function { r#type, .. }) => Type::Function(Box::new(r#type.clone())),
        }
    }

    pub fn add(&self, other: &Value) -> Value {
        let concrete = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => left.add(right),
            _ => panic!("{}", ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Value::Concrete(concrete)
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.subtract(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotSubtract(
                self.to_owned(),
                other.to_owned(),
            )),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.multiply(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotMultiply(
                self.to_owned(),
                other.to_owned(),
            )),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.divide(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotDivide(self.to_owned(), other.to_owned())),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.modulo(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotModulo(self.to_owned(), other.to_owned())),
        }
    }

    pub fn negate(&self) -> Value {
        let concrete = match self {
            Value::Concrete(concrete_value) => concrete_value.negate(),
            _ => panic!("{}", ValueError::CannotNegate(self.clone())),
        };

        Value::Concrete(concrete)
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        match self {
            Value::Concrete(concrete_value) => concrete_value.not().map(Value::Concrete),
            _ => Err(ValueError::CannotNot(self.to_owned())),
        }
    }

    pub fn equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.equal(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }

    pub fn less(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.less_than(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => {
                left.less_than_or_equal(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }

    pub fn display(&self, record: &Record) -> DustString {
        match self {
            Value::AbstractList(list) => list.display(record),
            Value::Concrete(concrete_value) => concrete_value.display(),
            Value::Function(function) => DustString::from(function.to_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Concrete(concrete_value) => write!(f, "{concrete_value}"),
            Value::AbstractList(list) => write!(f, "{list}"),
            Value::Function(function) => write!(f, "{function}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(Value, Value),
    CannotAnd(Value, Value),
    CannotCompare(Value, Value),
    CannotDivide(Value, Value),
    CannotModulo(Value, Value),
    CannotMultiply(Value, Value),
    CannotNegate(Value),
    CannotNot(Value),
    CannotSubtract(Value, Value),
    CannotOr(Value, Value),
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueError::CannotAdd(left, right) => {
                write!(f, "Cannot add {left} and {right}")
            }
            ValueError::CannotAnd(left, right) => {
                write!(f, "Cannot use logical AND operation on {left} and {right}")
            }
            ValueError::CannotCompare(left, right) => {
                write!(f, "Cannot compare {left} and {right}")
            }
            ValueError::CannotDivide(left, right) => {
                write!(f, "Cannot divide {left} by {right}")
            }
            ValueError::CannotModulo(left, right) => {
                write!(f, "Cannot use modulo operation on {left} and {right}")
            }
            ValueError::CannotMultiply(left, right) => {
                write!(f, "Cannot multiply {left} by {right}")
            }
            ValueError::CannotNegate(value) => {
                write!(f, "Cannot negate {value}")
            }
            ValueError::CannotNot(value) => {
                write!(f, "Cannot use logical NOT operation on {value}")
            }
            ValueError::CannotSubtract(left, right) => {
                write!(f, "Cannot subtract {right} from {left}")
            }
            ValueError::CannotOr(left, right) => {
                write!(f, "Cannot use logical OR operation on {left} and {right}")
            }
        }
    }
}
