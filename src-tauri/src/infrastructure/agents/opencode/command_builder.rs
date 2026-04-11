use std::process::{Command, Stdio};

use crate::domain::agent::runner::AgentRunConfig;

pub struct OpencodeCommandBuilder;

impl OpencodeCommandBuilder {
    /// Build a Command from an AgentRunConfig for the OpenCode CLI.
    pub fn build(config: &AgentRunConfig) -> Command {
        let binary = config.agent_binary.as_deref().unwrap_or("opencode");

        let mut cmd = Command::new(binary);
        cmd.arg("run");
        cmd.arg("--format").arg("json");

        if config.is_follow_up {
            cmd.arg("--session")
                .arg(config.task_id.0.as_str())
                .arg("--continue");
        }

        if let Some(ref project) = config.project_path {
            cmd.arg("--dir").arg(project.as_str());
            cmd.current_dir(project.as_str());
        }

        // Apply user env vars
        cmd.envs(&config.env_vars);

        if let Some(ref m) = config.model {
            cmd.arg("--model").arg(m);
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        // Message is positional in opencode
        cmd.arg(&config.prompt);

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::debug!(
            "[opencode_cmd] Built command for task {}: binary={} follow_up={} model={:?}",
            config.task_id.0,
            binary,
            config.is_follow_up,
            config.model,
        );

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::agent::runner::{AgentRunConfig, PermissionMode};
    use crate::domain::task::value_objects::TaskId;
    use std::collections::HashMap;

    fn base_config() -> AgentRunConfig {
        AgentRunConfig {
            task_id: TaskId::from_string("test-id".to_string()),
            prompt: "do something".to_string(),
            project_path: None,
            worktree: None,
            model: None,
            permission_mode: PermissionMode::Full,
            extra_args: vec![],
            is_follow_up: false,
            agent_binary: None,
            env_vars: HashMap::new(),
        }
    }

    fn args(cmd: &std::process::Command) -> Vec<String> {
        cmd.get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn uses_opencode_binary_by_default() {
        let cmd = OpencodeCommandBuilder::build(&base_config());
        assert_eq!(cmd.get_program(), "opencode");
    }

    #[test]
    fn uses_custom_binary() {
        let mut cfg = base_config();
        cfg.agent_binary = Some("/custom/opencode".to_string());
        let cmd = OpencodeCommandBuilder::build(&cfg);
        assert_eq!(cmd.get_program(), "/custom/opencode");
    }

    #[test]
    fn starts_with_run_and_format_json() {
        let a = args(&OpencodeCommandBuilder::build(&base_config()));
        assert_eq!(a[0], "run");
        assert_eq!(a[1], "--format");
        assert_eq!(a[2], "json");
    }

    #[test]
    fn follow_up_includes_session_and_continue() {
        let mut cfg = base_config();
        cfg.is_follow_up = true;
        let a = args(&OpencodeCommandBuilder::build(&cfg));
        assert!(a.contains(&"--session".to_string()));
        assert!(a.contains(&"--continue".to_string()));
    }

    #[test]
    fn model_flag_included_when_set() {
        let mut cfg = base_config();
        cfg.model = Some("kilo/anthropic/claude-opus-4.6".to_string());
        let a = args(&OpencodeCommandBuilder::build(&cfg));
        let idx = a.iter().position(|s| s == "--model").expect("--model missing");
        assert_eq!(a[idx + 1], "kilo/anthropic/claude-opus-4.6");
    }

    #[test]
    fn prompt_is_last_arg() {
        let mut cfg = base_config();
        cfg.prompt = "my prompt".to_string();
        let a = args(&OpencodeCommandBuilder::build(&cfg));
        assert_eq!(a.last().unwrap(), "my prompt");
    }

    #[test]
    fn plan_and_sandbox_modes_are_ignored() {
        let mut cfg = base_config();
        cfg.permission_mode = PermissionMode::Plan;
        let a = args(&OpencodeCommandBuilder::build(&cfg));
        assert!(!a.contains(&"--permission-mode".to_string()));

        cfg.permission_mode = PermissionMode::Sandbox;
        let a = args(&OpencodeCommandBuilder::build(&cfg));
        assert!(!a.contains(&"--dangerously-skip-permissions".to_string()));
    }
}
