use serde::{Deserialize, Serialize};

pub const PROJECT_CONFIG_PATH: &str = "dust.toml";
pub const EXAMPLE_PROGRAM: &str = "\
let name = read_line(\"What is your name?\");

write_line(\"Hello, \" + name + \"!\");
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub authors: Vec<String>,
    pub version: String,
}

impl ProjectConfig {
    pub fn example() -> Self {
        Self {
            name: "example_dust_project".to_string(),
            authors: Vec::new(),
            version: "0.1.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    pub name: String,
    pub path: String,
}

impl ProgramConfig {
    pub fn example() -> Self {
        Self {
            name: "example_program".to_string(),
            path: "src/main.dust".to_string(),
        }
    }
}
