use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Source {
    Script(Arc<SourceFile>),
    Files(Vec<Arc<SourceFile>>),
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
            Source::Script(source_file) => source_file.name.as_str(),
            Source::Files(sources) => {
                if let Some(source_file) = sources.first().as_ref() {
                    source_file.name.as_str()
                } else {
                    "unknown"
                }
            }
        }
    }

    pub fn get_file(&self, index: usize) -> Option<&Arc<SourceFile>> {
        match self {
            Source::Script(source_file) => {
                if index == 0 {
                    Some(source_file)
                } else {
                    None
                }
            }
            Source::Files(sources) => sources.get(index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub source: String,
}
