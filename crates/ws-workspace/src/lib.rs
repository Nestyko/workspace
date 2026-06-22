use duct::cmd;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use ws_core::error::WorkspaceError;
use ws_core::models::{
    CreateWorktreeInput, EnsureRepoCacheInput, LockedRepo, ServiceCatalog, Workspace, WorkspaceLock,
};
use ws_core::providers::CodeProvider;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub fn get_workspace_dir(root: &Path, epic_key: &str) -> std::path::PathBuf {
    root.join("workspaces").join(epic_key)
}

pub fn load_workspace(root: &Path, epic_key: &str) -> Result<Workspace, WorkspaceError> {
    let dir = get_workspace_dir(root, epic_key);
    let path = dir.join("workspace.yaml");
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Workspace for epic {} not found at {}",
            epic_key,
            path.display()
        )));
    }
    let content = fs::read_to_string(path)?;
    let ws: Workspace = serde_yaml::from_str(&content)?;
    Ok(ws)
}

pub fn save_workspace(root: &Path, epic_key: &str, ws: &Workspace) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, epic_key);
    fs::create_dir_all(&dir)?;
    let path = dir.join("workspace.yaml");
    let content = serde_yaml::to_string(ws)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_workspace_lock(root: &Path, epic_key: &str) -> Result<WorkspaceLock, WorkspaceError> {
    let dir = get_workspace_dir(root, epic_key);
    let path = dir.join("locks.yaml");
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Locks for epic {} not found at {}",
            epic_key,
            path.display()
        )));
    }
    let content = fs::read_to_string(path)?;
    let lock: WorkspaceLock = serde_yaml::from_str(&content)?;
    Ok(lock)
}

pub fn save_workspace_lock(
    root: &Path,
    epic_key: &str,
    lock: &WorkspaceLock,
) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, epic_key);
    fs::create_dir_all(&dir)?;
    let path = dir.join("locks.yaml");
    let content = serde_yaml::to_string(lock)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn generate_code_workspace(
    root: &Path,
    epic_key: &str,
    services: &[String],
) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, epic_key);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.code-workspace", epic_key));

    let mut folders = vec![serde_json::json!({
        "name": "control",
        "path": "../.."
    })];

    for s in services {
        folders.push(serde_json::json!({
            "name": s,
            "path": format!("repos/{}", s)
        }));
    }

    let workspace_json = serde_json::json!({
        "folders": folders,
        "settings": {
            "files.exclude": {
                "**/.cache": true,
                "**/.ws": true
            }
        }
    });

    let content = serde_json::to_string_pretty(&workspace_json)?;
    fs::write(path, content)?;
    Ok(())
}

pub async fn create_epic_workspace(
    root: &Path,
    code_provider: &dyn CodeProvider,
    epic_key: &str,
    services: Vec<ServiceCatalog>,
    base_branch: &str,
    create_branches: bool,
    editor: &str,
) -> Result<Workspace, WorkspaceError> {
    info!(
        "Creating workspace for epic {} with {} services",
        epic_key,
        services.len()
    );

    let ws_dir = get_workspace_dir(root, epic_key);
    fs::create_dir_all(&ws_dir)?;

    let mut locked_repos = HashMap::new();
    let service_ids: Vec<String> = services.iter().map(|s| s.id.clone()).collect();

    for service in &services {
        info!("Setting up service '{}'...", service.id);

        // 1. Ensure cache exists
        let cache = code_provider
            .ensure_repo_cache(EnsureRepoCacheInput {
                owner: service.repo.owner.clone(),
                name: service.repo.name.clone(),
                url: service.repo.url.clone(),
            })
            .await?;

        // 2. Create worktree
        let branch_name = if create_branches {
            epic_key.to_string()
        } else {
            service.repo.default_branch.clone()
        };

        let worktree = code_provider
            .create_worktree(CreateWorktreeInput {
                owner: service.repo.owner.clone(),
                name: service.repo.name.clone(),
                url: service.repo.url.clone(),
                epic_key: epic_key.to_string(),
                service_id: service.id.clone(),
                base_branch: base_branch.to_string(),
                branch: branch_name.clone(),
            })
            .await?;

        // Get baseline commit from the worktree
        let baseline_commit = cmd("git", &["rev-parse", "HEAD"])
            .dir(&worktree.path)
            .read()
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        locked_repos.insert(
            service.id.clone(),
            LockedRepo {
                provider: service.repo.provider.clone(),
                owner: service.repo.owner.clone(),
                name: service.repo.name.clone(),
                default_branch: service.repo.default_branch.clone(),
                baseline_commit,
            },
        );
    }

    let ws = Workspace {
        id: epic_key.to_string(),
        services: service_ids.clone(),
        base_branch: base_branch.to_string(),
        create_branches,
        editor: editor.to_string(),
    };

    let lock = WorkspaceLock {
        id: epic_key.to_string(),
        repos: locked_repos,
    };

    save_workspace(root, epic_key, &ws)?;
    save_workspace_lock(root, epic_key, &lock)?;
    generate_code_workspace(root, epic_key, &service_ids)?;

    Ok(ws)
}

