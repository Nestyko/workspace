use crate::error::WorkspaceError;
use crate::models::{
    AuthStatus, Comment, CreateEpicInput, CreateIssueInput, CreatePullRequestInput,
    CreateWorktreeInput, EnsureRepoCacheInput, Issue, LinkIssuesInput, ListRecentReposInput,
    PullRequest, PushBranchInput, RepoCache, RepoDetails, RepoRef, RepoSummary, UpdateIssueInput,
    Worktree,
};
use async_trait::async_trait;

#[async_trait]
pub trait IssueProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError>;

    async fn get_issue(&self, key: &str) -> Result<Issue, WorkspaceError>;

    async fn create_epic(&self, input: CreateEpicInput) -> Result<Issue, WorkspaceError>;

    async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue, WorkspaceError>;

    async fn update_issue(&self, key: &str, input: UpdateIssueInput) -> Result<Issue, WorkspaceError>;

    async fn link_issues(&self, input: LinkIssuesInput) -> Result<(), WorkspaceError>;

    async fn add_comment(&self, key: &str, body: &str) -> Result<(), WorkspaceError>;
}

#[async_trait]
pub trait CodeProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError>;

    async fn list_recent_repos(
        &self,
        input: ListRecentReposInput,
    ) -> Result<Vec<RepoSummary>, WorkspaceError>;

    async fn get_repo(&self, input: RepoRef) -> Result<RepoDetails, WorkspaceError>;

    async fn ensure_repo_cache(&self, input: EnsureRepoCacheInput) -> Result<RepoCache, WorkspaceError>;

    async fn create_worktree(&self, input: CreateWorktreeInput) -> Result<Worktree, WorkspaceError>;

    async fn push_branch(&self, input: PushBranchInput) -> Result<(), WorkspaceError>;

    async fn create_pull_request(
        &self,
        input: CreatePullRequestInput,
    ) -> Result<PullRequest, WorkspaceError>;
}

#[async_trait]
pub trait DocProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError>;

    async fn get_page(&self, space: &str, title: &str) -> Result<String, WorkspaceError>;

    async fn create_page(
        &self,
        space: &str,
        title: &str,
        body: &str,
    ) -> Result<String, WorkspaceError>;

    async fn update_page(
        &self,
        page_id: &str,
        title: &str,
        body: &str,
    ) -> Result<(), WorkspaceError>;
}
