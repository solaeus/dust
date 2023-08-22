use std::process::Command;

use sysinfo::{DiskExt, System, SystemExt};

use crate::{Macro, MacroInfo, Result, Table, Value};

pub struct ListDisks;

impl Macro for ListDisks {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "list_disks",
            description: "List all block devices.",
            group: "disks",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        argument.as_empty()?;

        let mut sys = System::new_all();
        sys.refresh_all();

        let mut disk_table = Table::new(vec![
            "name".to_string(),
            "kind".to_string(),
            "file system".to_string(),
            "mount point".to_string(),
            "total space".to_string(),
            "available space".to_string(),
            "is removable".to_string(),
        ]);

        for disk in sys.disks() {
            let name = disk.name().to_string_lossy().to_string();
            let kind = disk.kind();
            let file_system = String::from_utf8_lossy(disk.file_system()).to_string();
            let mount_point = disk.mount_point().to_str().unwrap().to_string();
            let total_space = disk.total_space() as i64;
            let available_space = disk.available_space() as i64;
            let is_removable = disk.is_removable();

            let row = vec![
                Value::String(name),
                Value::String(format!("{kind:?}")),
                Value::String(file_system),
                Value::String(mount_point),
                Value::Integer(total_space),
                Value::Integer(available_space),
                Value::Boolean(is_removable),
            ];

            disk_table.insert(row)?;
        }

        Ok(Value::Table(disk_table))
    }
}

pub struct Partition;

impl Macro for Partition {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
            identifier: "partition",
            description: "Partition a disk, clearing its content.",
            group: "disks",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_map()?;
        let path = argument
            .get_value("path")?
            .unwrap_or(Value::Empty)
            .as_string()?
            .clone();
        let label = argument
            .get_value("label")?
            .unwrap_or(Value::Empty)
            .as_string()?
            .clone();
        let name = argument
            .get_value("name")?
            .unwrap_or(Value::Empty)
            .as_string()?
            .clone();
        let filesystem = argument
            .get_value("filesystem")?
            .unwrap_or(Value::Empty)
            .as_string()?
            .clone();
        let range = argument
            .get_value("range")?
            .unwrap_or(Value::Empty)
            .as_list()?
            .clone();

        if range.len() != 2 {
            return Err(crate::Error::ExpectedFixedLenList {
                expected_len: 2,
                actual: Value::List(range),
            });
        }

        let range_start = range[0].as_string()?;
        let range_end = range[1].as_string()?;

        let script = format!(
            "sudo parted {path} mklabel {label} mkpart {name} {filesystem} {range_start} {range_end}"
        );

        Command::new("fish")
            .arg("-c")
            .arg(&script)
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}
