use crate::{AnnotatedError, Operation, Span};

pub const RUNTIME_ERROR_TEXT: &str = "An error occurred during the execution of the Dust VM. This is a bug in the VM or the compiler.";

#[derive(Debug, PartialEq)]
pub struct RuntimeError(pub Operation);

impl AnnotatedError for RuntimeError {
    fn title(&self) -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        vec![(
            format!(
                "An error occurred while executing the operation: {}",
                self.0
            ),
            Span(0, RUNTIME_ERROR_TEXT.len() - 1),
        )]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        vec![]
    }
}
