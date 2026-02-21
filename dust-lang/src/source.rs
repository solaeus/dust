use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io,
    ops::Range,
    path::{Path, PathBuf},
};

use annotate_snippets::{Group, Level};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::dust_error::{AnnotatedError, DustError, InternalError};

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

    pub fn get_file(&self, file_id: SourceFileId) -> Result<&SourceFile<'src>, InternalError> {
        self.files
            .get(file_id.0 as usize)
            .ok_or(InternalError::MissingSourceFile(file_id))
    }

    pub fn get_file_content(&self, position: &Position) -> Result<&str, InternalError> {
        self.get_file(position.file_id)?.content_str(position.span)
    }

    pub fn set_utf8_validated(&mut self, file_id: SourceFileId) {
        if let Some(SourceFile::BaseFile { utf8_validated, .. }) =
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
    BaseFile {
        path: String,
        mmap: Mmap,
        utf8_validated: bool,
    },
    ModuleFile {
        path: &'src str,
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

    pub fn base_file(path: PathBuf) -> Result<Self, SourceError> {
        let Ok(path) = path.canonicalize() else {
            return Err(SourceError::InvalidPath {
                found: path.display().to_string(),
            });
        };

        if !path.is_file() {
            return Err(SourceError::ExpectedFilePath {
                found: path.display().to_string(),
            });
        }

        let file = File::open(&path).map_err(|error| SourceError::CannotOpen {
            io_error: error.kind(),
        })?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|error| SourceError::CannotOpen {
            io_error: error.kind(),
        })?;
        let path = match path.into_os_string().into_string() {
            Ok(string) => string,
            Err(os_string) => {
                return Err(SourceError::ExpectedUtf8Path {
                    found: os_string.display().to_string(),
                });
            }
        };

        Ok(SourceFile::BaseFile {
            path,
            mmap,
            utf8_validated: false,
        })
    }

    pub fn module_file(path: &'src str, mmap: Mmap) -> Self {
        SourceFile::ModuleFile {
            path,
            mmap,
            utf8_validated: false,
        }
    }

    pub fn full_path(&self) -> &str {
        match self {
            Self::Embedded { path, .. }
            | Self::EmbeddedOwned { path, .. }
            | Self::ModuleFile { path, .. } => path,
            Self::BaseFile { path, .. } => path,
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Embedded { path, .. } | Self::EmbeddedOwned { path, .. } => path,
            Self::BaseFile { path, .. } => Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("File name conatins invalid UTF-8"),
            Self::ModuleFile { path, .. } => Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("File name conatins invalid UTF-8"),
        }
    }

    pub fn is_utf8_validated(&self) -> bool {
        match self {
            Self::Embedded { utf8_validated, .. }
            | Self::EmbeddedOwned { utf8_validated, .. }
            | Self::BaseFile { utf8_validated, .. }
            | Self::ModuleFile { utf8_validated, .. } => *utf8_validated,
        }
    }

    pub fn content_bytes(&self, span: Span) -> Result<&[u8], DustError> {
        let full_source = self.content_as_bytes();
        let range = span.as_usize_range();

        full_source.get(range).ok_or_else(|| {
            DustError::Internal(InternalError::MissingSourceFileContent {
                span,
                length: full_source.len(),
            })
        })
    }

    pub fn content_str(&self, span: Span) -> Result<&str, InternalError> {
        let full_source = self.content_as_str();
        let range = span.as_usize_range();

        full_source
            .get(range)
            .ok_or_else(|| InternalError::MissingSourceFileContent {
                span,
                length: full_source.len(),
            })
    }

    pub fn content_as_bytes(&self) -> &[u8] {
        match self {
            Self::Embedded { content, .. } => content,
            Self::EmbeddedOwned { content, .. } => content,
            Self::BaseFile { mmap, .. } | Self::ModuleFile { mmap, .. } => mmap,
        }
    }

    pub fn content_as_str(&self) -> &str {
        fn handle_utf8_validation<'a>(path: &str, source_bytes: &'a [u8]) -> &'a str {
            warn!(
                "Source file {} is being accessed before UTF-8 validation. Doing immediate \
                validation now. All files should be validated by the lexer before being accessed \
                to avoid this warning.",
                Path::new(path).file_name().unwrap().display()
            );

            let utf8_bytes = match str::from_utf8(source_bytes) {
                Ok(str) => return str,
                Err(error) => {
                    error!(
                        "Source file {} contains invalid UTF-8 at byte index {}.",
                        Path::new(path).display(),
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
                    handle_utf8_validation(path, source_bytes)
                }
            }
            Self::BaseFile {
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
            Self::ModuleFile {
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
    CannotOpen { io_error: io::ErrorKind },
    ExpectedFilePath { found: String },
    ExpectedUtf8Path { found: String },
    InvalidPath { found: String },
}

impl<'src> AnnotatedError<'src> for SourceError {
    type Context = ();

    fn annotated_error(&self, _: &Self::Context, reports: &mut Vec<Group<'src>>) {
        let group = match self {
            SourceError::CannotOpen { io_error } => {
                let title = "Cannot open source file".to_string();
                let message = io_error.to_string();

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(message))
            }
            SourceError::ExpectedFilePath { found } => {
                let title = "Expected file path".to_string();
                let message = format!("\"{found}\" exists but is not a file.");
                let help = if Path::new(found).is_dir() {
                    "You may have meant \"{found}.ds\" or \"{found}/mod.ds\"."
                } else {
                    "Please provide a path to a Dust source file."
                };

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(message))
                    .element(Level::HELP.message(help))
            }
            SourceError::ExpectedUtf8Path { found } => {
                let title = "Expected UTF-8 file path".to_string();
                let message = format!(
                    "\"{found}\" contains non-UTF-8 characters. Dust file paths must be UTF-8."
                );

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(message))
            }
            SourceError::InvalidPath { found } => {
                let title = "Invalid file path".to_string();
                let message = format!("\"{found}\" is not a valid path.");

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(message))
            }
        };

        reports.push(group);
    }
}