pub async fn add_service_to_epic_workspace(
    root: &Path,
    code_provider: &dyn CodeProvider,
    epic_key: &str,
    service: ServiceCatalog,
) -> Result<(), WorkspaceError> {
    let mut ws = load_workspace(root, epic_key)?;
    let mut lock = load_workspace_lock(root, epic_key)?;

    if ws.services.contains(&service.id) {
        warn!(
            "Service {} is already in workspace {}",
            service.id, epic_key
        );
        return Ok(());
    }

    info!("Adding service '{}' to workspace {}...", service.id, epic_key);

    // 1. Ensure cache
    code_provider
        .ensure_repo_cache(EnsureRepoCacheInput {
            owner: service.repo.owner.clone(),
            name: service.repo.name.clone(),
            url: service.repo.url.clone(),
        })
        .await?;

    // 2. Create worktree
    let branch_name = if ws.create_branches {
        epic_key.to_string()
    } else {
        service.repo.default_branch.clone()
    };

    let worktree = code_provider
        .create_worktree(CreateWorktreeInput {
            owner: service.repo.owner.clone(),
            name: service.repo.name.clone(),
            url: service.repo.url.clone(),
            epic_key: epic_key.to_string(),
            service_id: service.id.clone(),
            base_branch: ws.base_branch.clone(),
            branch: branch_name,
        })
        .await?;

    let baseline_commit = cmd("git", &["rev-parse", "HEAD"])
        .dir(&worktree.path)
        .read()
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    ws.services.push(service.id.clone());
    lock.repos.insert(
        service.id.clone(),
        LockedRepo {
            provider: service.repo.provider.clone(),
            owner: service.repo.owner.clone(),
            name: service.repo.name.clone(),
            default_branch: service.repo.default_branch.clone(),
            baseline_commit,
        },
    );

    save_workspace(root, epic_key, &ws)?;
    save_workspace_lock(root, epic_key, &lock)?;
    generate_code_workspace(root, epic_key, &ws.services)?;

    Ok(())
}

// ==========================================
// AI Command Implementations
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceCreateInput {
    pub epic_key: String,
    pub services: Vec<String>,
    pub base_branch: Option<String>,
    pub create_branches: Option<bool>,
    pub editor: Option<String>,
}

pub struct WorkspaceCreateCommand;

