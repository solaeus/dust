use std::fmt::{self, Display, Formatter};

use crate::{vm::Pointer, ConcreteValue, DustString, Value, ValueRef, Vm, VmError};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum AbstractValue {
    FunctionSelf,
    List { item_pointers: Vec<Pointer> },
}

impl AbstractValue {
    pub fn to_value(self) -> Value {
        Value::Abstract(self)
    }

    pub fn to_value_ref(&self) -> ValueRef {
        ValueRef::Abstract(self)
    }

    pub fn to_concrete_owned(&self, vm: &Vm) -> ConcreteValue {
        match self {
            AbstractValue::FunctionSelf => ConcreteValue::function(vm.chunk().clone()),
            AbstractValue::List { item_pointers, .. } => {
                let mut items: Vec<ConcreteValue> = Vec::with_capacity(item_pointers.len());

                for pointer in item_pointers {
                    let item_option = vm.follow_pointer_allow_empty(*pointer);
                    let item = match item_option {
                        Some(value_ref) => value_ref.into_concrete_owned(vm),
                        None => continue,
                    };

                    items.push(item);
                }

                ConcreteValue::List(items)
            }
        }
    }

    pub fn to_dust_string(&self, vm: &Vm) -> Result<DustString, VmError> {
        let mut display = DustString::new();

        match self {
            AbstractValue::FunctionSelf => display.push_str("self"),
            AbstractValue::List {
                item_pointers: items,
                ..
            } => {
                display.push('[');

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        display.push_str(", ");
                    }

                    let item_display = vm.follow_pointer(*item).display(vm)?;

                    display.push_str(&item_display);
                }

                display.push(']');
            }
        }

        Ok(display)
    }
}

impl Clone for AbstractValue {
    fn clone(&self) -> Self {
        log::trace!("Cloning abstract value {:?}", self);

        match self {
            AbstractValue::FunctionSelf => AbstractValue::FunctionSelf,
            AbstractValue::List {
                item_pointers: items,
            } => AbstractValue::List {
                item_pointers: items.clone(),
            },
        }
    }
}

impl Display for AbstractValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AbstractValue::FunctionSelf => write!(f, "self"),
            AbstractValue::List {
                item_pointers: items,
                ..
            } => {
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
