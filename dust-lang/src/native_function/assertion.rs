use std::panic;

use annotate_snippets::{Level, Renderer, Snippet};
use smallvec::SmallVec;

use crate::{DustString, NativeFunctionError, Value, ValueRef, Vm};

pub fn panic(
    vm: &Vm,
    arguments: SmallVec<[ValueRef; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut message: Option<DustString> = None;

    for value_ref in arguments {
        let string = match value_ref.display(vm) {
            Ok(string) => string,
            Err(error) => return Err(NativeFunctionError::Vm(Box::new(error))),
        };

        match message {
            Some(ref mut message) => message.push_str(&string),
            None => message = Some(string),
        }
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

        if let Some(message) = &message {
            println!("{}", message);
        }
    }));

    panic!();
}
