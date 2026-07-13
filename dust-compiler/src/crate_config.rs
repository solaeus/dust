use std::{
    collections::HashMap,
    fs::{File, read_to_string},
    io::{self, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::source::{Source, SourceCode, SourceError};

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
pub struct CrateConfig {
    pub package: PackageConfig,
    pub _dependencies: HashMap<String, DependencyConfig>,
}

impl CrateConfig {
    pub fn example() -> Self {
        Self {
            package: PackageConfig::example(),
            _dependencies: HashMap::new(),
        }
    }

    pub fn read_from_path(path: &Path) -> Result<Self, io::Error> {
        let content = read_to_string(path)?;

        toml::from_str(&content).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), io::Error> {
        #[expect(clippy::disallowed_methods)]
        let toml = toml::to_string_pretty(self)
            .expect("`DustConfig` is malformed and cannot be serialized to TOML");

        File::options()
            .create_new(true)
            .write(true)
            .open(path)?
            .write_all(toml.as_bytes())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub authors: Vec<String>,
    pub version: String,
    pub programs: Vec<ProgramConfig>,
}

impl PackageConfig {
    pub fn example() -> Self {
        Self {
            name: "example_dust_package".to_string(),
            authors: Vec::new(),
            version: "0.1.0".to_string(),
            programs: vec![ProgramConfig::example()],
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

    pub fn read_code(&self, source: &mut Source, crate_path: &Path) -> Result<(), SourceError> {
        let program_path = crate_path.join("src").join(&self.path);
        let code = SourceCode::file(program_path)?;

        source.add_code(code);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub git: Option<String>,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug)]
pub enum ConfigError {}
