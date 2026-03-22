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

        cmd
    }
}
