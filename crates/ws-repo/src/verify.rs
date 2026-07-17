//! `repo.verify` — deterministic run-all, post-setup only.
//!
//! Order: `install` → `dev`(+`verify_run` smoke) → `test` → `test_integration` →
//! `agent_verify`. Stops at the first failure. **Excludes `deploy`** (locked).
//!
//! No LLM, no harness. This is the post-setup deterministic confirmation that the
//! whole declared toolchain runs end-to-end.

use std::path::Path;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ws_catalog::get_service;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;

use crate::run::{RepoRunCommand, RepoRunInput, RepoRunOutput};

/// Ordered steps for `repo.verify`. `deploy` is intentionally absent.
const VERIFY_ORDER: &[&str] = &[
    "install",
    "dev", // serve-mode; smoke=verify_run
    "test",
    "test_integration",
    "agent_verify",
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoVerifyInput {
    pub service_id: String,
    /// Absolute or relative path to a working checkout of the repo.
    pub repo_path: String,
    /// Per-command timeout in seconds (default 180).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyStepResult {
    pub command: String,
    pub passed: bool,
    pub timed_out: bool,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoVerifyOutput {
    pub service_id: String,
    pub all_passed: bool,
    pub steps_run: usize,
    pub stopped_at: Option<String>,
    pub steps: Vec<VerifyStepResult>,
}

pub struct RepoVerifyCommand;

#[async_trait]
impl AiCommand for RepoVerifyCommand {
    const ID: &'static str = "repo.verify";
    const DESCRIPTION: &'static str =
        "Deterministic run-all (post-setup); excludes deploy; stops at first failure.";
    type Input = RepoVerifyInput;
    type Output = RepoVerifyOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let service = get_service(&ctx.workspace_root, &input.service_id)?;
        let repo_root = Path::new(&input.repo_path);
        if !repo_root.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "repo_path '{}' does not exist; ws never clones — pass a working checkout.",
                input.repo_path
            )));
        }

        let run_cmd = RepoRunCommand;
        let mut steps: Vec<VerifyStepResult> = Vec::new();
        let mut stopped_at: Option<String> = None;

        for key in VERIFY_ORDER {
            // Skip integration tests if not declared (optional, never blocking).
            if *key == "test_integration" && !service.commands.contains_key(*key) {
                continue;
            }
            // Skip any undeclared command.
            if !service.commands.contains_key(*key) {
                // install/dev/test/agent_verify are expected post-setup; if undeclared,
                // treat as an immediate stop (a real gap).
                stopped_at = Some(format!("{} (not declared)", key));
                break;
            }

            let res: RepoRunOutput = run_cmd
                .run(
                    ctx.clone(),
                    RepoRunInput {
                        service_id: input.service_id.clone(),
                        repo_path: input.repo_path.clone(),
                        command: key.to_string(),
                        timeout: input.timeout,
                    },
                )
                .await?;

            let passed = res.smoke_passed && !res.timed_out;
            let step = VerifyStepResult {
                command: res.command,
                passed,
                timed_out: res.timed_out,
                exit_code: res.exit_code,
                stdout_tail: res.stdout_tail,
                stderr_tail: res.stderr_tail,
                duration_secs: res.duration_secs,
            };
            let failed = !passed;
            steps.push(step);
            if failed {
                stopped_at = Some(key.to_string());
                break;
            }
        }

        let all_passed = stopped_at.is_none();
        Ok(RepoVerifyOutput {
            service_id: service.id,
            all_passed,
            steps_run: steps.len(),
            stopped_at,
            steps,
        })
    }
}
