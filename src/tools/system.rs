use sys_info::cpu_speed;

use crate::{Result, Tool, ToolInfo, Value};

pub struct CpuSpeed;

impl Tool for CpuSpeed {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "cpu_speed",
            description: "Return the current processor speed in megahertz.",
            group: "system",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        argument.as_empty()?;

        let speed = cpu_speed().unwrap_or_default() as i64;

        Ok(Value::Integer(speed))
    }
}
