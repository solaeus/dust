use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    dust_error::{AnnotatedError, DustError},
    source::{Position, Source},
    syntax::SyntaxKind,
    token::TokenKind,
};

#[derive(Debug)]
pub enum ParseError {
    CannotResolveModule {
        position: Position,
    },
    ExpectedSyntax {
        found: SyntaxKind,
        expected: SyntaxKind,
        position: Position,
    },
    ExpectedToken {
        found: TokenKind,
        expected: TokenKind,
        position: Position,
    },
    ExpectedMultipleTokens {
        found: TokenKind,
        expected: &'static [TokenKind],
        position: Position,
    },
    ExpectedItem {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        found: Option<SyntaxKind>,
        position: Position,
    },
    InvalidUtf8 {
        position: Position,
    },
    UnexpectedToken {
        found: TokenKind,
        position: Position,
    },
}

impl<'src> AnnotatedError<'src> for ParseError {
    type Context = &'src Source<'src>;

    fn add_report(&self, source: Self::Context, groups: &mut Vec<Group<'src>>) {
        match self {
            ParseError::CannotResolveModule { position } => {
                let title = "Cannot resolve module".to_string();
                let file = match source.get_file(position.file_id) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let module_name = match file.content_str(position.span) {
                    Ok(str) => str,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group =  Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file.content_as_str())
                        .path(file.file_name())
                        .fold(false)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!("Cannot find \"{module_name}.ds\" or \"{module_name}/mod.ds\" in this directory")),
                        ),
                );

                groups.push(group);
            }
            ParseError::InvalidUtf8 { position } => {
                let title = "Invalid UTF-8 sequence".to_string();
                let file = match source.get_file(position.file_id) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file.content_as_str())
                        .path(file.file_name())
                        .fold(false)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label("This is not valid UTF-8"),
                        ),
                );

                groups.push(group);
            }
            ParseError::ExpectedSyntax {
                found: actual,
                expected,
                position,
            } => {
                let title = format!("Expected {expected}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Expected {expected}, but found {actual}.")),
                    ),
                );

                groups.push(group);
            }
            ParseError::ExpectedToken {
                found: actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();
                let file = match source.get_file(position.file_id) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file.content_as_str())
                        .path(file.file_name())
                        .fold(false)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!("Expected {expected}, but found {actual}.")),
                        ),
                );

                groups.push(group);
            }
            ParseError::ExpectedMultipleTokens {
                found: actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();
                let file = match source.get_file(position.file_id) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let expected_list = expected
                    .iter()
                    .enumerate()
                    .map(|(index, token)| {
                        let is_last = index == expected.len() - 1;

                        if is_last && expected.len() > 1 {
                            format!("or {token}")
                        } else {
                            format!("{token}, ")
                        }
                    })
                    .collect::<String>();
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file.content_as_str())
                        .path(file.file_name())
                        .fold(false)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!(
                                    "Found {actual} but expected either {expected_list} here."
                                )),
                        ),
                );

                groups.push(group);
            }
            ParseError::UnexpectedToken { position, found } => {
                let title = "Unexpected token".to_string();
                let file = match source.get_file(position.file_id) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file.content_as_str())
                        .path(file.full_path())
                        .fold(false)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!("{found} was not expected here")),
                        ),
                );

                groups.push(group);
            }
            ParseError::ExpectedItem { position, found } => {
                let title = format!("Expected an item, but found {found}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                );

                groups.push(group);
            }
            ParseError::ExpectedStatement { position, found } => {
                let title = format!("Expected a statement, but found {found}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                );

                groups.push(group);
            }
            ParseError::ExpectedExpression { position, found } => {
                let title = match found {
                    Some(found) => format!("Expected an expression, but found {found}"),
                    None => "Expected an expression".to_string(),
                };
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                );

                groups.push(group);
            }
        }
    }
}
