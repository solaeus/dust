use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Source {
    files: Arc<RwLock<Vec<SourceFile>>>,
}

impl Source {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn read_files(&self) -> RwLockReadGuard<'_, Vec<SourceFile>> {
        self.files
            .read()
            .expect("Failed to acquire read lock on source files")
    }

    pub fn write_files(&self) -> RwLockWriteGuard<'_, Vec<SourceFile>> {
        self.files
            .write()
            .expect("Failed to acquire write lock on source files")
    }

    pub fn add_file(&self, file: SourceFile) -> SourceFileId {
        let mut files = self.write_files();
        let id = SourceFileId(files.len() as u32);

        files.push(file);

        id
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SourceFileId(pub u32);

impl SourceFileId {
    pub const MAIN: Self = SourceFileId(0);
}

#[derive(Debug)]
pub struct SourceFile {
    pub name: String,
    pub source_code: SourceCode,
}

#[derive(Debug)]
pub enum SourceCode {
    Bytes(Vec<u8>),
    String(String),
    Mmap(Mmap),
}

impl AsRef<[u8]> for SourceCode {
    fn as_ref(&self) -> &[u8] {
        match self {
            SourceCode::Bytes(bytes) => bytes.as_ref(),
            SourceCode::String(string) => string.as_bytes(),
            SourceCode::Mmap(mmap) => mmap.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub file_id: SourceFileId,
    pub span: Span,
}

impl Position {
    pub fn new(file_id: SourceFileId, span: Span) -> Self {
        Self { file_id, span }
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Span(pub u32, pub u32);

impl Span {
    pub fn new<T: TryInto<u32>>(start: T, end: T) -> Self {
        Self(
            start.try_into().unwrap_or_default(),
            end.try_into().unwrap_or_default(),
        )
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        Range {
            start: self.0 as usize,
            end: self.1 as usize,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.0, self.1)
    }
}
