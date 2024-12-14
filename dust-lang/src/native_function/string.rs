use smallvec::SmallVec;

use crate::{ConcreteValue, NativeFunctionError, Value, Vm};

pub fn to_string(
    vm: &Vm,
    arguments: SmallVec<[&Value; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    if arguments.len() != 1 {
        return Err(NativeFunctionError::ExpectedArgumentCount {
            expected: 1,
            found: 0,
            position: vm.current_position(),
        });
    }

    let argument_string = match arguments[0].display(vm) {
        Ok(string) => string,
        Err(error) => return Err(NativeFunctionError::Vm(Box::new(error))),
    };

    Ok(Some(Value::Concrete(ConcreteValue::string(
        argument_string,
    ))))
}
