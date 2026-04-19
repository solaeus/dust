use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read},
    ops::Range,
    path::{Path, PathBuf},
};

use annotate_snippets::{Group, Level, Renderer};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::error::{DustError, Error, ErrorContext, ErrorKind};

#[derive(Debug, Clone)]
pub struct Source<'src> {
    code: Vec<SourceCode<'src>>,
}

impl<'src> Source<'src> {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            code: Vec::with_capacity(capacity),
        }
    }

    pub fn file_count(&self) -> usize {
        self.code.len()
    }

    pub fn code(&self) -> &[SourceCode<'src>] {
        &self.code
    }

    pub fn add_code(&mut self, file: SourceCode<'src>) -> SourceCodeId {
        let id = SourceCodeId(self.code.len() as u32);

        self.code.push(file);

        id
    }

    pub fn get_code(&self, source_id: SourceCodeId) -> Result<&SourceCode<'src>, SourceError> {
        self.code
            .get(source_id.0 as usize)
            .ok_or(SourceError::MissingSourceFile(source_id))
    }

    pub fn get_content(&self, position: Position) -> Result<&str, SourceError> {
        self.get_code(position.source_id)?.get_str(position.span)
    }

    pub fn set_utf8_validated(&mut self, source_id: SourceCodeId) {
        if let Some(
            SourceCode::File { utf8_validated, .. }
            | SourceCode::Borrowed { utf8_validated, .. }
            | SourceCode::Owned { utf8_validated, .. },
        ) = self.code.get_mut(source_id.0 as usize)
        {
            *utf8_validated = true;
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = SourceCodeId> {
        (0..self.code.len() as u32).map(SourceCodeId)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceCodeId, &SourceCode<'src>)> {
        self.code
            .iter()
            .enumerate()
            .map(|(index, file)| (SourceCodeId(index as u32), file))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SourceCodeId, &mut SourceCode<'src>)> {
        self.code
            .iter_mut()
            .enumerate()
            .map(|(index, file)| (SourceCodeId(index as u32), file))
    }
}

impl Default for Source<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceCodeId(u32);

impl SourceCodeId {
    pub const MAIN: Self = SourceCodeId(0);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum SourceCode<'src> {
    File {
        path: PathBuf,
        content: Vec<u8>,
        utf8_validated: bool,
    },
    Borrowed {
        name: &'src str,
        content: &'src [u8],
        utf8_validated: bool,
    },
    Owned {
        name: &'src str,
        content: Vec<u8>,
        utf8_validated: bool,
    },
}

impl<'src> SourceCode<'src> {
    pub fn file<P: AsRef<Path>>(path: P) -> Result<Self, SourceError> {
        let path = path
            .as_ref()
            .canonicalize()
            .map_err(|error| SourceError::CannotOpen {
                io_error: error.kind(),
            })?;
        let metadata = path.metadata().map_err(|error| SourceError::CannotOpen {
            io_error: error.kind(),
        })?;

        if !metadata.is_file() {
            return Err(SourceError::ExpectedFilePath {
                found: path.display().to_string(),
            });
        }

        let mut content = Vec::with_capacity(metadata.len() as usize);

        File::options()
            .read(true)
            .open(&path)
            .map_err(|error| SourceError::CannotOpen {
                io_error: error.kind(),
            })?
            .read_to_end(&mut content)
            .map_err(|error| SourceError::CannotOpen {
                io_error: error.kind(),
            })?;

        Ok(SourceCode::File {
            path,
            content,
            utf8_validated: false,
        })
    }

    pub fn borrowed(name: &'src str, content: &'src [u8]) -> Self {
        SourceCode::Borrowed {
            name,
            content,
            utf8_validated: false,
        }
    }

    pub const fn validated_borrowed(name: &'src str, content: &'src str) -> Self {
        SourceCode::Borrowed {
            name,
            content: content.as_bytes(),
            utf8_validated: true,
        }
    }

    pub fn owned(name: &'src str, content: Vec<u8>) -> Self {
        SourceCode::Owned {
            name,
            content,
            utf8_validated: false,
        }
    }

    pub fn validated_owned(name: &'src str, content: String) -> Self {
        SourceCode::Owned {
            name,
            content: content.into_bytes(),
            utf8_validated: true,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::File { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Borrowed { name, .. } | Self::Owned { name, .. } => name,
            Self::File { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<invalid file name>"),
        }
    }

    pub fn path_or_name(&self) -> Cow<'_, str> {
        match self {
            Self::Borrowed { name, .. } | Self::Owned { name, .. } => Cow::Borrowed(name),
            Self::File { path, .. } => path.to_string_lossy(),
        }
    }

    pub fn utf8_validated(&self) -> bool {
        match self {
            Self::Borrowed { utf8_validated, .. }
            | Self::Owned { utf8_validated, .. }
            | Self::File { utf8_validated, .. } => *utf8_validated,
        }
    }

    pub fn set_utf8_validated(&mut self, validated: bool) {
        match self {
            Self::Borrowed { utf8_validated, .. }
            | Self::Owned { utf8_validated, .. }
            | Self::File { utf8_validated, .. } => *utf8_validated = validated,
        }
    }

    pub fn get_bytes(&self, span: Span) -> Result<&[u8], SourceError> {
        let full_source = self.content_as_bytes();
        let range = span.as_usize_range();

        full_source
            .get(range)
            .ok_or(SourceError::FileContentOutOfBounds {
                span,
                length: full_source.len(),
            })
    }

    pub fn get_str(&self, span: Span) -> Result<&str, SourceError> {
        let full_source = self.content_as_str();

        full_source
            .get(span.as_usize_range())
            .ok_or(SourceError::FileContentOutOfBounds {
                span,
                length: full_source.len(),
            })
    }

    pub fn content_as_bytes(&self) -> &[u8] {
        match self {
            Self::Borrowed { content, .. } => content,
            Self::File { content, .. } | Self::Owned { content, .. } => content,
        }
    }

    pub fn content_as_str(&self) -> &str {
        fn handle_utf8_validation<'a>(path_or_name: &str, source_bytes: &'a [u8]) -> &'a str {
            warn!(
                "Source file {path_or_name} is being accessed before UTF-8 validation. Doing \
                immediate validation now. All files should be validated by the lexer before being \
                accessed to avoid this warning.",
            );

            let utf8_bytes = match str::from_utf8(source_bytes) {
                Ok(str) => return str,
                Err(error) => {
                    error!(
                        "Source file {path_or_name} contains invalid UTF-8 at byte index {}.",
                        error.valid_up_to()
                    );

                    &source_bytes[0..error.valid_up_to()]
                }
            };

            unsafe { str::from_utf8_unchecked(utf8_bytes) }
        }

        match self {
            Self::Borrowed {
                name,
                content: source_bytes,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(source_bytes) }
                } else {
                    handle_utf8_validation(name, source_bytes)
                }
            }
            Self::Owned {
                name,
                content: source_bytes,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(source_bytes) }
                } else {
                    handle_utf8_validation(name, source_bytes)
                }
            }
            Self::File {
                path,
                content,
                utf8_validated,
            } => {
                if *utf8_validated {
                    unsafe { str::from_utf8_unchecked(content) }
                } else {
                    handle_utf8_validation(&path.to_string_lossy(), content)
                }
            }
        }
    }
}

