use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    AuthStatus, Comment, CreateEpicInput, CreateIssueInput, CreatePullRequestInput, Issue,
    LinkIssuesInput, ListRecentReposInput, PullRequest, PushBranchInput, RepoDetails, RepoRef,
    RepoSummary, UpdateIssueInput, Workspace,
};

// Re-export provider traits from core
pub use ws_core::providers::{CodeProvider, DocProvider, IssueProvider};

// ==========================================
// AI Command: provider.code.check_auth
// ==========================================
pub struct ProviderCodeCheckAuthCommand;

#[async_trait]
impl AiCommand for ProviderCodeCheckAuthCommand {
    const ID: &'static str = "provider.code.check_auth";
    const DESCRIPTION: &'static str =
        "Check authentication status with the configured code provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx
            .code_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No code provider configured".to_string()))?;
        code_provider.check_auth().await
    }
}

// ==========================================
// AI Command: provider.code.list_recent_repos
// ==========================================
pub struct ProviderCodeListRecentReposCommand;

#[async_trait]
impl AiCommand for ProviderCodeListRecentReposCommand {
    const ID: &'static str = "provider.code.list_recent_repos";
    const DESCRIPTION: &'static str = "List recently updated repositories from the code provider.";
    type Input = ListRecentReposInput;
    type Output = Vec<RepoSummary>;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx
            .code_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No code provider configured".to_string()))?;
        code_provider.list_recent_repos(input).await
    }
}

// ==========================================
// AI Command: provider.code.get_repo
// ==========================================
pub struct ProviderCodeGetRepoCommand;

#[async_trait]
impl AiCommand for ProviderCodeGetRepoCommand {
    const ID: &'static str = "provider.code.get_repo";
    const DESCRIPTION: &'static str = "Retrieve repository metadata details.";
    type Input = RepoRef;
    type Output = RepoDetails;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx
            .code_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No code provider configured".to_string()))?;
        code_provider.get_repo(input).await
    }
}

// ==========================================
// AI Command: provider.issue.check_auth
// ==========================================
pub struct ProviderIssueCheckAuthCommand;

#[async_trait]
impl AiCommand for ProviderIssueCheckAuthCommand {
    const ID: &'static str = "provider.issue.check_auth";
    const DESCRIPTION: &'static str =
        "Check authentication status with the configured issue provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.check_auth().await
    }
}

// ==========================================
// AI Command: provider.issue.get_issue
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderIssueGetInput {
    pub key: String,
}

pub struct ProviderIssueGetIssueCommand;

#[async_trait]
impl AiCommand for ProviderIssueGetIssueCommand {
    const ID: &'static str = "provider.issue.get_issue";
    const DESCRIPTION: &'static str = "Retrieve issue tracking details by key.";
    type Input = ProviderIssueGetInput;
    type Output = Issue;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.get_issue(&input.key).await
    }
}

// ==========================================
// AI Command: provider.issue.create_epic
// ==========================================
pub struct ProviderIssueCreateEpicCommand;

#[async_trait]
impl AiCommand for ProviderIssueCreateEpicCommand {
    const ID: &'static str = "provider.issue.create_epic";
    const DESCRIPTION: &'static str = "Create a new Epic in the issue tracker.";
    type Input = CreateEpicInput;
    type Output = Issue;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.create_epic(input).await
    }
}

// ==========================================
// AI Command: provider.issue.create_issue
// ==========================================
pub struct ProviderIssueCreateIssueCommand;

#[async_trait]
impl AiCommand for ProviderIssueCreateIssueCommand {
    const ID: &'static str = "provider.issue.create_issue";
    const DESCRIPTION: &'static str = "Create a new Issue task in the issue tracker.";
    type Input = CreateIssueInput;
    type Output = Issue;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.create_issue(input).await
    }
}

// ==========================================
// AI Command: provider.issue.link
// ==========================================
pub struct ProviderIssueLinkCommand;

