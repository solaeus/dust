use sysinfo::{RefreshKind, System, SystemExt, UserExt};

use crate::{Result, Tool, ToolInfo, Value, ValueType};

pub struct Users;

impl Tool for Users {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "users",
            description: "Get a list of the system's users.",
            group: "system",
            inputs: vec![ValueType::Empty],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        argument.as_empty()?;

        let users = System::new_with_specifics(RefreshKind::new().with_users_list())
            .users()
            .iter()
            .map(|user| Value::String(user.name().to_string()))
            .collect();

        Ok(Value::List(users))
    }
}