#[async_trait]
impl AiCommand for WorkspaceCreateCommand {
    const ID: &'static str = "workspace.create";
    const DESCRIPTION: &'static str = "Create a per-epic multi-repo workspace.";
    type Input = WorkspaceCreateInput;
    type Output = Workspace;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for workspace creation".to_string())
        })?;

        let mut services = Vec::new();
        for sid in &input.services {
            let svc = ws_catalog::get_service(&ctx.workspace_root, sid)?;
            services.push(svc);
        }

        let base_branch = input.base_branch.unwrap_or_else(|| "main".to_string());
        let create_branches = input.create_branches.unwrap_or(true);
        let editor = input.editor.unwrap_or_else(|| ctx.config.editor.default.clone());

        create_epic_workspace(
            &ctx.workspace_root,
            code_provider.as_ref(),
            &input.epic_key,
            services,
            &base_branch,
            create_branches,
            &editor,
        )
        .await
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAddServiceInput {
    pub epic_key: String,
    pub service_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StatusOutput {
    pub success: bool,
    pub message: String,
}

pub struct WorkspaceAddServiceCommand;

#[async_trait]
impl AiCommand for WorkspaceAddServiceCommand {
    const ID: &'static str = "workspace.add_service";
    const DESCRIPTION: &'static str = "Add a service to an active implementation workspace.";
    type Input = WorkspaceAddServiceInput;
    type Output = StatusOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for adding a service".to_string())
        })?;

        let service = ws_catalog::get_service(&ctx.workspace_root, &input.service_id)?;

        add_service_to_epic_workspace(
            &ctx.workspace_root,
            code_provider.as_ref(),
            &input.epic_key,
            service,
        )
        .await?;

        Ok(StatusOutput {
            success: true,
            message: format!("Service {} added to workspace {}.", input.service_id, input.epic_key),
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceGetInput {
    pub epic_key: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RepoStatus {
    pub service_id: String,
    pub branch: String,
    pub current_commit: String,
    pub baseline_commit: String,
    pub has_changes: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceStatusOutput {
    pub epic_key: String,
    pub services: Vec<String>,
    pub base_branch: String,
    pub create_branches: bool,
    pub editor: String,
    pub repo_statuses: HashMap<String, RepoStatus>,
}

pub struct WorkspaceStatusCommand;

#[async_trait]
impl AiCommand for WorkspaceStatusCommand {
    const ID: &'static str = "workspace.status";
    const DESCRIPTION: &'static str = "Show status of active workspace and repositories.";
    type Input = WorkspaceGetInput;
    type Output = WorkspaceStatusOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let ws = load_workspace(&ctx.workspace_root, &input.epic_key)?;
        let lock = load_workspace_lock(&ctx.workspace_root, &input.epic_key)?;

        let mut repo_statuses = HashMap::new();

        for service_id in &ws.services {
            let worktree_dir = get_workspace_dir(&ctx.workspace_root, &input.epic_key)
                .join("repos")
                .join(service_id);

            let mut branch = "unknown".to_string();
            let mut current_commit = "unknown".to_string();
            let mut has_changes = false;

            if worktree_dir.exists() {
                // Get branch name
                if let Ok(b) = cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"]).dir(&worktree_dir).read() {
                    branch = b.trim().to_string();
                }
                // Get current commit
                if let Ok(c) = cmd("git", &["rev-parse", "HEAD"]).dir(&worktree_dir).read() {
                    current_commit = c.trim().to_string();
                }
                // Check local changes
                if let Ok(status) = cmd("git", &["status", "--porcelain"]).dir(&worktree_dir).read() {
                    has_changes = !status.trim().is_empty();
                }
            }

            let baseline_commit = lock.repos.get(service_id)
                .map(|r| r.baseline_commit.clone())
                .unwrap_or_else(|| "unknown".to_string());

            repo_statuses.insert(service_id.clone(), RepoStatus {
                service_id: service_id.clone(),
                branch,
                current_commit,
                baseline_commit,
                has_changes,
            });
        }

        Ok(WorkspaceStatusOutput {
            epic_key: ws.id,
            services: ws.services,
            base_branch: ws.base_branch,
            create_branches: ws.create_branches,
            editor: ws.editor,
            repo_statuses,
        })
    }
}

pub struct WorkspaceLockCommand;

#[async_trait]
impl AiCommand for WorkspaceLockCommand {
    const ID: &'static str = "workspace.lock";
    const DESCRIPTION: &'static str = "Retrieve workspace lockfile details.";
    type Input = WorkspaceGetInput;
    type Output = WorkspaceLock;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        load_workspace_lock(&ctx.workspace_root, &input.epic_key)
    }
}

pub struct WorkspaceGenerateEditorFilesCommand;

#[async_trait]
impl AiCommand for WorkspaceGenerateEditorFilesCommand {
    const ID: &'static str = "workspace.generate_editor_files";
    const DESCRIPTION: &'static str = "Regenerate editor-specific workspace configurations.";
    type Input = WorkspaceGetInput;
    type Output = StatusOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let ws = load_workspace(&ctx.workspace_root, &input.epic_key)?;
        generate_code_workspace(&ctx.workspace_root, &input.epic_key, &ws.services)?;
        Ok(StatusOutput {
            success: true,
            message: "Editor workspace files regenerated.".to_string(),
        })
    }
}
