use core::panic;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io,
    ops::Range,
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

const SOURCE_NOT_FOUND: &str = "<dust internal error: source not found>";

#[derive(Debug)]
pub struct Source<'src> {
    files: Vec<SourceFile<'src>>,
}

impl<'src> Source<'src> {
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

    pub fn files(&self) -> &[SourceFile<'src>] {
        &self.files
    }

    pub fn add_file(&mut self, file: SourceFile<'src>) -> SourceFileId {
        let id = SourceFileId(self.files.len() as u32);

        self.files.push(file);

        id
    }

    pub fn get_file(&self, file_id: SourceFileId) -> &SourceFile<'src> {
        if let Some(file) = self.files.get(file_id.0 as usize) {
            file
        } else {
            panic!(
                "Failed to find source file for {file_id:?}. This indicates a misuse of the\
                `Source` type, which must be an append-only singleton"
            );
        }
    }

    pub fn get_file_content(&self, position: Position) -> &str {
        self.get_file(position.file_id).content_str(position.span)
    }

    pub fn set_utf8_validated(&mut self, file_id: SourceFileId) {
        if let Some(SourceFile::File { utf8_validated, .. }) =
            self.files.get_mut(file_id.0 as usize)
        {
            *utf8_validated = true;
        }
    }

    pub fn files_iter(&self) -> SourceIterator<'_> {
        SourceIterator::new(self)
    }
}

impl Default for Source<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceFileId(u32);

impl SourceFileId {
    pub const MAIN: Self = SourceFileId(0);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Debug)]
pub enum SourceFile<'src> {
    Embedded {
        path: &'src str,
        content: &'src [u8],
        utf8_validated: bool,
    },
    EmbeddedOwned {
        path: &'src str,
        content: Vec<u8>,
        utf8_validated: bool,
    },
    File {
        path: PathBuf,
        mmap: Mmap,
        utf8_validated: bool,
    },
    FileLinked {
        path: &'src Path,
        mmap: Mmap,
        utf8_validated: bool,
    },
}

impl<'src> SourceFile<'src> {
    pub fn built_in(path: &'static str, content: &'static str) -> Self {
        SourceFile::Embedded {
            path,
            content: content.as_bytes(),
            utf8_validated: true,
        }
    }

    pub fn non_validated(path: &'src str, content: &'src [u8]) -> Self {
        SourceFile::Embedded {
            path,
            content,
            utf8_validated: false,
        }
    }

    pub fn validated(path: &'src str, content: &'src str) -> Self {
        SourceFile::Embedded {
            path,
            content: content.as_bytes(),
            utf8_validated: true,
        }
    }

    pub fn non_validated_owned(path: &'src str, content: Vec<u8>) -> Self {
        SourceFile::EmbeddedOwned {
            path,
            content,
            utf8_validated: false,
        }
    }

    pub fn validated_owned(path: &'src str, content: String) -> Self {
        SourceFile::EmbeddedOwned {
            path,
            content: content.into_bytes(),
            utf8_validated: true,
        }
    }

    pub fn file(path: PathBuf) -> Result<Self, SourceError> {
        let Ok(path) = path.canonicalize() else {
            error!(
                "Path does not exist or is invalid for this platform \"{}\"",
                path.display()
            );

            return Err(SourceError::InvalidPath {
                found: path.display().to_string(),
            });
        };

        if !path.is_file() {
            error!("Path does not point to a file: \"{}\"", path.display());

            return Err(SourceError::ExpectedFilePath {
                found: path.display().to_string(),
            });
        }

        if path.to_str().is_none() {
            error!("Path contains non-UTF-8 characters: {}", path.display());

            return Err(SourceError::ExpectedUtf8Path {
                found: path.display().to_string(),
            });
        }

        let file = File::open(&path).map_err(|error| SourceError::CannotOpen {
            io_error: error.kind(),
        })?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|error| SourceError::CannotOpen {
            io_error: error.kind(),
        })?;

