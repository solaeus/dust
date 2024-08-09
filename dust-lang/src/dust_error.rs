//! Top-level error handling for the Dust language.
use annotate_snippets::{Level, Renderer, Snippet};
use std::{error::Error, fmt::Display};

use crate::{AnalyzerError, VmError};

/// An error that occurred during the execution of the Dust language and its
/// corresponding source code.
#[derive(Debug, Clone, PartialEq)]
pub struct DustError<'src> {
    vm_error: VmError,
    source: &'src str,
}

impl<'src> DustError<'src> {
    pub fn new(vm_error: VmError, source: &'src str) -> Self {
        Self { vm_error, source }
    }

    pub fn report(&self) -> String {
        let title = match &self.vm_error {
            VmError::AnaylyzerError(_) => "Analyzer error",
            VmError::ParseError(_) => "Parse error",
            VmError::ValueError(_) => "Value error",
            VmError::BuiltInFunctionCallError(_) => "Runtime error",
            _ => "Analysis Failure",
        };
        let span = match &self.vm_error {
            VmError::AnaylyzerError(analyzer_error) => match analyzer_error {
                AnalyzerError::ExpectedBoolean { position, .. } => position,
                AnalyzerError::ExpectedFunction { position, .. } => position,
                AnalyzerError::ExpectedIdentifier { position, .. } => position,
                AnalyzerError::ExpectedIdentifierOrValue { position, .. } => position,
                AnalyzerError::ExpectedIntegerOrFloat { position, .. } => position,
                AnalyzerError::ExpectedIntegerFloatOrString { position, .. } => position,
                AnalyzerError::UnexpectedIdentifier { position, .. } => position,
            },
            VmError::ParseError(_) => todo!(),
            VmError::ValueError(_) => todo!(),
            VmError::BuiltInFunctionCallError(_) => todo!(),
            VmError::ExpectedIdentifier { position } => position,
            VmError::ExpectedIdentifierOrInteger { position } => position,
            VmError::ExpectedInteger { position } => position,
            VmError::ExpectedFunction { position, .. } => position,
            VmError::ExpectedList { position } => position,
            VmError::ExpectedValue { position } => position,
        };
        let label = self.vm_error.to_string();
        let message = Level::Error.title(title).snippet(
            Snippet::source(self.source).annotation(Level::Info.span(span.0..span.1).label(&label)),
        );
        let renderer = Renderer::styled();

        format!("{}", renderer.render(message))
    }
}

impl Error for DustError<'_> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.vm_error)
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.vm_error, self.source)
    }
}
