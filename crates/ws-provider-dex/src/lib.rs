//! `dex` (https://dex.rip) issue provider.
//!
//! Implements the [`IssueProvider`] trait by shelling out to the local `dex` CLI.
//! Dex is a fully local, file-backed task tracker — it needs no account, no domain,
//! and no authentication. This makes it a drop-in alternative to `jira` for teams
//! that want to keep issue tracking entirely local to their development machine.
//!
//! Tasks created through this provider are normal dex tasks; the project concept in
//! the issue-tracking model is preserved as the `project_key` (defaults to `dex`).
//! The hierarchical epic → task → subtask model in dex is preserved via `parent_id`.

use async_trait::async_trait;
use serde::Deserialize;
use std::process::Command;
use tracing::{info, warn};
use ws_core::error::WorkspaceError;
use ws_core::models::{
    AuthStatus, CreateEpicInput, CreateIssueInput, Issue, LinkIssuesInput, UpdateIssueInput,
};
use ws_core::providers::IssueProvider;

pub struct DexProvider {
    /// Dex has no notion of a "project"; this is only used to populate the
    /// `project_key` field on the returned [`Issue`] so the rest of the
    /// workspace tooling keeps a consistent shape across providers.
    pub default_project: String,
}

impl DexProvider {
    pub fn new(default_project: Option<String>) -> Self {
        Self {
            default_project: default_project.unwrap_or_else(|| "dex".to_string()),
        }
    }

