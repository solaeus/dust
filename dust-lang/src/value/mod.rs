//! Runtime values used by the VM.
mod abstract_value;
mod concrete_value;
mod range_value;

pub use abstract_value::AbstractValue;
pub use concrete_value::{ConcreteValue, DustString};
pub use range_value::RangeValue;
use serde::{Deserialize, Serialize};

use std::fmt::{self, Debug, Display, Formatter};

use crate::{Type, Vm, VmError};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    #[serde(skip)]
    Abstract(AbstractValue),
    Concrete(ConcreteValue),
}

impl Value {
    pub fn into_concrete_owned(self, vm: &Vm) -> ConcreteValue {
        match self {
            Value::Abstract(abstract_value) => abstract_value.to_concrete_owned(vm),
            Value::Concrete(concrete_value) => concrete_value,
        }
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        if let Value::Concrete(ConcreteValue::Boolean(value)) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Abstract(abstract_value) => abstract_value.r#type(),
            Value::Concrete(concrete_value) => concrete_value.r#type(),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => left.add(right).map(Value::Concrete),
            _ => Err(ValueError::CannotAdd(self.to_owned(), other.to_owned())),
        }
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

    pub fn negate(&self) -> Result<Value, ValueError> {
        match self {
            Value::Concrete(concrete_value) => concrete_value.negate().map(Value::Concrete),
            _ => Err(ValueError::CannotNegate(self.to_owned())),
        }
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

    pub fn display(&self, vm: &Vm) -> Result<DustString, VmError> {
        match self {
            Value::Abstract(abstract_value) => abstract_value.to_dust_string(vm),
            Value::Concrete(concrete_value) => Ok(concrete_value.to_dust_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Abstract(abstract_value) => write!(f, "{}", abstract_value),
            Value::Concrete(concrete_value) => write!(f, "{}", concrete_value),
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
