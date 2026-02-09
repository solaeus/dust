use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
    path::PathBuf,
};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

const FILE_NOT_FOUND: &str = "<dust internal error: file not found>";
const SOURCE_NOT_FOUND: &str = "<dust internal error: source not found>";
const PATH_INVALID_UTF8: &str = "<dust internal error: path contains invalid UTF-8>";

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

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn add_file(&mut self, file: SourceFile) -> SourceFileId {
        let id = SourceFileId(self.files.len() as u32);

        self.files.push(file);

        id
    }

    pub fn get_file(&self, file_id: SourceFileId) -> &SourceFile {
        self.files
            .get(file_id.0 as usize)
            .unwrap_or(&SourceFile::NOT_FOUND)
    }

    pub fn set_utf8_validated(&mut self, file_id: SourceFileId) {
        if let Some(SourceFile::File { utf8_validated, .. }) =
            self.files.get_mut(file_id.0 as usize)
        {
            *utf8_validated = true;
        }
    }

    pub fn iter(&self) -> SourceIterator<'_> {
        SourceIterator::new(self)
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SourceFileId(u32);

impl SourceFileId {
    pub const MAIN: Self = SourceFileId(0);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Debug)]
pub enum SourceFile {
    BuiltIn {
        path: &'static str,
        source_str: &'static str,
    },
    Embedded {
        path: String,
        source_bytes: Vec<u8>,
        utf8_validated: bool,
    },
    File {
        path: PathBuf,
        mmap: Mmap,
        utf8_validated: bool,
    },
}

impl SourceFile {
    pub const NOT_FOUND: Self = SourceFile::BuiltIn {
        path: FILE_NOT_FOUND,
        source_str: FILE_NOT_FOUND,
    };

    pub fn built_in(path: &'static str, source_code: &'static str) -> Self {
        SourceFile::BuiltIn {
            path,
            source_str: source_code,
        }
    }

    pub fn embedded_string(path: String, source_code: String) -> Self {
        SourceFile::Embedded {
            path,
            source_bytes: source_code.into_bytes(),
            utf8_validated: true,
        }
    }

    pub fn embedded_bytes(path: String, source_code: Vec<u8>) -> Self {
        SourceFile::Embedded {
            path,
            source_bytes: source_code,
            utf8_validated: false,
        }
    }

    pub fn file(path: PathBuf, source_code: Mmap) -> Self {
        SourceFile::File {
            path,
            mmap: source_code,
            utf8_validated: false,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::BuiltIn { path, .. } => path,
            Self::Embedded { path, .. } => path.as_str(),
            Self::File { path, .. } => path.to_str().unwrap_or(PATH_INVALID_UTF8),
        }
    }

    pub fn is_utf8_validated(&self) -> bool {
        match self {
            Self::BuiltIn { .. } => true,
            Self::Embedded { utf8_validated, .. } => *utf8_validated,
            Self::File { utf8_validated, .. } => *utf8_validated,
        }
    }

    pub fn source_bytes(&self, span: Span) -> &[u8] {
        let full_source = self.full_source_bytes();
        let range = span.as_usize_range();

        full_source
            .get(range)
            .unwrap_or_else(|| SOURCE_NOT_FOUND.as_bytes())
    }

    pub fn source_str(&self, span: Span) -> &str {
        let full_source = self.full_source_str();
        let range = span.as_usize_range();

        full_source.get(range).unwrap_or(SOURCE_NOT_FOUND)
    }

    pub fn full_source_bytes(&self) -> &[u8] {
        match self {
            Self::BuiltIn { source_str, .. } => source_str.as_bytes(),
            Self::Embedded { source_bytes, .. } => source_bytes,
            Self::File { mmap, .. } => mmap,
        }
    }

    pub fn full_source_str(&self) -> &str {
        let handle_utf8_validation = |path: &str, source_bytes| -> &str {
            warn!(
                "Source file at {} is being accessed before UTF-8 validation. Doing\
                    immediate validation now. All files should be validated by the lexer before\
                    being accessed to avoid this warning.",
                path
            );

            let utf8_bytes = match str::from_utf8(source_bytes) {
                Ok(str) => return str,
                Err(error) => {
                    error!("Source file at {} contains invalid UTF-8.", path);

                    &source_bytes[0..error.valid_up_to()]
                }
            };

            unsafe { str::from_utf8_unchecked(utf8_bytes) }
        };

        match self {
            Self::BuiltIn { source_str, .. } => source_str,
            Self::Embedded {
                path,
                source_bytes,
                utf8_validated: false,
            } => handle_utf8_validation(path, source_bytes),
            Self::Embedded {
                source_bytes,
                utf8_validated: true,
                ..
            } => unsafe { str::from_utf8_unchecked(source_bytes) },
            Self::File {
                path,
                mmap,
                utf8_validated: false,
            } => {
                let path_str = path.to_str().unwrap_or(PATH_INVALID_UTF8);

                handle_utf8_validation(path_str, mmap)
            }
            Self::File {
                mmap,
                utf8_validated: true,
                ..
            } => unsafe { str::from_utf8_unchecked(mmap) },
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
        let start = start.try_into().unwrap_or_default();
        let end = end.try_into().unwrap_or_default().max(start);

        Self(start, end)
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

pub struct SourceIterator<'a> {
    source: &'a Source,
    position: usize,
}

impl<'a> SourceIterator<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            position: 0,
        }
    }
}

impl<'a> Iterator for SourceIterator<'a> {
    type Item = (SourceFileId, &'a SourceFile);

    fn next(&mut self) -> Option<Self::Item> {
        let file_id = SourceFileId(self.position as u32);
        let file = self.source.files.get(self.position)?;

        self.position += 1;

        Some((file_id, file))
    }
}

impl ExactSizeIterator for SourceIterator<'_> {}
