use crate::source::SourceFile;

pub const CORE: SourceFile = SourceFile::validated("core.ds", include_str!("../../std/core.ds"));
