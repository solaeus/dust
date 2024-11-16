//! Runtime values used by the VM.
mod abstract_value;
mod concrete_value;
mod range_value;

pub use abstract_value::AbstractValue;
pub use concrete_value::ConcreteValue;
pub use range_value::RangeValue;

use std::fmt::{self, Debug, Display, Formatter};

use crate::{Vm, VmError};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ValueOwned {
    Abstract(AbstractValue),
    Concrete(ConcreteValue),
}

impl ValueOwned {
    pub fn to_concrete_owned(&self, vm: &Vm) -> Result<ConcreteValue, VmError> {
        match self {
            ValueOwned::Abstract(abstract_value) => abstract_value.to_concrete_owned(vm),
            ValueOwned::Concrete(concrete_value) => Ok(concrete_value.clone()),
        }
    }

    pub fn display(&self, vm: &Vm) -> Result<String, VmError> {
        match self {
            ValueOwned::Abstract(abstract_value) => abstract_value.display(vm),
            ValueOwned::Concrete(concrete_value) => Ok(concrete_value.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ValueRef<'a> {
    Abstract(&'a AbstractValue),
    Concrete(&'a ConcreteValue),
}

impl ValueRef<'_> {
    pub fn to_concrete_owned(&self, vm: &Vm) -> Result<ConcreteValue, VmError> {
        match self {
            ValueRef::Abstract(abstract_value) => abstract_value.to_concrete_owned(vm),
            ValueRef::Concrete(concrete_value) => Ok((*concrete_value).clone()),
        }
    }

    pub fn display(&self, vm: &Vm) -> Result<String, VmError> {
        match self {
            ValueRef::Abstract(abstract_value) => abstract_value.display(vm),
            ValueRef::Concrete(concrete_value) => Ok(concrete_value.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(ConcreteValue, ConcreteValue),
    CannotAnd(ConcreteValue, ConcreteValue),
    CannotCompare(ConcreteValue, ConcreteValue),
    CannotDivide(ConcreteValue, ConcreteValue),
    CannotModulo(ConcreteValue, ConcreteValue),
    CannotMultiply(ConcreteValue, ConcreteValue),
    CannotNegate(ConcreteValue),
    CannotNot(ConcreteValue),
    CannotSubtract(ConcreteValue, ConcreteValue),
    CannotOr(ConcreteValue, ConcreteValue),
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