#[async_trait]
impl AiCommand for ProviderIssueLinkCommand {
    const ID: &'static str = "provider.issue.link";
    const DESCRIPTION: &'static str = "Link two issues together in the issue tracker.";
    type Input = LinkIssuesInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.link_issues(input).await?;
        Ok(StatusOutput {
            success: true,
            message: "Issues linked successfully.".to_string(),
        })
    }
}

// ==========================================
// AI Command: provider.issue.comment
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderIssueCommentInput {
    pub key: String,
    pub body: String,
}

pub struct ProviderIssueCommentCommand;

#[async_trait]
impl AiCommand for ProviderIssueCommentCommand {
    const ID: &'static str = "provider.issue.comment";
    const DESCRIPTION: &'static str = "Add a comment to an issue in the tracker.";
    type Input = ProviderIssueCommentInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx
            .issue_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No issue provider configured".to_string()))?;
        issue_provider.add_comment(&input.key, &input.body).await?;
        Ok(StatusOutput {
            success: true,
            message: "Comment added successfully.".to_string(),
        })
    }
}

// ==========================================
// AI Command: provider.doc.check_auth
// ==========================================
pub struct ProviderDocCheckAuthCommand;

#[async_trait]
impl AiCommand for ProviderDocCheckAuthCommand {
    const ID: &'static str = "provider.doc.check_auth";
    const DESCRIPTION: &'static str =
        "Check authentication status with the configured doc provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx
            .doc_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No doc provider configured".to_string()))?;
        doc_provider.check_auth().await
    }
}

// ==========================================
// AI Command: provider.doc.get_page
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderDocGetPageInput {
    pub space: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderDocGetPageOutput {
    pub content: String,
}

pub struct ProviderDocGetPageCommand;

#[async_trait]
impl AiCommand for ProviderDocGetPageCommand {
    const ID: &'static str = "provider.doc.get_page";
    const DESCRIPTION: &'static str = "Retrieve page content from the doc provider knowledge base.";
    type Input = ProviderDocGetPageInput;
    type Output = ProviderDocGetPageOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx
            .doc_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No doc provider configured".to_string()))?;
        let content = doc_provider.get_page(&input.space, &input.title).await?;
        Ok(ProviderDocGetPageOutput { content })
    }
}

// ==========================================
// AI Command: provider.doc.create_page
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderDocCreatePageInput {
    pub space: String,
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderDocCreatePageOutput {
    pub page_id: String,
}

pub struct ProviderDocCreatePageCommand;

#[async_trait]
impl AiCommand for ProviderDocCreatePageCommand {
    const ID: &'static str = "provider.doc.create_page";
    const DESCRIPTION: &'static str = "Create a new documentation page in the doc provider.";
    type Input = ProviderDocCreatePageInput;
    type Output = ProviderDocCreatePageOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx
            .doc_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No doc provider configured".to_string()))?;
        let page_id = doc_provider
            .create_page(&input.space, &input.title, &input.body)
            .await?;
        Ok(ProviderDocCreatePageOutput { page_id })
    }
}

// ==========================================
// AI Command: provider.doc.update_page
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderDocUpdatePageInput {
    pub page_id: String,
    pub title: String,
    pub body: String,
}

pub struct ProviderDocUpdatePageCommand;

#[async_trait]
impl AiCommand for ProviderDocUpdatePageCommand {
    const ID: &'static str = "provider.doc.update_page";
    const DESCRIPTION: &'static str = "Update an existing page in the doc provider.";
    type Input = ProviderDocUpdatePageInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx
            .doc_provider
            .as_ref()
            .ok_or_else(|| WorkspaceError::Config("No doc provider configured".to_string()))?;
        doc_provider
            .update_page(&input.page_id, &input.title, &input.body)
            .await?;
        Ok(StatusOutput {
            success: true,
            message: "Documentation page updated successfully.".to_string(),
        })
    }
}

// ==========================================
// AI Command: provider.config.get_instructions
// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderConfigGetInstructionsInput {
    pub provider_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderConfigGetInstructionsOutput {
    pub provider_id: String,
    pub instructions: Option<String>,
}

pub struct ProviderConfigGetInstructionsCommand;