        Ok(SourceFile::File {
            path,
            mmap,
            utf8_validated: false,
        })
    }

    pub fn file_linked(path: &'src Path, mmap: Mmap) -> Result<Self, SourceError> {
        Ok(SourceFile::FileLinked {
            path,
            mmap,
            utf8_validated: false,
        })
    }

    pub fn full_path(&self) -> &str {
        match self {
            Self::Embedded { path, .. } | Self::EmbeddedOwned { path, .. } => path,
            Self::File { path, .. } => path.to_str().expect("File path contains invalid UTF-8"),
            Self::FileLinked { path, .. } => {
                path.to_str().expect("File path contains invalid UTF-8")
            }
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Embedded { path, .. } | Self::EmbeddedOwned { path, .. } => path,
            Self::File { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("File name conatins invalid UTF-8"),
            Self::FileLinked { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("File name conatins invalid UTF-8"),
        }
    }

    pub fn is_utf8_validated(&self) -> bool {
        match self {
            Self::Embedded { utf8_validated, .. }
            | Self::EmbeddedOwned { utf8_validated, .. }
            | Self::File { utf8_validated, .. }
            | Self::FileLinked { utf8_validated, .. } => *utf8_validated,
        }
    }

    pub fn content_bytes(&self, span: Span) -> &[u8] {
        let full_source = self.content_as_bytes();
        let range = span.as_usize_range();

        full_source.get(range).unwrap_or_else(|| {
            let path = self.full_path();

            error!("Failed to get source at {path}:{span}");

            SOURCE_NOT_FOUND.as_bytes()
        })
    }

    pub fn content_str(&self, span: Span) -> &str {
        let full_source = self.content_as_str();
        let range = span.as_usize_range();

        full_source.get(range).unwrap_or(SOURCE_NOT_FOUND)
    }

    pub fn content_as_bytes(&self) -> &[u8] {
        match self {
            Self::Embedded { content, .. } => content,
            Self::EmbeddedOwned { content, .. } => content,
            Self::File { mmap, .. } | Self::FileLinked { mmap, .. } => mmap,
        }
    }

    pub fn content_as_str(&self) -> &str {
        fn handle_utf8_validation<'a>(path: &Path, source_bytes: &'a [u8]) -> &'a str {
            warn!(
                "Source file {} is being accessed before UTF-8 validation. Doing immediate \
                validation now. All files should be validated by the lexer before being accessed \
                to avoid this warning.",
                path.file_name().unwrap().display()
            );

            let utf8_bytes = match str::from_utf8(source_bytes) {
                Ok(str) => return str,
                Err(error) => {
                    error!(
                        "Source file {} contains invalid UTF-8 at byte index {}.",
                        path.display(),
                        error.valid_up_to()
                    );

                    &source_bytes[0..error.valid_up_to()]
                }
            };

            unsafe { str::from_utf8_unchecked(utf8_bytes) }
        }

        match self {
            Self::Embedded {
                path,
                content: source_bytes,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(source_bytes) }
                } else {
                    let path = Path::new(path);

                    handle_utf8_validation(path, source_bytes)
                }
            }
            Self::EmbeddedOwned {
                path,
                content: source_bytes,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(source_bytes) }
                } else {
                    let path = Path::new(path);

                    handle_utf8_validation(path, source_bytes)
                }
            }
            Self::File {
                path,
                mmap,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(mmap) }
                } else {
                    handle_utf8_validation(path, mmap)
                }
            }
            Self::FileLinked {
                path,
                mmap,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(mmap) }
                } else {
                    handle_utf8_validation(path, mmap)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
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
pub struct Span(u32, u32);

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

    pub fn start(&self) -> u32 {
        self.0
    }

    pub fn end(&self) -> u32 {
        self.1
    }

    pub fn length(&self) -> u32 {
        self.1.saturating_sub(self.0)
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
    source: &'a Source<'a>,
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
    type Item = (SourceFileId, &'a SourceFile<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let file_id = SourceFileId(self.position as u32);
        let file = self.source.files.get(self.position)?;

        self.position += 1;

        Some((file_id, file))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.source.files.len() - self.position;

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for SourceIterator<'_> {}

#[derive(Debug)]
pub enum SourceError {
    InvalidPath { found: String },
    ExpectedFilePath { found: String },
    ExpectedUtf8Path { found: String },
    SpanOutOfBounds { span: Span, length: usize },
    CannotOpen { io_error: io::ErrorKind },
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SourceError::InvalidPath { found } => write!(
                f,
                "The path \"{found}\" does not exist or is invalid for this platform."
            ),
            SourceError::ExpectedFilePath { found } => {
                write!(f, "The path \"{found}\" does not point to a file.")
            }
            SourceError::ExpectedUtf8Path { found } => {
                write!(f, "The path {found} contains non-UTF-8 characters.")
            }
            SourceError::SpanOutOfBounds { span, length } => {
                write!(
                    f,
                    "The span ({span}) is out of bounds, the file's length is {length}."
                )
            }
            SourceError::CannotOpen { io_error } => {
                write!(f, "Failed to open file: {io_error}")
            }
        }
    }
}
