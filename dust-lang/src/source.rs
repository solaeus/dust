use std::slice;

#[derive(Debug, Clone)]
pub enum Source {
    Script(SourceFile),
    Files(Vec<SourceFile>),
}

impl Source {
    pub fn script(name: String, source_code: String) -> Self {
        Source::Script(SourceFile { name, source_code })
    }

    pub fn files(file_count: usize) -> Self {
        Source::Files(Vec::with_capacity(file_count))
    }

    pub fn add_file(&mut self, name: String, source_code: String) -> SourceFileId {
        let id = SourceFileId(self.len() as u32);

        match self {
            Source::Files(sources) => {
                sources.push(SourceFile { name, source_code });
            }
            Source::Script(_) => {}
        }

        id
    }

    pub fn get_file(&self, file_id: SourceFileId) -> Option<&SourceFile> {
        match self {
            Source::Script(source_file) => {
                if file_id.0 == 0 {
                    Some(source_file)
                } else {
                    None
                }
            }
            Source::Files(sources) => sources.get(file_id.0 as usize),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Source::Script(_) => 1,
            Source::Files(sources) => sources.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn program_name(&self) -> &str {
        match self {
            Source::Script(source_file) => &source_file.name,
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
            Source::Script(source_file) => source_file.name,
            Source::Files(sources) => {
                if let Some(source_file) = sources.into_iter().next() {
                    source_file.name
                } else {
                    "anonymous".to_string()
                }
            }
        }
    }

    pub fn get_files(&self) -> &[SourceFile] {
        match self {
            Source::Script(source_file) => slice::from_ref(source_file),
            Source::Files(sources) => sources.as_slice(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SourceFileId(pub u32);

impl SourceFileId {
    const MAIN: Self = SourceFileId(0);
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub source_code: String,
}
