use std::panic::{self, Location, PanicHookInfo};

use annotate_snippets::{Level, Renderer, Snippet};
use smallvec::SmallVec;

use crate::{NativeFunctionError, Value, ValueRef, Vm};

pub fn panic(
    vm: &Vm,
    arguments: SmallVec<[ValueRef; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut message = String::new();

    for value_ref in arguments {
        let string = match value_ref.display(vm) {
            Ok(string) => string,
            Err(error) => return Err(NativeFunctionError::Vm(Box::new(error))),
        };

        message.push_str(&string);
    }

    let position = vm.current_position();
    let error_output = Level::Error.title("Explicit Panic").snippet(
        Snippet::source(vm.source()).fold(false).annotation(
            Level::Error
                .span(position.0..position.1)
                .label("Explicit panic occured here"),
        ),
    );
    let renderer = Renderer::plain();
    let report = renderer.render(error_output).to_string();

    panic::set_hook(Box::new(move |_| {
        println!("{}", report);
        println!("Panic Message: {}", message);
    }));

    panic!();
}
