use std::sync::Arc;

use crate::{Chunk, DustString, List};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Object {
    Empty,
    Function(Arc<Chunk>),
    ValueList(List),
    String(DustString),
}
