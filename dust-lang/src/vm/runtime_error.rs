use crate::{AnnotatedError, Operation, Span};

#[derive(Debug, PartialEq)]
pub struct RuntimeError(pub Operation);

impl AnnotatedError for RuntimeError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        "An error occurred during the execution of the Dust VM. This is a bug in the VM or the compiler."
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        vec![]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        vec![]
    }
}
