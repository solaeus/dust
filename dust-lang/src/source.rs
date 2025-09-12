use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Source {
    Script {
        name: Arc<String>,
        content: Arc<String>,
    },
    Files(Vec<SourceFile>),
}

impl Source {
    pub fn program_name(&self) -> &str {
        match self {
            Source::Script { name, .. } => name,
            Source::Files(sources) => {
                if let Some(SourceFile { name, .. }) = sources.first() {
                    name
                } else {
                    "unknown"
                }
            }
        }
    }

    pub fn get_file_source(&self, index: usize) -> Option<&str> {
        match self {
            Source::Script {
                content: source, ..
            } => {
                if index == 0 {
                    Some(source)
                } else {
                    None
                }
            }
            Source::Files(sources) => sources
                .get(index)
                .map(|SourceFile { source, .. }| source.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: Arc<String>,
    pub source: Arc<String>,
}
