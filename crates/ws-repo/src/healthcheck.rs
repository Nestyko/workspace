//! `repo.healthcheck` — read-only, 10-point repo-init healthcheck discovery.
//!
//! `ws` is the deterministic oracle: this command NEVER fixes, NEVER scaffolds,
//! NEVER runs commands. It discovers, per checklist point, whether the gap is open
//! or closed, and emits a row the harness can act on. Declaration-only for every
//! point except #1 (file+CI existence) and #2 (structural README), which have
//! filesystem checks. #3 is informational-only (never blocks). #7 is optional
//! (never blocks). #9 and #11 were deliberately de-scoped — no row, no field.
//!
//! See `workflows/repo-init.md` for the per-point remediation templates the
//! harness follows when a row reports `missing` / `not_declared` / `partial`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ws_catalog::get_service;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::DeployConfig;

/// All active checklist point ids (9 and 11 deliberately de-scoped).
pub const ACTIVE_CHECK_IDS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "10"];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Fully satisfied (used by file/structural + declaration gates when present).
    Present,
    /// Required gate not satisfied at all.
    Missing,
    /// Some sub-parts satisfied, others not.
    Partial,
    /// Declaration gate satisfied (field present in catalog).
    Declared,
    /// Declaration gate not satisfied (field absent from catalog).
    NotDeclared,
    /// Explicitly not applicable (unused by active points today; reserved).
    Na,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthcheckRow {
    /// One of "1".."8","10".
    pub check_id: String,
    pub title: String,
    pub status: CheckStatus,
    /// True when an unsatisfied status blocks repo readiness.
    pub blocking: bool,
    /// Human-readable evidence trail (paths, present/absent keys).
    pub evidence: String,
    /// Brief next-step hint for the harness (full templates in workflows/repo-init.md).
    pub run_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct HealthcheckSummary {
    pub total: usize,
    pub present: usize,
    pub missing: usize,
    pub partial: usize,
    pub declared: usize,
    pub not_declared: usize,
    /// Rows that are blocking AND unsatisfied (Present/Declared do not count).
    pub blocking_failures: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoHealthcheckOutput {
    pub service_id: String,
    pub repo_path: String,
    pub rows: Vec<HealthcheckRow>,
    pub summary: HealthcheckSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoHealthcheckInput {
    /// Catalog service id whose repo is being healthchecked.
    pub service_id: String,
    /// Absolute or relative path to a *working checkout* (working tree) of the repo.
    /// `ws` never clones; the harness supplies the path (e.g. a workspace worktree
    /// at `workspaces/<epic>/repos/<service_id>` or any local clone).
    pub repo_path: String,
    /// `"all"` (default) or a single check id like `"1"`, `"5"`, `"10"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
}

pub struct RepoHealthcheckCommand;

#[async_trait]
impl AiCommand for RepoHealthcheckCommand {
    const ID: &'static str = "repo.healthcheck";
    const DESCRIPTION: &'static str = "Read-only 10-point repo-init healthcheck for a service repo (declaration + file-existence).";
    type Input = RepoHealthcheckInput;
    type Output = RepoHealthcheckOutput;

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

        let mut rows = Vec::with_capacity(ACTIVE_CHECK_IDS.len());
        for id in ACTIVE_CHECK_IDS {
            rows.push(check_point(id, &service, repo_root));
        }

        // Optional single-check filter.
        if let Some(filter) = input.check.as_deref() {
            if !filter.eq_ignore_ascii_case("all") {
                rows.retain(|r| r.check_id == filter);
                if rows.is_empty() {
                    return Err(WorkspaceError::Validation(format!(
                        "Unknown check id '{}'. Active ids: {:?}",
                        filter, ACTIVE_CHECK_IDS
                    )));
                }
            }
        }

        let summary = summarize(&rows);

        Ok(RepoHealthcheckOutput {
            service_id: service.id,
            repo_path: fs::canonicalize(repo_root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| input.repo_path.clone()),
            rows,
            summary,
        })
    }
}

fn summarize(rows: &[HealthcheckRow]) -> HealthcheckSummary {
    let mut s = HealthcheckSummary {
        total: rows.len(),
        ..Default::default()
    };
    for r in rows {
        match r.status {
            CheckStatus::Present => s.present += 1,
            CheckStatus::Missing => s.missing += 1,
            CheckStatus::Partial => s.partial += 1,
            CheckStatus::Declared => s.declared += 1,
            CheckStatus::NotDeclared => s.not_declared += 1,
            CheckStatus::Na => {}
        }
        if r.blocking && !matches!(r.status, CheckStatus::Present | CheckStatus::Declared) {
            s.blocking_failures += 1;
        }
    }
    s
}

fn check_point(
    id: &str,
    service: &ws_core::models::ServiceCatalog,
    repo_root: &Path,
) -> HealthcheckRow {
    match id {
        "1" => check_understand_anything(service, repo_root),
        "2" => check_readme(service, repo_root),
        "3" => check_agent_doc(service, repo_root),
        "4" => check_command_declared(
            service,
            "4",
            "Install deps",
            "install",
            true,
            "ws ai run repo.run -- to execute `commands.install` (exit 0 expected).",
        ),
        "5" => check_run_locally(service),
        "6" => check_command_declared(
            service,
            "6",
            "Unit tests",
            "test",
            true,
            "ws ai run repo.run -- to execute `commands.test` (exit 0 expected).",
        ),
        "7" => check_command_declared(
            service,
            "7",
            "Integration tests (optional)",
            "test_integration",
            false,
            "Optional — never blocks. If applicable, ws ai run repo.run -- to execute it.",
        ),
        "8" => check_deploy(service),
        "10" => check_command_declared(
            service,
            "10",
            "Agent-verification",
            "agent_verify",
            true,
            "ws ai run repo.run -- to execute `commands.agent_verify` after a change.",
        ),
        other => HealthcheckRow {
            check_id: other.to_string(),
            title: "Unknown".to_string(),
            status: CheckStatus::Na,
            blocking: false,
            evidence: format!("Unknown check id {}", other),
            run_hint: String::new(),
        },
    }
}

// ---- #1 Understand-Anything (file + CI existence) ----

fn check_understand_anything(
    service: &ws_core::models::ServiceCatalog,
    repo_root: &Path,
) -> HealthcheckRow {
    let enabled = service
        .understand_anything
        .as_ref()
        .map(|c| c.enabled)
        .unwrap_or(false);

    if !enabled {
        return HealthcheckRow {
            check_id: "1".into(),
            title: "Understand-Anything".into(),
            status: CheckStatus::NotDeclared,
            blocking: true,
            evidence: "catalog `understand_anything.enabled` is false / unset.".into(),
            run_hint: "Set understand_anything.enabled via catalog.service.update; commit the artifact + .gitattributes lines + the GitHub Action (see workflows/repo-init.md #1).".into(),
        };
    }

    let gitattributes = repo_root.join(".gitattributes");
    let gitattributes_ok = if let Ok(content) = fs::read_to_string(&gitattributes) {
        content.contains(".understand-anything/knowledge-graph.json")
            && content.contains(".understand-anything/")
    } else {
        false
    };

    let workflows_dir = repo_root.join(".github/workflows");
    let named_workflow_ok = [".yml", ".yaml"].iter().any(|ext| {
        // .with_extension replaces the extension; build the path explicitly instead.
        let mut p = repo_root.join(".github/workflows/understand-anything");
        let mut s = p.into_os_string();
        s.push(ext);
        Path::new(&s).exists()
    });
    let workflow_ok = named_workflow_ok || dir_has_workflow_mention(workflows_dir);

    let artifact_ok = repo_root
        .join(".understand-anything/knowledge-graph.json")
        .exists();

    let satisfied = [gitattributes_ok, workflow_ok, artifact_ok]
        .iter()
        .filter(|x| **x)
        .count();
    let status = if satisfied == 3 {
        CheckStatus::Present
    } else if satisfied == 0 {
        CheckStatus::Missing
    } else {
        CheckStatus::Partial
    };
    let evidence = format!(
        "enabled=true; gitattributes={} workflow={} artifact={}",
        yn(gitattributes_ok),
        yn(workflow_ok),
        yn(artifact_ok)
    );
    HealthcheckRow {
        check_id: "1".into(),
        title: "Understand-Anything".into(),
        status,
        blocking: true,
        evidence,
        run_hint: "Commit .understand-anything/knowledge-graph.json with .gitattributes diff-suppression lines + .github/workflows/understand-anything.yml; verify with ws ai run repo.understand.verify.".into(),
    }
}

fn dir_has_workflow_mention(dir: std::path::PathBuf) -> bool {
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension()
                .and_then(|x| x.to_str())
                .map_or(false, |x| x == "yml" || x == "yaml")
            {
                if let Ok(content) = fs::read_to_string(&p) {
                    if content.contains("understand-anything")
                        || content.contains("understand_anything")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ---- #2 README (structural) ----

fn check_readme(service: &ws_core::models::ServiceCatalog, repo_root: &Path) -> HealthcheckRow {
    // Resolve the README path from the catalog declaration if present, else README.md.
    let declared = service
        .docs
        .iter()
        .find(|d| d.r#type == "readme")
        .map(|d| d.path.clone())
        .unwrap_or_else(|| "README.md".to_string());

    let path = repo_root.join(&declared);
    let exists = path.exists();
    let (status, evidence) = if !exists {
        (
            CheckStatus::Missing,
            format!("file not found at {}", declared),
        )
    } else {
        match fs::read_to_string(&path) {
            Err(e) => (
                CheckStatus::Missing,
                format!("unreadable {}: {}", declared, e),
            ),
            Ok(content) if content.trim().is_empty() => (
                CheckStatus::Partial,
                format!("{} exists but is empty", declared),
            ),
            Ok(content) => {
                let has_heading = content.lines().any(|l| l.trim_start().starts_with("# "));
                if has_heading {
                    (
                        CheckStatus::Present,
                        format!("{} exists, non-empty, ≥1 `#` heading", declared),
                    )
                } else {
                    (
                        CheckStatus::Partial,
                        format!("{} exists but no `#` heading", declared),
                    )
                }
            }
        }
    };
    HealthcheckRow {
        check_id: "2".into(),
        title: "README.md".into(),
        status,
        blocking: true,
        evidence,
        run_hint: "Create README.md at repo root: project name, one-paragraph purpose, install/test/run commands (pull from commands.*), link to CONTRIBUTING.md if present.".into(),
    }
}

// ---- #3 AGENT.md / CLAUDE.md (informational only, never blocks) ----

fn check_agent_doc(service: &ws_core::models::ServiceCatalog, repo_root: &Path) -> HealthcheckRow {
    let agent_docs: Vec<_> = service
        .docs
        .iter()
        .filter(|d| d.r#type == "agent")
        .collect();
    let (status, evidence) = if agent_docs.is_empty() {
        (
            CheckStatus::NotDeclared,
            "no docs entry with type: agent".to_string(),
        )
    } else {
        let paths: Vec<String> = agent_docs.iter().map(|d| d.path.clone()).collect();
        let on_disk: Vec<String> = paths
            .iter()
            .filter(|p| repo_root.join(p).exists())
            .cloned()
            .collect();
        let ev = format!(
            "declared agent docs: [{}]; present on disk: [{}]",
            paths.join(", "),
            on_disk.join(", ")
        );
        (CheckStatus::Declared, ev)
    };
    HealthcheckRow {
        check_id: "3".into(),
        title: "AGENT.md / CLAUDE.md (informational)".into(),
        status,
        blocking: false, // informational only — never blocks
        evidence,
        run_hint: "Optional — no validation. If your repo has an AGENT.md/CLAUDE.md, declare it via docs:[{type:agent, path:...}] in the catalog.".into(),
    }
}

// ---- #4/#6/#7/#10 single-command declaration gates ----

fn check_command_declared(
    service: &ws_core::models::ServiceCatalog,
    id: &str,
    title: &str,
    key: &str,
    blocking: bool,
    run_hint: &str,
) -> HealthcheckRow {
    let (status, evidence) = if service.commands.contains_key(key) {
        (
            CheckStatus::Declared,
            format!(
                "commands.{} = `{}`",
                key,
                service.commands.get(key).unwrap()
            ),
        )
    } else {
        (
            CheckStatus::NotDeclared,
            format!("commands.{} not declared", key),
        )
    };
    HealthcheckRow {
        check_id: id.into(),
        title: title.into(),
        status,
        blocking,
        evidence,
        run_hint: run_hint.into(),
    }
}

// ---- #5 dev / run / verify_run (declaration ×3) ----

fn check_run_locally(service: &ws_core::models::ServiceCatalog) -> HealthcheckRow {
    let keys = ["dev", "run", "verify_run"];
    let present: HashSet<&str> = keys
        .iter()
        .copied()
        .filter(|k| service.commands.contains_key(*k))
        .collect();
    let detail: Vec<String> = keys
        .iter()
        .map(|k| {
            format!(
                "{}={}",
                k,
                service
                    .commands
                    .get(*k)
                    .map(|v| format!("`{}`", v))
                    .unwrap_or_else(|| "(absent)".into())
            )
        })
        .collect();
    let status = if present.len() == 3 {
        CheckStatus::Present
    } else if present.is_empty() {
        CheckStatus::Missing
    } else {
        CheckStatus::Partial
    };
    HealthcheckRow {
        check_id: "5".into(),
        title: "Run locally e2e".into(),
        status,
        blocking: true,
        evidence: detail.join(", "),
        run_hint: "dev = local dev start (may not exit); run = production start; verify_run = one-liner confirming the service came up (e.g. curl localhost:8080/healthz). Record via catalog.service.update.".into(),
    }
}

// ---- #8 deploy (declaration or skip) ----

fn check_deploy(service: &ws_core::models::ServiceCatalog) -> HealthcheckRow {
    let (status, evidence) = match &service.deploy {
        None => (CheckStatus::NotDeclared, "deploy not declared".to_string()),
        Some(DeployConfig::Command(cmd)) => (CheckStatus::Declared, format!("deploy = `{}`", cmd)),
        Some(DeployConfig::Skip { skip, reason }) => (
            CheckStatus::Declared,
            format!(
                "deploy.skip = {} reason = {}",
                skip,
                reason.as_deref().unwrap_or("(none)")
            ),
        ),
    };
    HealthcheckRow {
        check_id: "8".into(),
        title: "Deploy + envs + when".into(),
        status,
        blocking: true,
        evidence,
        run_hint: "Plain command string OR {skip:true, reason}. Envs/when/triggers are company-level in workflows/deploy.md (not a catalog field).".into(),
    }
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}
