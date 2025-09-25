use memmap2::Mmap;

#[derive(Debug)]
pub enum Source {
    Script { name: String, source: Vec<u8> },
    Files(Vec<SourceFile>),
}

impl Source {
    pub fn script(name: String, source: Vec<u8>) -> Self {
        Source::Script { name, source }
    }

    pub fn files(file_count: usize) -> Self {
        Source::Files(Vec::with_capacity(file_count))
    }

    pub fn len(&self) -> usize {
        match self {
            Source::Script { .. } => 1,
            Source::Files(sources) => sources.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_file(&mut self, name: String, source_code: Mmap) -> SourceFileId {
        let id = SourceFileId(self.len() as u32);

        match self {
            Source::Files(sources) => {
                sources.push(SourceFile { name, source_code });
            }
            Source::Script { .. } => {}
        }

        id
    }

    pub fn program_name(&self) -> &str {
        match self {
            Source::Script { name, .. } => name,
            Source::Files(sources) => {
                if let Some(source_file) = sources.first().as_ref() {
                    &source_file.name
                } else {
                    "anonymous"
                }
            }
        }
    }

    pub fn into_program_name(self) -> String {
        match self {
            Source::Script { name, .. } => name,
            Source::Files(sources) => {
                if let Some(source_file) = sources.into_iter().next() {
                    source_file.name
                } else {
                    "anonymous".to_string()
                }
            }
        }
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
    pub source_code: Mmap,
}
