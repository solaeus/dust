use std::fmt::{self, Display, Formatter};

use annotate_snippets::Renderer;

use crate::{error::DustError, source::SourceCodeId, syntax::SyntaxId};

#[derive(Clone, Debug)]
pub enum SyntaxError {
    MissingTree(SourceCodeId),
    MissingNode(SyntaxId),
    MissingChild {
        missing_index: u32,
        total_children: u32,
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

impl<'a> DustError<'a> for SyntaxError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        self.add_internal_report(groups);
    }
}
