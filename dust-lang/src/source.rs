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
    const FILE_UNAVAILABLE: &str = "<internal error: file not found>";
    const SOURCE_UNAVAILABLE: &str = "<internal error: source not found>";

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

    pub fn get_file_as_str(&self, file_id: SourceFileId) -> &str {
        self.files
            .get(file_id.0 as usize)
            .map_or(Self::FILE_UNAVAILABLE, |file| {
                let bytes = file.source_code.as_ref();

                unsafe { str::from_utf8_unchecked(bytes) }
            })
    }

    pub fn get_source_bytes(&self, position: &Position) -> &[u8] {
        let Some(file) = self.files.get(position.file_id.0 as usize) else {
            return Self::SOURCE_UNAVAILABLE.as_bytes();
        };
        let file_bytes = file.source_code.as_ref();
        let span_range = position.span.as_usize_range();

        if span_range.end <= file_bytes.len() {
            &file_bytes[span_range]
        } else {
            Self::SOURCE_UNAVAILABLE.as_bytes()
        }
    }

    pub fn get_source_str(&self, position: &Position) -> &str {
        let bytes = self.get_source_bytes(position);

        unsafe { str::from_utf8_unchecked(bytes) }
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

    pub fn shrink(&self, offset: u32) -> Position {
        Position {
            file_id: self.file_id,
            span: self.span.shrink(offset),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Span(pub(crate) u32, pub(crate) u32);

impl Span {
    pub fn new<T: TryInto<u32>>(start: T, end: T) -> Self {
        Self(
            start.try_into().unwrap_or_default(),
            end.try_into().unwrap_or_default(),
        )
    }

    pub fn join(&self, other: &Span) -> Span {
        let new_start = self.0.min(other.0);
        let new_end = self.1.max(other.1);

        Span(new_start, new_end)
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        Range {
            start: self.0 as usize,
            end: self.1 as usize,
        }
    }

    pub fn shrink(&self, offset: u32) -> Span {
        let new_start = self.0.saturating_add(offset);
        let new_end = self.1.saturating_sub(offset);

        if new_start > new_end {
            Span(new_start, new_start)
        } else {
            Span(new_start, new_end)
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.0, self.1)
    }
}
