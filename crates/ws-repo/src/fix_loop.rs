//! `repo.fix_loop.prompt` — harness-readable spec emitter for the 2-subagent fix-loop.
//!
//! `ws` does NOT run the loop. It emits a deterministic markdown spec the harness
//! executes during setup: for each declared command (excluding `deploy`), a runner
//! subagent calls `repo.run`; on non-zero it spawns an implementor subagent to fix
//! the repo, reports back, and re-runs; capped at N=4 attempts per command.

use std::collections::HashMap;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ws_catalog::get_service;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;

const MAX_ATTEMPTS: u8 = 4;

/// Commands the fix-loop targets, in order. `deploy` is excluded (locked dangerous).
const LOOP_COMMANDS: &[&str] = &[
    "install",
    "dev", // serve-mode; smoke = verify_run
    "test",
    "test_integration",
    "agent_verify",
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoFixLoopPromptInput {
    pub service_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoFixLoopPromptOutput {
    pub service_id: String,
    /// The full markdown spec the harness follows.
    pub spec: String,
}

pub struct RepoFixLoopPromptCommand;

#[async_trait]
impl AiCommand for RepoFixLoopPromptCommand {
    const ID: &'static str = "repo.fix_loop.prompt";
    const DESCRIPTION: &'static str = "Emit a harness-readable markdown spec for the 2-subagent setup fix-loop. ws does NOT run it.";
    type Input = RepoFixLoopPromptInput;
    type Output = RepoFixLoopPromptOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let service = get_service(&ctx.workspace_root, &input.service_id)?;
        let commands: HashMap<&str, &str> = LOOP_COMMANDS
            .iter()
            .filter(|k| service.commands.contains_key(**k))
            .map(|k| (*k, service.commands.get(*k).unwrap().as_str()))
            .collect();

        let mut spec = String::new();
        spec.push_str("# Repo Setup Fix-Loop Spec\n\n");
        spec.push_str(&format!(
            "Service: `{}` (id `{}`)\n\n",
            service.name, service.id
        ));
        spec.push_str("## Roles\n\n");
        spec.push_str("- **Runner subagent** — invokes `ws ai run repo.run` for a single command, reads the structured result, decides pass/fail.\n");
        spec.push_str("- **Implementor subagent** — spawned by the runner on failure to edit the repo (scripts/config/code) so the next run passes. Reports a short diff summary back to the runner.\n\n");
        spec.push_str("## Loop contract (per command)\n\n");
        spec.push_str(&format!("1. Runner calls `ws ai run repo.run --input <json>` with `{}` (run the command for THIS service + repo_path).\n", "{service_id, repo_path, command, timeout}"));
        spec.push_str("2. Pass condition: `smoke_passed == true && timed_out == false`.\n");
        spec.push_str(&format!("3. On failure: runner spawns implementor subagent with the failing command, `stdout_tail`/`stderr_tail`, and the relevant remediation template from `workflows/repo-init.md`. Implementor fixes the repo only (never edits the catalog here — that is a separate `catalog.service.update` call the runner makes after the command passes).\n"));
        spec.push_str(&format!("4. Runner re-runs `repo.run`. Repeat up to **N={} attempts**. If still failing after the cap, the runner reports the gap and halts (do not silently succeed).\n", MAX_ATTEMPTS));
        spec.push_str("5. On pass: runner records the declaration via `ws ai run catalog.service.update` if the command was newly added, then advances to the next command.\n\n");
        spec.push_str("**`deploy` is excluded** from this loop (locked dangerous; envs/when/triggers live in `workflows/deploy.md`).\n\n");
        spec.push_str("## Commands to converge (in order)\n\n");
        spec.push_str("| # | command | declared | mode | pass condition |\n");
        spec.push_str("|---|---|---|---|---|\n");
        for (i, key) in LOOP_COMMANDS.iter().enumerate() {
            let declared = service.commands.contains_key(*key);
            let mode = if *key == "dev" || *key == "run" {
                "serve (poll verify_run)"
            } else {
                "exit 0"
            };
            let val = if declared {
                format!("`{}`", service.commands.get(*key).unwrap())
            } else {
                "— (declare first)".to_string()
            };
            spec.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |\n",
                i + 1,
                key,
                val,
                mode,
                "smoke_passed && !timed_out"
            ));
        }
        spec.push_str("\n**Notes:**\n");
        spec.push_str("- `test_integration` is optional — if absent from the catalog, skip it (the gap is not blocking per the healthcheck).\n");
        spec.push_str("- This spec is a *prompt*: `ws` emits it and stops. The harness owns the actual subagent orchestration and repo mutations.\n");

        Ok(RepoFixLoopPromptOutput {
            service_id: service.id,
            spec,
        })
    }
}
