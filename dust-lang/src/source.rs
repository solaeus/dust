use std::slice;

#[derive(Debug, Clone)]
pub enum Source {
    Script(SourceFile),
    Files(Vec<SourceFile>),
}

impl Source {
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

    pub fn get_file(&self, index: u32) -> Option<&SourceFile> {
        match self {
            Source::Script(source_file) => {
                if index == 0 {
                    Some(source_file)
                } else {
                    None
                }
            }
            Source::Files(sources) => sources.get(index as usize),
        }
    }

    pub fn files(&self) -> &[SourceFile] {
        match self {
            Source::Script(source_file) => slice::from_ref(source_file),
            Source::Files(sources) => sources.as_slice(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub source_code: String,
}
