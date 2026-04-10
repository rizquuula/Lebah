pub mod project_commands;
pub mod session_commands;
pub mod task_commands;
pub mod terminal_commands;
pub mod utility_commands;

use std::collections::HashMap;

fn expand_tilde(value: &str) -> String {
    if value == "~" {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| value.to_string())
    } else if value.starts_with("~/") {
        dirs::home_dir()
            .map(|p| format!("{}{}", p.to_string_lossy(), &value[1..]))
            .unwrap_or_else(|| value.to_string())
    } else {
        value.to_string()
    }
}

pub(super) fn expand_env_values(vars: HashMap<String, String>) -> HashMap<String, String> {
    vars.into_iter()
        .map(|(k, v)| (k, expand_tilde(&v)))
        .collect()
}
