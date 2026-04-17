use std::fmt::{self, Display, Formatter};

use annotate_snippets::Renderer;

use crate::{
    error::AnnotatedError,
    source::FileId,
    syntax::{SyntaxId, node::SyntaxKind},
};

#[derive(Debug)]
pub enum SyntaxError {
    MissingSyntaxTree(FileId),
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChild {
        missing_index: u32,
        total_children: u32,
    },
    Unexpected {
        expected: &'static [SyntaxKind],
        found: SyntaxKind,
    },
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
