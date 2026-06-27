//! `repo.understand.verify` — the Point #1 test.
//!
//! Asserts three things:
//! 1. `.understand-anything/knowledge-graph.json` exists at the repo root, parses
//!    as JSON, and is non-empty.
//! 2. The GitHub Action run on the onboarding PR completed green.
//! 3. That PR got merged.
//!
//! Credential model: uses the `gh` CLI (consistent with the rest of `ws`),
//! authenticated via the harness environment (`GITHUB_TOKEN` / `gh auth login`).
//! The harness invokes this AFTER the PR's Action run completes.

use std::path::Path;

use async_trait::async_trait;
use duct::cmd;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ws_catalog::get_service;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoUnderstandVerifyInput {
    pub service_id: String,
    /// Absolute or relative path to a working checkout of the repo.
    pub repo_path: String,
    /// The onboarding PR number that adds the workflow + commits the artifact.
    pub pr_number: u64,
    /// Optional explicit workflow run id. If given, its conclusion is checked
    /// directly (more precise than PR-check name heuristics).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoUnderstandVerifyOutput {
    pub service_id: String,
    pub artifact_ok: bool,
    pub workflow_run_green: bool,
    pub pr_merged: bool,
    pub overall: bool,
    pub evidence: String,
}

pub struct RepoUnderstandVerifyCommand;

#[async_trait]
impl AiCommand for RepoUnderstandVerifyCommand {
    const ID: &'static str = "repo.understand.verify";
    const DESCRIPTION: &'static str = "Verify Point #1 (Understand-Anything): artifact present + Action green + PR merged.";
    type Input = RepoUnderstandVerifyInput;
    type Output = RepoUnderstandVerifyOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let service = get_service(&ctx.workspace_root, &input.service_id)?;
        let repo_root = Path::new(&input.repo_path);
        if !repo_root.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "repo_path '{}' does not exist; ws never clones — pass a working checkout.",
                input.repo_path
            )));
        }

        let mut evidence = String::new();

        // (i) Artifact exists + parses + non-empty.
        let artifact_path = repo_root.join(".understand-anything/knowledge-graph.json");
        let artifact_ok = if !artifact_path.exists() {
            evidence.push_str("artifact: not found at .understand-anything/knowledge-graph.json; ");
            false
        } else {
            match std::fs::read_to_string(&artifact_path) {
                Err(e) => {
                    evidence.push_str(&format!("artifact: unreadable: {}; ", e));
                    false
                }
                Ok(content) if content.trim().is_empty() => {
                    evidence.push_str("artifact: present but empty; ");
                    false
                }
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Err(e) => {
                        evidence.push_str(&format!("artifact: invalid JSON: {}; ", e));
                        false
                    }
                    Ok(v) => {
                        let non_empty = match &v {
                            serde_json::Value::Array(a) => !a.is_empty(),
                            serde_json::Value::Object(o) => !o.is_empty(),
                            serde_json::Value::String(s) => !s.is_empty(),
                            _ => true,
                        };
                        if non_empty {
                            evidence.push_str("artifact: ok (parses, non-empty); ");
                            true
                        } else {
                            evidence.push_str("artifact: valid JSON but empty collection; ");
                            false
                        }
                    }
                },
            }
        };

        // (ii) Action run green.
        let workflow_run_green = check_workflow_green(repo_root, input.pr_number, input.run_id, &mut evidence)?;

        // (iii) PR merged.
        let pr_merged = check_pr_merged(repo_root, input.pr_number, &mut evidence)?;

        let overall = artifact_ok && workflow_run_green && pr_merged;
        evidence.push_str(&format!("overall: {}", overall));

        Ok(RepoUnderstandVerifyOutput {
            service_id: service.id,
            artifact_ok,
            workflow_run_green,
            pr_merged,
            overall,
            evidence,
        })
    }
}

fn run_gh(args: &[&str], cwd: &Path) -> Result<String, WorkspaceError> {
    cmd("gh", args)
        .dir(cwd)
        .read()
        .map_err(|e| {
            WorkspaceError::provider(
                "github-gh",
                format!("gh {:?} failed: {} (is gh installed and authenticated?)", args, e),
            )
        })
}

fn check_workflow_green(
    cwd: &Path,
    pr_number: u64,
    run_id: Option<u64>,
    evidence: &mut String,
) -> Result<bool, WorkspaceError> {
    if let Some(rid) = run_id {
        let out = run_gh(
            &["run", "view", &rid.to_string(), "--json", "status,conclusion"],
            cwd,
        )?;
        let v: serde_json::Value = serde_json::from_str(&out).map_err(|e| {
            WorkspaceError::provider("github-gh", format!("gh run view parse error: {}", e))
        })?;
        let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
        let conclusion = v.get("conclusion").and_then(|s| s.as_str()).unwrap_or("");
        let green = status == "completed" && conclusion == "success";
        evidence.push_str(&format!("run {}: status={} conclusion={} -> {}; ", rid, status, conclusion, yn(green)));
        return Ok(green);
    }

    // Fallback: inspect PR checks for an understand-anything-named check.
    let out = run_gh(
        &["pr", "checks", &pr_number.to_string(), "--json", "name,state"],
        cwd,
    )?;
    let v: serde_json::Value = serde_json::from_str(&out).map_err(|e| {
        WorkspaceError::provider("github-gh", format!("gh pr checks parse error: {}", e))
    })?;
    let checks = v.as_array().cloned().unwrap_or_default();
    let mut found = false;
    let mut green = false;
    for chk in checks {
        let name = chk.get("name").and_then(|s| s.as_str()).unwrap_or("");
        if name.to_lowercase().contains("understand") {
            found = true;
            let state = chk.get("state").and_then(|s| s.as_str()).unwrap_or("");
            if state.eq_ignore_ascii_case("success") {
                green = true;
            }
            evidence.push_str(&format!("check '{}' state={}; ", name, state));
        }
    }
    if !found {
        evidence.push_str("no PR check mentioning 'understand' found; ");
    }
    Ok(green)
}

fn check_pr_merged(cwd: &Path, pr_number: u64, evidence: &mut String) -> Result<bool, WorkspaceError> {
    let out = run_gh(
        &["pr", "view", &pr_number.to_string(), "--json", "state,merged"],
        cwd,
    )?;
    let v: serde_json::Value = serde_json::from_str(&out).map_err(|e| {
        WorkspaceError::provider("github-gh", format!("gh pr view parse error: {}", e))
    })?;
    let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("");
    let merged = v.get("merged").and_then(|b| b.as_bool()).unwrap_or(false);
    let ok = merged && state.eq_ignore_ascii_case("closed");
    evidence.push_str(&format!("PR #{}: state={} merged={} -> {}; ", pr_number, state, merged, yn(ok)));
    Ok(ok)
}

fn yn(b: bool) -> &'static str {
    if b { "ok" } else { "no" }
}