#[async_trait]
impl AiCommand for ProviderConfigGetInstructionsCommand {
    const ID: &'static str = "provider.config.get_instructions";
    const DESCRIPTION: &'static str =
        "Retrieve custom instruction guidelines (e.g. AGENT.md equivalent) for a provider.";
    type Input = ProviderConfigGetInstructionsInput;
    type Output = ProviderConfigGetInstructionsOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let root = &ctx.workspace_root;
        let paths_to_try = vec![
            root.join("config")
                .join("providers")
                .join(format!("{}.md", input.provider_id)),
            root.join(".ws")
                .join("providers")
                .join(format!("{}.md", input.provider_id)),
        ];

        for path in paths_to_try {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    return Ok(ProviderConfigGetInstructionsOutput {
                        provider_id: input.provider_id,
                        instructions: Some(content),
                    });
                }
            }
        }

        Ok(ProviderConfigGetInstructionsOutput {
            provider_id: input.provider_id,
            instructions: None,
        })
    }
}

// ==========================================
// AI Command: provider.config.sync_instructions
// ==========================================

/// Regenerate the ws-managed **company `AGENTS.md`** at the workspace root to reflect
/// the current `workflows/*.md`, the current catalog, and company practices (incl. the
/// repo-init healthcheck surface). Counterpart to read-only `provider.config.get_instructions`.
///
/// Non-destructive to third-party integration blocks: any `<!-- BEGIN ... -->` /
/// `<!-- END ... -->` block already present (e.g. the Beads tracker section) is preserved
/// verbatim and re-appended.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderConfigSyncInstructionsInput {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ProviderConfigSyncInstructionsOutput {
    pub success: bool,
    pub message: String,
    pub path: String,
}

pub struct ProviderConfigSyncInstructionsCommand;

#[async_trait]
impl AiCommand for ProviderConfigSyncInstructionsCommand {
    const ID: &'static str = "provider.config.sync_instructions";
    const DESCRIPTION: &'static str = "Regenerate the ws-managed company AGENTS.md (workflows + catalog + practices). Non-destructive to integration blocks.";
    type Input = ProviderConfigSyncInstructionsInput;
    type Output = ProviderConfigSyncInstructionsOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let root = &ctx.workspace_root;
        let body = generate_company_agents_md(root);

        // Preserve third-party integration blocks (anything between explicit
        // BEGIN/END markers) found in the existing file, so regeneration is non-destructive
        // to e.g. the Beads tracker section injected by `bd init`.
        let preserved = extract_integration_blocks(&root.join("AGENTS.md"));
        let mut content = body;
        if !preserved.trim().is_empty() {
            content.push_str("\n\n");
            content.push_str(&preserved);
            if !preserved.ends_with('\n') {
                content.push('\n');
            }
        }

        let path = root.join("AGENTS.md");
        fs::write(&path, &content)?;

        Ok(ProviderConfigSyncInstructionsOutput {
            success: true,
            message: format!(
                "Regenerated company AGENTS.md ({} bytes) at {}",
                content.len(),
                path.display()
            ),
            path: path.display().to_string(),
        })
    }
}

