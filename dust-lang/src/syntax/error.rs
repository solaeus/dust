use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    dust_error::AnnotatedError,
    source::{Position, Source},
    syntax::{SyntaxId, SyntaxKind, SyntaxPayload},
};

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedItem {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        found: SyntaxKind,
        position: Position,
    },
    Internal(InternalSyntaxError),
}

impl<'a> AnnotatedError<'a> for SyntaxError {
    type Input = &'a Source;

    fn annotated_error(&self, source: Self::Input) -> Group<'a> {
        match self {
            SyntaxError::ExpectedItem { found, position } => {
                let title = "Syntax Error: expected item".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("This {found} was expected to be an item.")),
                    ),
                )
            }
            SyntaxError::ExpectedStatement { found, position } => {
                let title = "Syntax Error: expected statement".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("This {found} was expected to be a statement.")),
                    ),
                )
            }
            SyntaxError::ExpectedExpression { found, position } => {
                let title = "Syntax Error: expected expression".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("The {found} was expected to be an expression.")),
                    ),
                )
            }
            SyntaxError::Internal(internal_syntax_error) => {
                let title = "Internal Syntax Error".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(format!("{internal_syntax_error:#?}")))
            }
        }
    }
}

#[derive(Debug)]
pub enum InternalSyntaxError {
    ExpectedChild,
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChildren(SyntaxPayload),
    EmptySyntaxTree,
}