/// Represents a slice of a file's content that can be read from the `Source`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub source_id: SourceCodeId,
    pub span: Span,
}

impl Position {
    pub fn new(source_id: SourceCodeId, span: Span) -> Self {
        Self { source_id, span }
    }

    pub fn shrink(self, offset: u32) -> Position {
        Position {
            source_id: self.source_id,
            span: self.span.shrink(offset),
        }
    }
}

/// Half-open range of byte indices in a source file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span(u32, u32);

impl Span {
    pub fn new<T: IntoSpanIndex>(start: T, end: T) -> Self {
        let start = start.into_span_index();
        let end = end.into_span_index();

        debug_assert!(start <= end);

        Self(start, end)
    }

    pub fn empty() -> Self {
        Self(0, 0)
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
        self.1 - self.0
    }

    pub fn join(self, other: &Span) -> Span {
        let new_start = self.0.min(other.0);
        let new_end = self.1.max(other.1).max(new_start);

        Span(new_start, new_end)
    }

    pub fn shrink(self, offset: u32) -> Span {
        let new_start = self.0.saturating_add(offset);
        let new_end = self.1.saturating_sub(offset).max(new_start);

        Span(new_start, new_end)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.0, self.1)
    }
}

pub trait IntoSpanIndex {
    fn into_span_index(self) -> u32;
}

impl IntoSpanIndex for u32 {
    fn into_span_index(self) -> u32 {
        self
    }
}

impl IntoSpanIndex for usize {
    fn into_span_index(self) -> u32 {
        self as u32
    }
}

#[cfg(test)]
impl IntoSpanIndex for i32 {
    fn into_span_index(self) -> u32 {
        self as u32
    }
}

#[derive(Debug)]
pub enum SourceError {
    // User errors
    CannotOpen { io_error: io::ErrorKind },
    ExpectedFilePath { found: String },

    // Internal errors
    MissingSourceFile(SourceCodeId),
    FileContentOutOfBounds { span: Span, length: usize },
}

impl<'src> SourceError {
    pub fn to_full_error(self) -> Error<'src> {
        Error::new(vec![ErrorKind::Source(self)], ErrorContext::None)
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

impl<'src> DustError<'src> for SourceError {
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
            SourceError::MissingSourceFile(_) | SourceError::FileContentOutOfBounds { .. } => {
                self.add_internal_report(reports);

                return;
            }
        };

        reports.push(group);
    }
}
