use std::fmt::{self, Display, Formatter};

use crate::{vm::Pointer, ConcreteValue, Vm, VmError};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum AbstractValue {
    FunctionSelf,
    List { items: Vec<Pointer> },
}

impl AbstractValue {
    pub fn to_concrete_owned(&self, vm: &Vm) -> Result<ConcreteValue, VmError> {
        match self {
            AbstractValue::FunctionSelf => Ok(ConcreteValue::Function(vm.chunk().clone())),
            AbstractValue::List { items, .. } => {
                let mut resolved_items = Vec::with_capacity(items.len());

                for pointer in items {
                    let resolved_item = vm.follow_pointer(*pointer)?.to_concrete_owned(vm)?;

                    resolved_items.push(resolved_item);
                }

                Ok(ConcreteValue::List(resolved_items))
            }
        }
    }

    pub fn display(&self, vm: &Vm) -> Result<String, VmError> {
        match self {
            AbstractValue::FunctionSelf => Ok("self".to_string()),
            AbstractValue::List { items, .. } => {
                let mut display = "[".to_string();

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        display.push_str(", ");
                    }

                    let item_display = vm.follow_pointer(*item)?.display(vm)?;

                    display.push_str(&item_display);
                }

                display.push(']');

                Ok(display)
            }
        }
    }
}

impl Clone for AbstractValue {
    fn clone(&self) -> Self {
        log::trace!("Cloning abstract value {:?}", self);

        match self {
            AbstractValue::FunctionSelf => AbstractValue::FunctionSelf,
            AbstractValue::List { items } => AbstractValue::List {
                items: items.clone(),
            },
        }
    }
}

impl Display for AbstractValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AbstractValue::FunctionSelf => write!(f, "self"),
            AbstractValue::List { items, .. } => {
                write!(f, "[")?;

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", item)?;
                }

                write!(f, "]")
            }
        }
    }
}
