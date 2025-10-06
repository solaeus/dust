use serde::{Deserialize, Serialize};

pub const PROJECT_CONFIG_PATH: &str = "dust.toml";
pub const DEFAULT_PROGRAM_PATH: &str = "src/main.ds";
pub const EXAMPLE_PROGRAM: &str = "\
use lib::say_hello;

say_hello();
";
pub const EXAMPLE_LIBRARY: &str = "\
pub fn say_hello() {
    write_line(\"Welcome to Dust!\");
    write_line(\"What is your name?\");

    let name = read_line();

    write_line(\"Hello, \" + name + \"!\");
}
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub authors: Vec<String>,
    pub version: String,
    pub program: Option<ProgramConfig>,
}

impl ProjectConfig {
    pub fn example() -> Self {
        Self {
            name: "example_dust_project".to_string(),
            authors: Vec::new(),
            version: "0.1.0".to_string(),
            program: Some(ProgramConfig::example()),
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
            path: DEFAULT_PROGRAM_PATH.to_string(),
        }
    }
}
