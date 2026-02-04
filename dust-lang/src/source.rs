use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Source {
    files: Vec<SourceFile>,
}

impl Source {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            files: Vec::with_capacity(capacity),
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn files(&self) -> &Vec<SourceFile> {
        &self.files
    }

    pub fn add_file(&mut self, file: SourceFile) -> SourceFileId {
        let id = SourceFileId(self.files.len() as u32);

        self.files.push(file);

        id
    }

    pub fn get_file(&self, file_id: SourceFileId) -> Option<&SourceFile> {
        self.files.get(file_id.0 as usize)
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

impl SourceCode {
    pub fn get(&self, start: usize, end: usize) -> &str {
        let bytes = self.get_bytes(start, end);

        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_span(&self, span: Span) -> &str {
        self.get(span.0 as usize, span.1 as usize)
    }

    pub fn get_bytes(&self, start: usize, end: usize) -> &[u8] {
        self.as_ref().get(start..end).unwrap_or_default()
    }

    pub fn get_span_bytes(&self, span: Span) -> &[u8] {
        self.get_bytes(span.0 as usize, span.1 as usize)
    }
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

    pub fn shrink(&self, offset: u32) -> Span {
        Span(self.0 + offset, self.1 - offset)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.0, self.1)
    }
}
