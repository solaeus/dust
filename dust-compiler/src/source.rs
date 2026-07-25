use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, Read},
    ops::Range,
    path::{Path, PathBuf},
};

use annotate_snippets::{Group, Level, Renderer};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::error::DustError;

#[derive(Debug, Clone)]
pub struct Source<'src> {
    code: Vec<Code<'src>>,
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

    pub fn code(&self) -> &[Code<'src>] {
        &self.code
    }

    pub fn add_code(&mut self, file: Code<'src>) -> CodeId {
        let id = CodeId(self.code.len() as u32);

        self.code.push(file);

        id
    }

    pub fn get_code(&self, code_id: CodeId) -> &Code<'src> {
        &self.code[code_id.0 as usize]
    }

    pub(crate) fn get_code_mut(&mut self, code_id: CodeId) -> &mut Code<'src> {
        &mut self.code[code_id.0 as usize]
    }

    pub fn get_content(&self, position: Position) -> Result<&str, SourceError> {
        self.get_code(position.code_id).get_str(position.span)
    }

    pub fn set_utf8_validated(&mut self, code_id: CodeId) {
        if let Some(code) = self.code.get_mut(code_id.0 as usize) {
            code.set_utf8_validated(true);
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = CodeId> {
        (0..self.code.len() as u32).map(CodeId)
    }

    pub fn iter(&self) -> impl Iterator<Item = (CodeId, &Code<'src>)> {
        self.code
            .iter()
            .enumerate()
            .map(|(index, file)| (CodeId(index as u32), file))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (CodeId, &mut Code<'src>)> {
        self.code
            .iter_mut()
            .enumerate()
            .map(|(index, file)| (CodeId(index as u32), file))
    }
}

impl Default for Source<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CodeId(u32);

impl CodeId {
    pub const MAIN: Self = CodeId(0);

    pub fn inner(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Code<'src> {
    inner: CodeInner<'src>,
    utf8_validated: bool,
    declaration_scanned: bool,
}

#[derive(Clone)]
pub enum CodeInner<'src> {
    File {
        path: PathBuf,
        content: Vec<u8>,
    },
    Borrowed {
        name: &'src str,
        content: &'src [u8],
    },
    Owned {
        name: &'src str,
        content: Vec<u8>,
    },
}

impl<'src> Code<'src> {
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

        Ok(Code {
            inner: CodeInner::File { path, content },
            utf8_validated: false,
            declaration_scanned: false,
        })
    }

    pub fn unvalidated(name: &'src str, content: &'src [u8]) -> Self {
        Code {
            inner: CodeInner::Borrowed { name, content },
            utf8_validated: false,
            declaration_scanned: false,
        }
    }

    pub fn unvalidated_owned(name: &'src str, content: Vec<u8>) -> Self {
        Code {
            inner: CodeInner::Owned { name, content },
            utf8_validated: false,
            declaration_scanned: false,
        }
    }

    pub const fn validated(name: &'src str, content: &'src str) -> Self {
        Code {
            inner: CodeInner::Borrowed {
                name,
                content: content.as_bytes(),
            },
            utf8_validated: true,
            declaration_scanned: false,
        }
    }

    pub fn validated_owned(name: &'src str, content: String) -> Self {
        Code {
            inner: CodeInner::Owned {
                name,
                content: content.into_bytes(),
            },
            utf8_validated: true,
            declaration_scanned: false,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.inner {
            CodeInner::File { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn file_name(&self) -> &str {
        match &self.inner {
            CodeInner::Borrowed { name, .. } | CodeInner::Owned { name, .. } => name,
            CodeInner::File { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<invalid file name>"),
        }
    }

    pub fn path_or_name(&self) -> Cow<'_, str> {
        match &self.inner {
            CodeInner::Borrowed { name, .. } | CodeInner::Owned { name, .. } => Cow::Borrowed(name),
            CodeInner::File { path, .. } => path.to_string_lossy(),
        }
    }

    pub fn utf8_validated(&self) -> bool {
        self.utf8_validated
    }

    pub fn set_utf8_validated(&mut self, validated: bool) {
        self.utf8_validated = validated;
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
        match &self.inner {
            CodeInner::Borrowed { content, .. } => content,
            CodeInner::File { content, .. } | CodeInner::Owned { content, .. } => content,
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

        match &self.inner {
            CodeInner::Borrowed { name, content } => {
                if self.utf8_validated {
                    unsafe { str::from_utf8_unchecked(content) }
                } else {
                    handle_utf8_validation(name, content)
                }
            }
            CodeInner::Owned { name, content } => {
                if self.utf8_validated {
                    unsafe { str::from_utf8_unchecked(content) }
                } else {
                    handle_utf8_validation(name, content)
                }
            }
            CodeInner::File { path, content } => {
                if self.utf8_validated {
                    unsafe { str::from_utf8_unchecked(content) }
                } else {
                    handle_utf8_validation(&path.to_string_lossy(), content)
                }
            }
        }
    }
}

impl Debug for CodeInner<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::File { path, content } => f
                .debug_struct("File")
                .field("path", path)
                .field("content", &String::from_utf8_lossy(content))
                .finish(),
            Self::Borrowed { name, content } => f
                .debug_struct("Borrowed")
                .field("name", name)
                .field("content", &String::from_utf8_lossy(content))
                .finish(),
            Self::Owned { name, content } => f
                .debug_struct("Owned")
                .field("name", name)
                .field("content", &String::from_utf8_lossy(content))
                .finish(),
        }
    }
}

/// Represents a slice of a file's content that can be read from the `Source`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub code_id: CodeId,
    pub span: Span,
}

impl Position {
    pub fn new(code_id: CodeId, span: Span) -> Self {
        Self { code_id, span }
    }

    pub fn shrink(self, offset: u32) -> Position {
        Position {
            code_id: self.code_id,
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

#[derive(Clone, Debug)]
pub enum SourceError {
    // User errors
    CannotOpen { io_error: io::ErrorKind },
    ExpectedFilePath { found: String },

    // Internal errors
    FileContentOutOfBounds { span: Span, length: usize },
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
    type Info = ();

    fn add_report(&self, _: Self::Info, reports: &mut Vec<Group<'src>>) {
        let group = match self {
            SourceError::CannotOpen { io_error } => {
                let title = "Cannot open source file";
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
            SourceError::FileContentOutOfBounds { .. } => {
                self.add_internal_report(reports);

                return;
            }
        };

        reports.push(group);
    }
}
