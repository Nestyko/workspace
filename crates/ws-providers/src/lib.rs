use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    AuthStatus, Comment, CreateEpicInput, CreateIssueInput, CreatePullRequestInput,
    Issue, LinkIssuesInput, ListRecentReposInput, PullRequest, PushBranchInput, RepoDetails,
    RepoRef, RepoSummary, UpdateIssueInput, Workspace,
};

// Re-export provider traits from core
pub use ws_core::providers::{CodeProvider, IssueProvider, DocProvider};

// ==========================================
// AI Command: provider.code.check_auth
// ==========================================
pub struct ProviderCodeCheckAuthCommand;

#[async_trait]
impl AiCommand for ProviderCodeCheckAuthCommand {
    const ID: &'static str = "provider.code.check_auth";
    const DESCRIPTION: &'static str = "Check authentication status with the configured code provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(&self, ctx: CommandContext, _input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured".to_string())
        })?;
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
    const DESCRIPTION: &'static str = "Check authentication status with the configured issue provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(&self, ctx: CommandContext, _input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let issue_provider = ctx.issue_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No issue provider configured".to_string())
        })?;
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
    const DESCRIPTION: &'static str = "Check authentication status with the configured doc provider.";
    type Input = crate::EmptyInput;
    type Output = AuthStatus;

    async fn run(&self, ctx: CommandContext, _input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx.doc_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No doc provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx.doc_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No doc provider configured".to_string())
        })?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx.doc_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No doc provider configured".to_string())
        })?;
        let page_id = doc_provider.create_page(&input.space, &input.title, &input.body).await?;
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

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let doc_provider = ctx.doc_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No doc provider configured".to_string())
        })?;
        doc_provider.update_page(&input.page_id, &input.title, &input.body).await?;
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
    const DESCRIPTION: &'static str = "Retrieve custom instruction guidelines (e.g. AGENT.md equivalent) for a provider.";
    type Input = ProviderConfigGetInstructionsInput;
    type Output = ProviderConfigGetInstructionsOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let root = &ctx.workspace_root;
        let paths_to_try = vec![
            root.join("config").join("providers").join(format!("{}.md", input.provider_id)),
            root.join(".ws").join("providers").join(format!("{}.md", input.provider_id)),
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
// AI Command: pr.create
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
    const DESCRIPTION: &'static str = "Create pull requests for changes in the workspace repositories.";
    type Input = PrCreateInput;
    type Output = PrCreateOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for PR creation".to_string())
        })?;

        // Load workspace directly to avoid cyclic dependencies
        let ws_path = ctx.workspace_root
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
                let _ = issue_provider.add_comment(&input.workspace_id, &comment_body).await;
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
