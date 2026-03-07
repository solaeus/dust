use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs::File,
    io,
    ops::Range,
    path::{Path, PathBuf},
};

use annotate_snippets::{Group, Level, Renderer};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::error::{AnnotatedError, InternalError};

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
        if let Some(
            SourceFile::File { utf8_validated, .. }
            | SourceFile::Embedded { utf8_validated, .. }
            | SourceFile::EmbeddedOwned { utf8_validated, .. },
        ) = self.files.get_mut(file_id.0 as usize)
        {
            *utf8_validated = true;
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = SourceFileId> {
        (0..self.files.len() as u32).map(SourceFileId)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceFileId, &SourceFile<'src>)> {
        self.files
            .iter()
            .enumerate()
            .map(|(index, file)| (SourceFileId(index as u32), file))
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
    File {
        path: PathBuf,
        mmap: Mmap,
        utf8_validated: bool,
    },
    Embedded {
        name: &'src str,
        content: &'src [u8],
        utf8_validated: bool,
    },
    EmbeddedOwned {
        name: &'src str,
        content: Vec<u8>,
        utf8_validated: bool,
    },
}

impl<'src> SourceFile<'src> {
    pub fn non_validated(name: &'src str, content: &'src [u8]) -> Self {
        SourceFile::Embedded {
            name,
            content,
            utf8_validated: false,
        }
    }

    pub const fn validated(name: &'src str, content: &'src str) -> Self {
        SourceFile::Embedded {
            name,
            content: content.as_bytes(),
            utf8_validated: true,
        }
    }

    pub fn non_validated_owned(name: &'src str, content: Vec<u8>) -> Self {
        SourceFile::EmbeddedOwned {
            name,
            content,
            utf8_validated: false,
        }
    }

    pub fn validated_owned(name: &'src str, content: String) -> Self {
        SourceFile::EmbeddedOwned {
            name,
            content: content.into_bytes(),
            utf8_validated: true,
        }
    }

    pub fn file_from_path(path: &Path) -> Result<Self, SourceError> {
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

        Ok(SourceFile::File {
            path,
            mmap,
            utf8_validated: false,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::File { path, .. } => Some(path.as_path()),
            _ => None,
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Embedded { name, .. } | Self::EmbeddedOwned { name, .. } => name,
            Self::File { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("File name conatins invalid UTF-8"),
        }
    }

    pub fn path_or_name(&self) -> Cow<'_, str> {
        match self {
            Self::Embedded { name, .. } | Self::EmbeddedOwned { name, .. } => Cow::Borrowed(name),
            Self::File { path, .. } => path.to_string_lossy(),
        }
    }

    pub fn is_utf8_validated(&self) -> bool {
        match self {
            Self::Embedded { utf8_validated, .. }
            | Self::EmbeddedOwned { utf8_validated, .. }
            | Self::File { utf8_validated, .. } => *utf8_validated,
        }
    }

    pub fn content_bytes(&self, span: Span) -> Result<&[u8], InternalError> {
        let full_source = self.content_as_bytes();
        let range = span.as_usize_range();

        full_source
            .get(range)
            .ok_or(InternalError::MissingSourceFileContent {
                span,
                length: full_source.len(),
            })
    }

    pub fn content_str(&self, span: Span) -> Result<&str, InternalError> {
        let full_source = self.content_as_str();
        let range = span.as_usize_range();

        full_source
            .get(range)
            .ok_or(InternalError::MissingSourceFileContent {
                span,
                length: full_source.len(),
            })
    }

    pub fn content_as_bytes(&self) -> &[u8] {
        match self {
            Self::Embedded { content, .. } => content,
            Self::EmbeddedOwned { content, .. } => content,
            Self::File { mmap, .. } => mmap,
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
                name: path,
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
                name: path,
                content: source_bytes,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(source_bytes) }
                } else {
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
                    handle_utf8_validation(&path.to_string_lossy(), mmap)
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
        (self.0 as usize)..(self.1 as usize)
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

#[derive(Debug)]
pub enum SourceError {
    CannotOpen { io_error: io::ErrorKind },
    ExpectedFilePath { found: String },
    ExpectedUtf8Path { found: String },
    InvalidPath { found: String },
}

impl SourceError {
    pub fn print_and_exit(&self) -> ! {
        eprintln!("{self}");

        std::process::exit(1);
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut groups = Vec::new();

        self.add_report((), &mut groups);

        let renderer = Renderer::styled();
        let report = renderer.render(&groups);

        write!(f, "{report}")
    }
}

impl<'src> AnnotatedError<'src> for SourceError {
    type Context = ();

    fn add_report(&self, _: Self::Context, reports: &mut Vec<Group<'src>>) {
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