fn generate_company_agents_md(root: &Path) -> String {
    let mut s = String::new();
    s.push_str("# AI Workspace Rules for Autonomous Coding Agents\n\n");
    s.push_str("Welcome Agent! This document defines your behavioral boundaries, rules, and guidelines when working in this multi-repo workspace. **This file is ws-managed** — regenerate it with `ws ai run provider.config.sync_instructions --input '{}'`. Do not hand-edit the ws-managed sections; append custom integration blocks between dedicated BEGIN/END markers instead.\n\n");

    s.push_str("## Core Rules\n\n");
    s.push_str("1. **Use the JSON API:** Always prefer running commands via the `ws ai run <command_id> --input <file>` interface rather than executing manual git or file operations, unless specifically instructed. This ensures workspace lockfiles (`locks.yaml`) and workspace configs (`workspace.yaml`) remain in sync.\n");
    s.push_str("2. **Grow the Catalog Incrementally:** Do not edit global catalog configurations. If you introduce or work with a new service or repository, create a separate YAML file for it under `catalog/services/<repo-name>.yaml`.\n");
    s.push_str("3. **Keep Workspaces Disposable:** Local epic workspaces created under `workspaces/` are temporary, generated environments. Do not store permanent configuration, logs, or uncommitted work outside of git repositories or the `.ws` config directory.\n");
    s.push_str("4. **Preserve Baseline Commits:** Always reference `baseline_commit` inside `locks.yaml` when analyzing changes or creating pull requests.\n");
    s.push_str("5. **Always Validate:** Before committing new catalogs, run `ws ai run catalog.validate --input '{}'` to ensure parsing schemas are fully respected.\n\n");

    s.push_str("## Workflow Rules\n\n");
    s.push_str(
        "Refer to the workflows documented under `workflows/` for step-by-step processes:\n",
    );
    let workflows_dir = root.join("workflows");
    let mut workflows: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&workflows_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("md") {
                if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                    workflows.push(stem.to_string());
                }
            }
        }
    }
    workflows.sort();
    for stem in &workflows {
        s.push_str(&format!(
            "  - [workflows/{}.md](workflows/{}.md)\n",
            stem, stem
        ));
    }
    s.push('\n');

    s.push_str("## Repo-Init Healthcheck (ws / harness / customer split)\n\n");
    s.push_str("When a repo is initialized/added to the catalog, the harness works it through the locked 10-point checklist in [workflows/repo-init.md](workflows/repo-init.md). The split is invariant:\n\n");
    s.push_str("- **`ws`** = the deterministic oracle: reads (`repo.healthcheck`), executes a single command (`repo.run`), emits specs (`repo.fix_loop.prompt`), validates writes (`catalog.service.update`, strict). **Never fixes, never judges, never owns an LLM.**\n");
    s.push_str("- **Customer's harness** = the agent. Runs setup, picks the harness+provider for #1, performs the 2-subagent fix-loop ([workflows/repo-verify.md](workflows/repo-verify.md)), authors `verify_run`/`agent_verify` scripts, calls `catalog.service.update`.\n");
    s.push_str("- **Customer (human)** = fills gaps the harness can't (probe-script content, deploy envs in [workflows/deploy.md](workflows/deploy.md), decides integration-test applicability).\n\n");
    s.push_str("Relevant `ws ai run` commands: `repo.healthcheck`, `repo.run`, `repo.verify`, `repo.fix_loop.prompt`, `repo.understand.verify`, `catalog.service.update`. Run `ws ai manifest` for the full list and `ws ai schema <command> input` for input shapes.\n\n");

    s.push_str("## Catalog Snapshot\n\n");
    let svc = count_yaml(&root.join("catalog/services"));
    let prod = count_yaml(&root.join("catalog/products"));
    let team = count_yaml(&root.join("catalog/teams"));
    s.push_str(&format!(
        "- Services: {} registered under `catalog/services/`\n",
        svc
    ));
    s.push_str(&format!(
        "- Products: {} registered under `catalog/products/`\n",
        prod
    ));
    s.push_str(&format!(
        "- Teams: {} registered under `catalog/teams/`\n",
        team
    ));
    s.push_str("\n");

    s.push_str("## Product Knowledge Base\n\n");
    s.push_str("The company product side maintains an LLM-maintained wiki under `catalog/knowledge/`, following the [LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f). It is a persistent, compounding artifact: source documents are compiled once into interlinked markdown and kept current over time.\n\n");
    s.push_str("- **Read `catalog/knowledge/SCHEMA.md` before maintaining the wiki** — it is the authoritative contract for structure, conventions, and the ingest/query/update/lint operations.\n");
    s.push_str("- **Raw sources** go in `catalog/knowledge/raw/` (the human drops files here; it is gitignored and never modified by the agent). External Document sources are also supported — by default **Confluence** (see `config/providers/confluence.md` and each product's `knowledge_sources`).\n");
    s.push_str("- **The wiki** (`catalog/knowledge/wiki/`) is agent-owned, version-controlled markdown (`index.md` + `log.md` + topic/entity/source/synthesis pages).\n");
    s.push_str("- When brainstorming an idea, use the wiki as the primary context layer. If a new, unconfirmed fact surfaces, **ask the user to confirm it and provide a source before integrating it** — never inject unconfirmed facts as if they were sourced.\n");
    s.push_str("- This covers the **product side only**. Do not restructure `catalog/teams/` or `catalog/services/` or mirror their content into this wiki.\n");

    s.trim_end().to_string()
}

