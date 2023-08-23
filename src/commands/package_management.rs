use std::process::Command;

use crate::{Error, Macro, MacroInfo, Result, Value};

pub struct CoprRepositories;

impl Macro for CoprRepositories {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "enable_copr_repository",
            description: "Enable one or more COPR repositories.",
            group: "package management",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let repo_list_string = if let Ok(repo) = argument.as_string().cloned() {
            repo
        } else if let Ok(repos) = argument.as_list() {
            repos.iter().map(|value| value.to_string() + " ").collect()
        } else {
            return Err(crate::Error::ExpectedString {
                actual: argument.clone(),
            });
        };

        Command::new("fish")
            .arg("-c")
            .arg(format!("sudo dnf -y copr enable {repo_list_string}"))
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}

pub struct InstallPackage;

impl Macro for InstallPackage {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "install_package",
            description: "Install one or more packages.",
            group: "package management",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let package_list_string = if let Ok(package) = argument.as_string().cloned() {
            package
        } else if let Ok(packages) = argument.as_list() {
            packages
                .iter()
                .map(|value| value.to_string() + " ")
                .collect()
        } else {
            return Err(Error::ExpectedString {
                actual: argument.clone(),
            });
        };

        Command::new("fish")
            .arg("-c")
            .arg(format!("sudo dnf -y install {package_list_string}"))
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}

pub struct EnableRpmRepositories;

impl Macro for EnableRpmRepositories {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "enable_rpm_repositories",
            description: "Enable one or more RPM repositories.",
            group: "package management",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        if let Ok(repo) = argument.as_string() {
            Command::new("fish")
                .arg("-c")
                .arg(format!("sudo dnf -y config-manager --add-repo {repo}"))
                .spawn()?
                .wait()?;
        } else if let Ok(repos) = argument.as_list() {
            for repo in repos {
                Command::new("fish")
                    .arg("-c")
                    .arg(format!("sudo dnf -y config-manager --add-repo {repo}"))
                    .spawn()?
                    .wait()?;
            }
        } else {
            return Err(crate::Error::ExpectedString {
                actual: argument.clone(),
            });
        };

        Ok(Value::Empty)
    }
}

pub struct UninstallPackage;

impl Macro for UninstallPackage {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "uninstall_package",
            description: "Uninstall one or more packages.",
            group: "package management",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let package_list_string = if let Ok(package) = argument.as_string().cloned() {
            package
        } else if let Ok(packages) = argument.as_list() {
            packages
                .iter()
                .map(|value| value.to_string() + " ")
                .collect()
        } else {
            return Err(Error::ExpectedString {
                actual: argument.clone(),
            });
        };

        Command::new("fish")
            .arg("-c")
            .arg(format!("sudo dnf -y remove {package_list_string}"))
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}

pub struct UpgradePackages;

impl Macro for UpgradePackages {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "upgrade_packages",
            description: "Upgrade all installed packages.",
            group: "package management",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        argument.as_empty()?;

        Command::new("fish")
            .arg("-c")
            .arg("sudo dnf -y upgrade")
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}