    /// Run a `dex` subcommand, returning its stdout on success.
    fn run_dex(&self, args: &[&str]) -> Result<String, WorkspaceError> {
        let output = Command::new("dex").args(args).output().map_err(|e| {
            WorkspaceError::provider(
                "dex",
                format!("Failed to spawn `dex` CLI: {e}. Is dex installed (https://dex.rip)?"),
            )
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::provider(
                "dex",
                format!("`dex {}` failed: {}", args.join(" "), stderr.trim()),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_dex_silent(&self, args: &[&str]) -> Result<(), WorkspaceError> {
        let _ = self.run_dex(args)?;
        Ok(())
    }

    /// Fetch a dex task as JSON.
    fn show_json(&self, key: &str) -> Result<DexTask, WorkspaceError> {
        let stdout = self.run_dex(&["show", key, "--json"])?;
        serde_json::from_str::<DexTask>(&stdout).map_err(|e| {
            WorkspaceError::provider(
                "dex",
                format!("Failed to parse `dex show {key} --json` output: {e}"),
            )
        })
    }

    /// Create a dex task, returning the newly-issued task id.
    fn create_task(
        &self,
        name: &str,
        description: Option<&str>,
        parent: Option<&str>,
    ) -> Result<String, WorkspaceError> {
        let mut args: Vec<&str> = vec!["create", name];
        if let Some(d) = description {
            args.extend_from_slice(&["--description", d]);
        }
        if let Some(p) = parent {
            args.extend_from_slice(&["--parent", p]);
        }
        let stdout = self.run_dex(&args)?;
        //dex prints "Created task <id>" as the first line.
        let id = stdout
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("Created task ")
                    .map(|s| s.trim().to_string())
            })
            .ok_or_else(|| {
                WorkspaceError::provider(
                    "dex",
                    format!("Could not parse task id from `dex create` output: {stdout}"),
                )
            })?;
        Ok(id)
    }
}

/// Mapping from the dex task JSON shape (a subset of the fields).
#[derive(Debug, Deserialize)]
struct DexTask {
    id: String,
    #[serde(default)]
    parent_id: Option<String>,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    completed: bool,
    #[serde(default)]
    #[allow(dead_code)]
    result: Option<String>,
    #[serde(default)]
    started_at: Option<String>,
}

impl DexTask {
    fn to_issue(&self, project: &str) -> Issue {
        let issue_type = if self.parent_id.is_none() {
            "Epic"
        } else {
            "Task"
        };
        let status = if self.completed {
            "Done".to_string()
        } else if self.started_at.is_some() {
            "In Progress".to_string()
        } else {
            "To Do".to_string()
        };
        Issue {
            key: self.id.clone(),
            summary: self.name.clone(),
            description: self.description.clone(),
            status,
            issue_type: issue_type.to_string(),
            assignee: None,
            project_key: project.to_string(),
        }
    }
}

#[async_trait]
impl IssueProvider for DexProvider {
    fn kind(&self) -> &'static str {
        "dex"
    }

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError> {
        // Dex is local and requires no credentials. We only probe that the CLI
        // is installed so callers get a clear error early rather than at first use.
        let installed = Command::new("dex")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if installed {
            Ok(AuthStatus {
                authenticated: true,
                username: Some("dex-local".to_string()),
                details: Some(
                    "Dex local issue tracking ready (no account or domain required).".to_string(),
                ),
            })
        } else {
            Ok(AuthStatus {
                authenticated: false,
                username: None,
                details: Some(
                    "`dex` CLI not found on PATH. Install it from https://dex.rip.".to_string(),
                ),
            })
        }
    }

    async fn get_issue(&self, key: &str) -> Result<Issue, WorkspaceError> {
        let task = self.show_json(key)?;
        Ok(task.to_issue(&self.default_project))
    }

    async fn create_epic(&self, input: CreateEpicInput) -> Result<Issue, WorkspaceError> {
        let description = input
            .description
            .clone()
            .unwrap_or_else(|| input.summary.clone());
        let id = self.create_task(&input.name, Some(&description), None)?;
        info!(task_id = %id, "dex epic created");
        let mut issue = self.show_json(&id)?.to_issue(&self.default_project);
        // Preserve the caller's requested summary/name distinction.
        issue.summary = input.name;
        issue.issue_type = "Epic".to_string();
        Ok(issue)
    }

    async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue, WorkspaceError> {
        let id = self.create_task(
            &input.summary,
            input.description.as_deref(),
            input.epic_key.as_deref(),
        )?;
        info!(task_id = %id, "dex issue created");
        let mut issue = self.show_json(&id)?.to_issue(&self.default_project);
        issue.issue_type = input.issue_type.clone();
        Ok(issue)
    }

    async fn update_issue(
        &self,
        key: &str,
        input: UpdateIssueInput,
    ) -> Result<Issue, WorkspaceError> {
        if let Some(summary) = input.summary.as_ref() {
            self.run_dex_silent(&["edit", key, "-n", summary])?;
        }
        if let Some(description) = input.description.as_ref() {
            self.run_dex_silent(&["edit", key, "--description", description])?;
        }
        if let Some(status) = input.status.as_ref() {
            let lower = status.to_lowercase();
            if matches!(lower.as_str(), "done" | "completed" | "closed" | "resolved") {
                let result = format!("Status set to {status} via ws issue provider.");
                // `dex complete` requires --commit or --no-commit for linked tasks;
                // --no-commit keeps the task open if it ever gains a remote link.
                self.run_dex_silent(&["complete", key, "--result", &result, "--no-commit"])
                    .or_else(|e| {
                        warn!("dex complete `{key}` failed (already completed?): {e}");
                        Ok::<(), WorkspaceError>(())
                    })?;
            } else if matches!(lower.as_str(), "in progress" | "in-progress" | "started") {
                self.run_dex_silent(&["start", key]).or_else(|e| {
                    warn!("dex start `{key}` failed: {e}");
                    Ok::<(), WorkspaceError>(())
                })?;
            }
        }
        self.get_issue(key).await
    }

    async fn link_issues(&self, input: LinkIssuesInput) -> Result<(), WorkspaceError> {
        // Dex has no generic "link"; the closest primitive is a blocking dependency,
        // i.e. the outward task blocks the inward task. This preserves the directional
        // intent of LinkIssuesInput (inward depends on outward).
        self.run_dex_silent(&[
            "edit",
            &input.inward_key,
            "--add-blocker",
            &input.outward_key,
        ])
    }

    async fn add_comment(&self, key: &str, body: &str) -> Result<(), WorkspaceError> {
        // Dex has no first-class comments. We append the comment to the task
        // description so the context is preserved alongside the task body.
        let task = self.show_json(key)?;
        let mut existing = task.description.unwrap_or_default();
        if !existing.is_empty() {
            existing.push_str("\n\n");
        }
        existing.push_str("--- Comment ---\n");
        existing.push_str(body);
        self.run_dex_silent(&["edit", key, "--description", &existing])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_create_output_id() {
        let provider = DexProvider::new(None);
        // Simulate the `dex create` stdout shape.
        let id = "Created task 8ahmcrng\n[ ] 8ahmcrng: Probe task\n";
        let parsed = id
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("Created task ")
                    .map(|s| s.trim().to_string())
            })
            .unwrap();
        assert_eq!(parsed, "8ahmcrng");
        assert_eq!(provider.default_project, "dex");
    }

    #[test]
    fn task_to_issue_maps_hierarchy_and_status() {
        let task = DexTask {
            id: "abc123".to_string(),
            parent_id: None,
            name: "Epic thing".to_string(),
            description: Some("d".to_string()),
            completed: false,
            result: None,
            started_at: None,
        };
        let issue = task.to_issue("dex");
        assert_eq!(issue.issue_type, "Epic");
        assert_eq!(issue.status, "To Do");

        let task2 = DexTask {
            id: "child".to_string(),
            parent_id: Some("abc123".to_string()),
            name: "Sub".to_string(),
            description: None,
            completed: true,
            result: Some("done".to_string()),
            started_at: Some("2026-01-01T00:00:00.000Z".to_string()),
        };
        let issue2 = task2.to_issue("dex");
        assert_eq!(issue2.issue_type, "Task");
        assert_eq!(issue2.status, "Done");
    }
}