fn count_yaml(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .map_or(false, |x| x == "yaml" || x == "yml")
                })
                .count()
        })
        .unwrap_or(0)
}

/// Extract and return verbatim any `<!-- BEGIN ... --> ... <!-- END ... -->` blocks
/// from the given file, joined with blank lines. Used to preserve third-party
/// integration sections (e.g. Beads) across regeneration.
fn extract_integration_blocks(path: &Path) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in content.lines() {
        if line.trim_start().starts_with("<!-- BEGIN") {
            current = Some(String::new());
        }
        if let Some(buf) = current.as_mut() {
            buf.push_str(line);
            buf.push('\n');
        }
        if line.trim_start().starts_with("<!-- END") {
            if let Some(b) = current.take() {
                blocks.push(b);
            }
        }
    }
    blocks.join("\n")
}

// ==========================================
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PrCreateInput {
    pub workspace_id: String,
    pub services: Vec<String>,
    pub title: String,
    pub body: String,
    pub draft: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PrCreateOutput {
    pub workspace_id: String,
    pub prs: HashMap<String, String>, // service_id -> pr_url
}

pub struct PrCreateCommand;

#[async_trait]
impl AiCommand for PrCreateCommand {
    const ID: &'static str = "pr.create";
    const DESCRIPTION: &'static str =
        "Create pull requests for changes in the workspace repositories.";
    type Input = PrCreateInput;
    type Output = PrCreateOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for PR creation".to_string())
        })?;

        // Load workspace directly to avoid cyclic dependencies
        let ws_path = ctx
            .workspace_root
            .join("workspaces")
            .join(&input.workspace_id)
            .join("workspace.yaml");

        if !ws_path.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "Workspace {} not found at {}",
                input.workspace_id,
                ws_path.display()
            )));
        }

        let ws_content = fs::read_to_string(ws_path)?;
        let ws: Workspace = serde_yaml::from_str(&ws_content)?;

        let mut prs = HashMap::new();

        for service_id in &input.services {
            if !ws.services.contains(service_id) {
                return Err(WorkspaceError::Validation(format!(
                    "Service {} is not registered in workspace {}",
                    service_id, input.workspace_id
                )));
            }

            let branch_name = if ws.create_branches {
                input.workspace_id.clone()
            } else {
                input.workspace_id.clone()
            };

            // 1. Push branch
            code_provider
                .push_branch(PushBranchInput {
                    epic_key: input.workspace_id.clone(),
                    service_id: service_id.clone(),
                    branch: branch_name.clone(),
                })
                .await?;

            // 2. Create PR
            let pr = code_provider
                .create_pull_request(CreatePullRequestInput {
                    epic_key: input.workspace_id.clone(),
                    service_id: service_id.clone(),
                    branch: branch_name.clone(),
                    title: input.title.clone(),
                    body: input.body.clone(),
                    draft: input.draft,
                })
                .await?;

            prs.insert(service_id.clone(), pr.url.clone());

            // 3. Comment on Jira if available
            if let Some(issue_provider) = &ctx.issue_provider {
                let comment_body = format!(
                    "Created Pull Request for **{}**: {}\n(Branch: `{}`)",
                    service_id, pr.url, branch_name
                );
                let _ = issue_provider
                    .add_comment(&input.workspace_id, &comment_body)
                    .await;
            }
        }

        Ok(PrCreateOutput {
            workspace_id: input.workspace_id,
            prs,
        })
    }
}

// Helper types
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EmptyInput {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StatusOutput {
    pub success: bool,
    pub message: String,
}
