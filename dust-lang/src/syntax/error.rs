use std::fmt::{self, Display, Formatter};

use annotate_snippets::Renderer;

use crate::{
    error::AnnotatedError,
    source::SourceFileId,
    syntax::{SyntaxId, node::SyntaxPayload},
};

#[derive(Debug)]
pub enum SyntaxError {
    MissingSyntaxTree(SourceFileId),
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChild { total_children: usize },
    InvalidSyntaxPayload(SyntaxPayload),
    ExpectedSyntaxChildren { expected: usize, actual: usize },
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut groups = Vec::with_capacity(1);

        self.add_report((), &mut groups);

        let renderer = Renderer::styled();
        let display = renderer.render(&groups);

        write!(f, "{display}")
    }
}

impl<'a> AnnotatedError<'a> for SyntaxError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        self.add_internal_report(groups);
    }
}
