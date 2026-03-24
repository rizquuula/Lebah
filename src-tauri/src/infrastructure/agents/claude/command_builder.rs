use std::process::{Command, Stdio};

use crate::domain::agent::runner::{AgentRunConfig, PermissionMode};

pub struct ClaudeCommandBuilder;

impl ClaudeCommandBuilder {
    /// Build a Command from an AgentRunConfig for the Claude CLI.
    pub fn build(config: &AgentRunConfig) -> Command {
        let binary = config
            .agent_binary
            .as_deref()
            .unwrap_or("claude");

        let mut cmd = Command::new(binary);

        if config.is_follow_up {
            cmd.arg("--continue")
                .arg("--session-id")
                .arg(config.task_id.0.as_str())
                .arg("--fork-session");
        } else {
            cmd.arg("--session-id").arg(config.task_id.0.as_str());
        }

        if let Some(ref project) = config.project_path {
            cmd.current_dir(project.as_str());
        }

        // Apply user env vars before permission mode so Sandbox IS_SANDBOX=1 takes precedence
        cmd.envs(&config.env_vars);

        match config.permission_mode {
            PermissionMode::Plan => {
                cmd.arg("--permission-mode").arg("plan");
            }
            PermissionMode::Sandbox => {
                cmd.env("IS_SANDBOX", "1");
                cmd.arg("--dangerously-skip-permissions");
            }
            PermissionMode::Full => {}
        }

        if let Some(ref wt) = config.worktree {
            cmd.arg("--worktree").arg(wt.as_str());
        }

        if let Some(ref m) = config.model {
            cmd.arg("--model").arg(m);
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--print")
            .arg(&config.prompt);

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::debug!(
            "[claude_cmd] Built command for task {}: binary={} follow_up={} permission={:?} model={:?}",
            config.task_id.0,
            binary,
            config.is_follow_up,
            config.permission_mode,
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
        cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn uses_claude_binary_by_default() {
        let cmd = ClaudeCommandBuilder::build(&base_config());
        assert_eq!(cmd.get_program(), "claude");
    }

    #[test]
    fn uses_custom_binary() {
        let mut cfg = base_config();
        cfg.agent_binary = Some("/custom/claude".to_string());
        let cmd = ClaudeCommandBuilder::build(&cfg);
        assert_eq!(cmd.get_program(), "/custom/claude");
    }

    #[test]
    fn includes_session_id() {
        let cmd = ClaudeCommandBuilder::build(&base_config());
        let a = args(&cmd);
        let idx = a.iter().position(|s| s == "--session-id").expect("--session-id missing");
        assert_eq!(a[idx + 1], "test-id");
    }

    #[test]
    fn follow_up_includes_continue_flag() {
        let mut cfg = base_config();
        cfg.is_follow_up = true;
        let a = args(&ClaudeCommandBuilder::build(&cfg));
        assert!(a.contains(&"--continue".to_string()));
        assert!(a.contains(&"--fork-session".to_string()));
    }

    #[test]
    fn plan_mode_adds_permission_flag() {
        let mut cfg = base_config();
        cfg.permission_mode = PermissionMode::Plan;
        let a = args(&ClaudeCommandBuilder::build(&cfg));
        let idx = a.iter().position(|s| s == "--permission-mode").expect("missing");
        assert_eq!(a[idx + 1], "plan");
    }

    #[test]
    fn sandbox_mode_adds_skip_permissions() {
        let mut cfg = base_config();
        cfg.permission_mode = PermissionMode::Sandbox;
        let a = args(&ClaudeCommandBuilder::build(&cfg));
        assert!(a.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn model_flag_included_when_set() {
        let mut cfg = base_config();
        cfg.model = Some("claude-opus-4-6".to_string());
        let a = args(&ClaudeCommandBuilder::build(&cfg));
        let idx = a.iter().position(|s| s == "--model").expect("--model missing");
        assert_eq!(a[idx + 1], "claude-opus-4-6");
    }

    #[test]
    fn output_format_stream_json_always_present() {
        let a = args(&ClaudeCommandBuilder::build(&base_config()));
        let idx = a.iter().position(|s| s == "--output-format").expect("missing");
        assert_eq!(a[idx + 1], "stream-json");
    }

    #[test]
    fn prompt_is_last_meaningful_arg() {
        let mut cfg = base_config();
        cfg.prompt = "my prompt".to_string();
        let a = args(&ClaudeCommandBuilder::build(&cfg));
        assert_eq!(a.last().unwrap(), "my prompt");
    }
}
