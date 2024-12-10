//! Runtime values used by the VM.
mod abstract_value;
mod concrete_value;
mod range_value;

pub use abstract_value::AbstractValue;
pub use concrete_value::{ConcreteValue, DustString};
pub use range_value::RangeValue;

use std::fmt::{self, Debug, Display, Formatter};

use crate::{Vm, VmError};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Abstract(AbstractValue),
    Concrete(ConcreteValue),
}

impl Value {
    pub fn to_ref(&self) -> ValueRef {
        match self {
            Value::Abstract(abstract_value) => ValueRef::Abstract(abstract_value),
            Value::Concrete(concrete_value) => ValueRef::Concrete(concrete_value),
        }
    }

    pub fn into_concrete_owned(self, vm: &Vm) -> Result<ConcreteValue, VmError> {
        match self {
            Value::Abstract(abstract_value) => abstract_value.to_concrete_owned(vm),
            Value::Concrete(concrete_value) => Ok(concrete_value),
        }
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        if let Value::Concrete(ConcreteValue::Boolean(value)) = self {
            Some(value)
        } else {
            None
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum ValueRef<'a> {
    Abstract(&'a AbstractValue),
    Concrete(&'a ConcreteValue),
}

impl ValueRef<'_> {
    pub fn to_owned(&self) -> Value {
        match self {
            ValueRef::Abstract(abstract_value) => Value::Abstract((*abstract_value).clone()),
            ValueRef::Concrete(concrete_value) => Value::Concrete((*concrete_value).clone()),
        }
    }

    pub fn into_concrete_owned(self, vm: &Vm) -> Result<ConcreteValue, VmError> {
        match self {
            ValueRef::Abstract(abstract_value) => abstract_value.to_concrete_owned(vm),
            ValueRef::Concrete(concrete_value) => Ok(concrete_value.clone()),
        }
    }

    pub fn display(&self, vm: &Vm) -> Result<DustString, VmError> {
        match self {
            ValueRef::Abstract(abstract_value) => abstract_value.to_dust_string(vm),
            ValueRef::Concrete(concrete_value) => Ok(concrete_value.to_dust_string()),
        }
    }

    #[inline(always)]
    pub fn add(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.add(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotAdd(self.to_owned(), other.to_owned())),
        }
    }

    pub fn subtract(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.subtract(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotSubtract(
                self.to_owned(),
                other.to_owned(),
            )),
        }
    }

    pub fn multiply(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.multiply(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotMultiply(
                self.to_owned(),
                other.to_owned(),
            )),
        }
    }

    pub fn divide(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.divide(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotDivide(self.to_owned(), other.to_owned())),
        }
    }

    pub fn modulo(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.modulo(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotModulo(self.to_owned(), other.to_owned())),
        }
    }

    pub fn negate(&self) -> Result<Value, ValueError> {
        match self {
            ValueRef::Concrete(concrete_value) => concrete_value.negate().map(Value::Concrete),
            _ => Err(ValueError::CannotNegate(self.to_owned())),
        }
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        match self {
            ValueRef::Concrete(concrete_value) => concrete_value.not().map(Value::Concrete),
            _ => Err(ValueError::CannotNot(self.to_owned())),
        }
    }

    pub fn equal(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.equal(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }

    #[inline(always)]
    pub fn less_than(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.less_than(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }

    pub fn less_than_or_equal(&self, other: ValueRef) -> Result<Value, ValueError> {
        match (self, other) {
            (ValueRef::Concrete(left), ValueRef::Concrete(right)) => {
                left.less_than_or_equal(right).map(Value::Concrete)
            }
            _ => Err(ValueError::CannotCompare(self.to_owned(), other.to_owned())),
        }
    }
}

impl Display for ValueRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueRef::Abstract(abstract_value) => write!(f, "{}", abstract_value),
            ValueRef::Concrete(concrete_value) => write!(f, "{}", concrete_value),
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
