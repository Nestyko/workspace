use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

// ==========================================
// Code & Git Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoSummary {
    pub provider: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub description: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoDetails {
    pub summary: RepoSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoCache {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Worktree {
    pub path: String,
    pub service_id: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRecentReposInput {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoRef {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsureRepoCacheInput {
    pub owner: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWorktreeInput {
    pub owner: String,
    pub name: String,
    pub url: String,
    pub epic_key: String,
    pub service_id: String,
    pub base_branch: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushBranchInput {
    pub epic_key: String,
    pub service_id: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePullRequestInput {
    pub epic_key: String,
    pub service_id: String,
    pub branch: String,
    pub title: String,
    pub body: String,
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PullRequest {
    pub number: usize,
    pub url: String,
    pub state: String,
}

// ==========================================
// Catalog Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceCatalog {
    pub id: String,
    pub name: String,
    pub kind: String, // "service"
    pub description: String,
    pub team: String,
    pub products: Vec<String>,
    pub repo: CatalogRepo,
    pub owns: Vec<String>,
    pub likely_relevant_when: Vec<String>,
    pub commands: HashMap<String, String>,
    pub issue_tracking: CatalogIssueTracking,
    pub docs: Vec<CatalogDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CatalogRepo {
    pub provider: String,
    pub owner: String,
    pub name: String,
    pub url: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CatalogIssueTracking {
    pub provider: String,
    pub project: String,
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CatalogDoc {
    pub r#type: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductCatalog {
    pub id: String,
    pub name: String,
    pub kind: String, // "product"
    pub description: String,
    pub agent: ProductAgent,
    pub knowledge_sources: Vec<ProductKnowledgeSource>,
    pub services: ProductServices,
    pub routing_rules: Vec<ProductRoutingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductAgent {
    pub name: String,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductKnowledgeSource {
    pub provider: String,
    pub space: Option<String>,
    pub project: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductServices {
    pub primary: Vec<String>,
    pub related: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductRoutingRule {
    pub when: String,
    pub inspect_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TeamCatalog {
    pub id: String,
    pub name: String,
    pub kind: String, // "team"
    pub description: String,
    pub lead: Option<String>,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeCatalog {
    pub id: String,
    pub name: String,
    pub kind: String, // "knowledge"
    pub description: String,
    pub provider: String,
    pub space: Option<String>,
    pub url: Option<String>,
}

// ==========================================
// Issue / Project Tracking Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub username: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Issue {
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub issue_type: String,
    pub assignee: Option<String>,
    pub project_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Epic {
    pub key: String,
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Comment {
    pub id: Option<String>,
    pub author: Option<String>,
    pub body: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateEpicInput {
    pub project: String,
    pub name: String,
    pub summary: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateIssueInput {
    pub project: String,
    pub summary: String,
    pub description: Option<String>,
    pub issue_type: String,
    pub epic_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateIssueInput {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LinkIssuesInput {
    pub inward_key: String,
    pub outward_key: String,
    pub link_type: String,
}

// ==========================================
// Local Config / State Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub issue_provider: IssueProviderConfig,
    pub code_provider: CodeProviderConfig,
    pub doc_provider: Option<DocProviderConfig>,
    pub editor: EditorConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueProviderConfig {
    pub r#type: String,
    pub base_url: Option<String>,
    pub default_project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeProviderConfig {
    pub r#type: String,
    pub default_owner: Option<String>,
    pub protocol: Option<String>, // "ssh" or "https"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocProviderConfig {
    pub r#type: String,
    pub base_url: Option<String>,
    pub default_space: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub default: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub cache_dir: String,
    pub workspaces_dir: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            issue_provider: IssueProviderConfig {
                r#type: "jira".to_string(),
                base_url: None,
                default_project: None,
            },
            code_provider: CodeProviderConfig {
                r#type: "github-gh".to_string(),
                default_owner: None,
                protocol: Some("ssh".to_string()),
            },
            doc_provider: Some(DocProviderConfig {
                r#type: "confluence".to_string(),
                base_url: None,
                default_space: None,
            }),
            editor: EditorConfig {
                default: "cursor".to_string(),
            },
            paths: PathConfig {
                cache_dir: ".cache/repos".to_string(),
                workspaces_dir: "workspaces".to_string(),
            },
        }
    }
}


// ==========================================
// Workspace Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Workspace {
    pub id: String,
    pub services: Vec<String>,
    pub base_branch: String,
    pub create_branches: bool,
    pub editor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceLock {
    pub id: String,
    pub repos: HashMap<String, LockedRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LockedRepo {
    pub provider: String,
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub baseline_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenWorkspaceInput {
    pub epic_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenServiceInput {
    pub epic_key: String,
    pub service_id: String,
}
