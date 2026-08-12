use async_trait::async_trait;
use duct::cmd;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    CreateWorktreeInput, EnsureRepoCacheInput, LockedRepo, MoveWorktreeInput, ServiceCatalog,
    Workspace, WorkspaceLock, WorkspaceTask,
};
use ws_core::providers::CodeProvider;

// Re-export shared resolution helpers (content-based identity; no manifest).
pub use ws_core::workspaces::{
    get_workspace_dir, list_workspaces, resolve_workspace, slugify, workspace_dir_name,
    WorkspaceOnDisk,
};

// ==========================================
// On-disk load / save (folder-keyed)
// ==========================================

pub fn load_workspace(root: &Path, folder: &str) -> Result<Workspace, WorkspaceError> {
    let path = get_workspace_dir(root, folder).join("workspace.yaml");
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Workspace {} not found at {}",
            folder,
            path.display()
        )));
    }
    let content = fs::read_to_string(path)?;
    let ws: Workspace = serde_yaml::from_str(&content)?;
    Ok(ws)
}

pub fn save_workspace(root: &Path, folder: &str, ws: &Workspace) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, folder);
    fs::create_dir_all(&dir)?;
    let content = serde_yaml::to_string(ws)?;
    fs::write(dir.join("workspace.yaml"), content)?;
    Ok(())
}

pub fn load_workspace_lock(root: &Path, folder: &str) -> Result<WorkspaceLock, WorkspaceError> {
    let path = get_workspace_dir(root, folder).join("locks.yaml");
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Locks for workspace {} not found at {}",
            folder,
            path.display()
        )));
    }
    let content = fs::read_to_string(path)?;
    let lock: WorkspaceLock = serde_yaml::from_str(&content)?;
    Ok(lock)
}

pub fn save_workspace_lock(
    root: &Path,
    folder: &str,
    lock: &WorkspaceLock,
) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, folder);
    fs::create_dir_all(&dir)?;
    let content = serde_yaml::to_string(lock)?;
    fs::write(dir.join("locks.yaml"), content)?;
    Ok(())
}

// ==========================================
// Derived artifacts
// ==========================================

pub fn generate_code_workspace(
    root: &Path,
    folder: &str,
    services: &[String],
) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, folder);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.code-workspace", folder));

    let mut folders = vec![serde_json::json!({
        "name": "control",
        "path": "../.."
    })];

    for s in services {
        folders.push(serde_json::json!({
            "name": s,
            "path": format!("repos/{}", s)
        }));
        // Task worktrees are surfaced too, if present.
    }
    for task in load_workspace_tasks(root, folder) {
        for s in &task.repos {
            folders.push(serde_json::json!({
                "name": format!("{}:{}", task.slug, s),
                "path": format!("tasks/{}/{}", task.slug, s)
            }));
        }
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

/// Best-effort read of a workspace's task list (missing file → empty).
fn load_workspace_tasks(root: &Path, folder: &str) -> Vec<WorkspaceTask> {
    if let Ok(ws) = load_workspace(root, folder) {
        ws.tasks
    } else {
        Vec::new()
    }
}

/// The single minimal resume file (agent, auto-loaded on `cd`).
pub fn generate_resume_file(
    root: &Path,
    folder: &str,
    ws: &Workspace,
) -> Result<(), WorkspaceError> {
    let dir = get_workspace_dir(root, folder);
    fs::create_dir_all(&dir)?;

    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", ws.title));
    if !ws.description.is_empty() {
        md.push_str(&format!("{}\n\n", ws.description));
    }
    md.push_str(&format!("- **workspace**: `{}`\n", ws.id));
    match &ws.ticket {
        Some(t) => md.push_str(&format!("- **ticket**: `{}`\n", t)),
        None => md.push_str("- **ticket**: unticketed\n"),
    }
    md.push_str(&format!("- **services**: {}\n", ws.services.join(", ")));
    md.push_str(&format!("- **base_branch**: `{}`\n", ws.base_branch));
    if !ws.tasks.is_empty() {
        md.push_str("- **tasks**:\n");
        for t in &ws.tasks {
            md.push_str(&format!(
                "  - `{}` (`{}`) on `{}-{}` — {}\n",
                t.key,
                t.slug,
                ws.id,
                t.slug,
                t.repos.join(", ")
            ));
        }
    }
    md.push_str("\nResume with: `ws tasks`\n");

    fs::write(dir.join("AGENTS.md"), md)?;
    Ok(())
}

// ==========================================
// Status collection (shared by `workspace.status` and `ws tasks`)
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RepoStatus {
    pub service_id: String,
    pub branch: String,
    pub current_commit: String,
    pub baseline_commit: String,
    pub has_changes: bool,
    /// Commits ahead of the upstream tracking branch (`None` = no upstream configured).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unpushed_count: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TaskStatusOutput {
    pub key: String,
    pub slug: String,
    pub branch: String,
    pub repo_statuses: HashMap<String, RepoStatus>,
}

fn inspect_repo_dir(dir: &Path, service_id: &str, baseline_commit: &str) -> RepoStatus {
    let mut branch = "unknown".to_string();
    let mut current_commit = "unknown".to_string();
    let mut has_changes = false;
    let mut unpushed_count = None;

    if dir.exists() {
        if let Ok(b) = cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"])
            .dir(dir)
            .read()
        {
            branch = b.trim().to_string();
        }
        if let Ok(c) = cmd("git", &["rev-parse", "HEAD"]).dir(dir).read() {
            current_commit = c.trim().to_string();
        }
        if let Ok(status) = cmd("git", &["status", "--porcelain"]).dir(dir).read() {
            has_changes = !status.trim().is_empty();
        }
        // Count commits ahead of the upstream tracking branch (best-effort).
        if let Ok(n) = cmd("git", &["rev-list", "--count", "@{u}..HEAD"])
            .dir(dir)
            .read()
        {
            unpushed_count = n.trim().parse::<u64>().ok();
        }
    }

    RepoStatus {
        service_id: service_id.to_string(),
        branch,
        current_commit,
        baseline_commit: baseline_commit.to_string(),
        has_changes,
        unpushed_count,
    }
}

/// Per-service status of the main `repos/<service>` worktrees.
pub fn collect_repo_statuses(
    root: &Path,
    folder: &str,
    ws: &Workspace,
    baselines: &HashMap<String, LockedRepo>,
) -> HashMap<String, RepoStatus> {
    let mut out = HashMap::new();
    for service_id in &ws.services {
        let worktree_dir = get_workspace_dir(root, folder)
            .join("repos")
            .join(service_id);
        let baseline = baselines
            .get(service_id)
            .map(|r| r.baseline_commit.clone())
            .unwrap_or_else(|| "unknown".to_string());
        out.insert(
            service_id.clone(),
            inspect_repo_dir(&worktree_dir, service_id, &baseline),
        );
    }
    out
}

/// Per-task status of `tasks/<slug>/<service>` worktrees (slice 4).
pub fn collect_task_statuses(
    root: &Path,
    folder: &str,
    ws: &Workspace,
    task_baselines: &HashMap<String, HashMap<String, LockedRepo>>,
) -> Vec<TaskStatusOutput> {
    let mut out = Vec::new();
    for task in &ws.tasks {
        let mut repo_statuses = HashMap::new();
        for svc in &task.repos {
            let dir = get_workspace_dir(root, folder)
                .join("tasks")
                .join(&task.slug)
                .join(svc);
            let baseline = task_baselines
                .get(&task.slug)
                .and_then(|m| m.get(svc))
                .map(|r| r.baseline_commit.clone())
                .unwrap_or_else(|| "unknown".to_string());
            repo_statuses.insert(svc.clone(), inspect_repo_dir(&dir, svc, &baseline));
        }
        out.push(TaskStatusOutput {
            key: task.key.clone(),
            slug: task.slug.clone(),
            branch: format!("{}-{}", ws.id, task.slug),
            repo_statuses,
        });
    }
    out
}

/// Fallback lock for workspaces without `locks.yaml` (e.g. hand-authored ones).
pub fn load_workspace_lock_or_default(root: &Path, folder: &str, ws: &Workspace) -> WorkspaceLock {
    load_workspace_lock(root, folder).unwrap_or_else(|_| WorkspaceLock {
        id: ws.id.clone(),
        repos: HashMap::new(),
        tasks: HashMap::new(),
    })
}

// ==========================================
// Create / grow / attach / task worktrees
// ==========================================

pub struct CreateWorkspaceRequest {
    pub id: String,
    pub ticket: Option<String>,
    pub title: String,
    pub description: String,
    pub services: Vec<ServiceCatalog>,
    pub base_branch: String,
    pub create_branches: bool,
    pub editor: String,
}

pub async fn create_epic_workspace(
    root: &Path,
    code_provider: &dyn CodeProvider,
    req: CreateWorkspaceRequest,
) -> Result<Workspace, WorkspaceError> {
    let ticket = req
        .ticket
        .as_ref()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());

    let ws = Workspace {
        id: req.id.clone(),
        ticket: ticket.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        services: req.services.iter().map(|s| s.id.clone()).collect(),
        base_branch: req.base_branch.clone(),
        create_branches: req.create_branches,
        editor: req.editor.clone(),
        tasks: Vec::new(),
    };

    let folder = workspace_dir_name(&ws);
    let ws_dir = get_workspace_dir(root, &folder);
    if ws_dir.join("workspace.yaml").exists() {
        return Err(WorkspaceError::Validation(format!(
            "A workspace already exists at {}",
            ws_dir.display()
        )));
    }
    fs::create_dir_all(&ws_dir)?;

    info!(
        "Creating workspace '{}' ({}) with {} services",
        ws.id,
        folder,
        req.services.len()
    );

    let mut locked_repos = HashMap::new();

    for service in &req.services {
        info!("Setting up service '{}'...", service.id);

        code_provider
            .ensure_repo_cache(EnsureRepoCacheInput {
                owner: service.repo.owner.clone(),
                name: service.repo.name.clone(),
                url: service.repo.url.clone(),
            })
            .await?;

        let branch = if ws.create_branches {
            ws.ticket.clone().unwrap_or_else(|| ws.id.clone())
        } else {
            service.repo.default_branch.clone()
        };

        let worktree = code_provider
            .create_worktree(CreateWorktreeInput {
                owner: service.repo.owner.clone(),
                name: service.repo.name.clone(),
                url: service.repo.url.clone(),
                folder: folder.clone(),
                service_id: service.id.clone(),
                base_branch: ws.base_branch.clone(),
                branch: branch.clone(),
                subdir: None,
            })
            .await?;

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

    let lock = WorkspaceLock {
        id: ws.id.clone(),
        repos: locked_repos,
        tasks: HashMap::new(),
    };

    save_workspace(root, &folder, &ws)?;
    save_workspace_lock(root, &folder, &lock)?;
    generate_code_workspace(root, &folder, &ws.services)?;
    generate_resume_file(root, &folder, &ws)?;

    Ok(ws)
}

pub async fn add_service_to_epic_workspace(
    root: &Path,
    code_provider: &dyn CodeProvider,
    folder: &str,
    service: ServiceCatalog,
) -> Result<Workspace, WorkspaceError> {
    let mut ws = load_workspace(root, folder)?;
    let mut lock = load_workspace_lock(root, folder)?;

    if ws.services.contains(&service.id) {
        warn!(
            "Service {} is already in workspace {} ({})",
            service.id, ws.id, folder
        );
        return Ok(ws);
    }

    info!("Adding service '{}' to workspace {}...", service.id, ws.id);

    code_provider
        .ensure_repo_cache(EnsureRepoCacheInput {
            owner: service.repo.owner.clone(),
            name: service.repo.name.clone(),
            url: service.repo.url.clone(),
        })
        .await?;

    let branch = if ws.create_branches {
        ws.ticket.clone().unwrap_or_else(|| ws.id.clone())
    } else {
        service.repo.default_branch.clone()
    };

    let worktree = code_provider
        .create_worktree(CreateWorktreeInput {
            owner: service.repo.owner.clone(),
            name: service.repo.name.clone(),
            url: service.repo.url.clone(),
            folder: folder.to_string(),
            service_id: service.id.clone(),
            base_branch: ws.base_branch.clone(),
            branch: branch.clone(),
            subdir: None,
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

    save_workspace(root, folder, &ws)?;
    save_workspace_lock(root, folder, &lock)?;
    generate_code_workspace(root, folder, &ws.services)?;
    generate_resume_file(root, folder, &ws)?;

    Ok(ws)
}

/// Add a task slice: one worktree per (task, repo) under `tasks/<slug>/<repo>`,
/// on branch `<ws.id>-<slug>` (git forbids nesting a ref under a branch ref),
/// recording each baseline in `locks.yaml`.
pub async fn add_task_worktrees(
    root: &Path,
    code_provider: &dyn CodeProvider,
    folder: &str,
    key: &str,
    slug: &str,
    services: &[String],
) -> Result<Workspace, WorkspaceError> {
    let mut ws = load_workspace(root, folder)?;
    let mut lock = load_workspace_lock(root, folder)?;

    let task_index = ws.tasks.iter().position(|t| t.slug == slug);
    let mut task_repos = task_index
        .map(|i| ws.tasks[i].repos.clone())
        .unwrap_or_default();

    let task_baselines = lock.tasks.entry(slug.to_string()).or_default();

    for svc in services {
        if task_repos.contains(svc) {
            warn!(
                "Task {} in workspace {} already has a worktree for {}",
                slug, ws.id, svc
            );
            continue;
        }

        let svc_catalog = ws_catalog::get_service(root, svc)?;
        code_provider
            .ensure_repo_cache(EnsureRepoCacheInput {
                owner: svc_catalog.repo.owner.clone(),
                name: svc_catalog.repo.name.clone(),
                url: svc_catalog.repo.url.clone(),
            })
            .await?;

        let worktree = code_provider
            .create_worktree(CreateWorktreeInput {
                owner: svc_catalog.repo.owner.clone(),
                name: svc_catalog.repo.name.clone(),
                url: svc_catalog.repo.url.clone(),
                folder: folder.to_string(),
                service_id: svc.clone(),
                base_branch: ws.base_branch.clone(),
                branch: format!("{}-{}", ws.id, slug),
                subdir: Some(format!("tasks/{}/{}", slug, svc)),
            })
            .await?;

        let baseline_commit = cmd("git", &["rev-parse", "HEAD"])
            .dir(&worktree.path)
            .read()
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        task_baselines.insert(
            svc.clone(),
            LockedRepo {
                provider: svc_catalog.repo.provider.clone(),
                owner: svc_catalog.repo.owner.clone(),
                name: svc_catalog.repo.name.clone(),
                default_branch: svc_catalog.repo.default_branch.clone(),
                baseline_commit,
            },
        );
        task_repos.push(svc.clone());
    }

    match task_index {
        Some(i) => ws.tasks[i].repos = task_repos,
        None => ws.tasks.push(WorkspaceTask {
            key: key.to_string(),
            slug: slug.to_string(),
            repos: task_repos,
        }),
    }

    save_workspace(root, folder, &ws)?;
    save_workspace_lock(root, folder, &lock)?;
    generate_code_workspace(root, folder, &ws.services)?;
    generate_resume_file(root, folder, &ws)?;

    Ok(ws)
}

/// Attach a ticket: sets `workspace.ticket`, rewrites the folder to `TICKET-slug`,
/// moves worktrees with `git worktree move`, rewires derived files. Identity is
/// content-based, so only the folder name changes. Returns the new folder + workspace.
pub async fn attach_workspace(
    root: &Path,
    code_provider: &dyn CodeProvider,
    folder: &str,
    ticket: &str,
) -> Result<(String, Workspace), WorkspaceError> {
    let ticket = ticket.trim().to_string();
    if ticket.is_empty() {
        return Err(WorkspaceError::Validation(
            "Ticket must not be empty.".to_string(),
        ));
    }

    let mut ws = load_workspace(root, folder)?;
    if let Some(existing) = &ws.ticket {
        if existing == &ticket {
            return Ok((folder.to_string(), ws));
        }
        return Err(WorkspaceError::Validation(format!(
            "Workspace {} is already attached to ticket '{}'; detach is not supported yet.",
            ws.id, existing
        )));
    }

    ws.ticket = Some(ticket.clone());
    let new_folder = workspace_dir_name(&ws);
    let old_dir = get_workspace_dir(root, folder);
    let new_dir = get_workspace_dir(root, &new_folder);

    if new_folder == folder {
        save_workspace(root, folder, &ws)?;
        generate_resume_file(root, folder, &ws)?;
        return Ok((folder.to_string(), ws));
    }
    if new_dir.join("workspace.yaml").exists() {
        return Err(WorkspaceError::Validation(format!(
            "Cannot attach: a workspace already exists at {}",
            new_dir.display()
        )));
    }
    fs::create_dir_all(&new_dir)?;

    let lock = load_workspace_lock(root, folder)?;

    // Move main worktrees first (git worktree metadata follows the move).
    for svc in &ws.services {
        if let Some(lr) = lock.repos.get(svc) {
            move_worktree(
                code_provider,
                &old_dir.join("repos").join(svc),
                &new_dir.join("repos").join(svc),
                &lr.owner,
                &lr.name,
            )
            .await?;
        }
    }
    // Then task worktrees.
    for task in &ws.tasks {
        if let Some(svcs) = lock.tasks.get(&task.slug) {
            for svc in &task.repos {
                if let Some(lr) = svcs.get(svc) {
                    move_worktree(
                        code_provider,
                        &old_dir.join("tasks").join(&task.slug).join(svc),
                        &new_dir.join("tasks").join(&task.slug).join(svc),
                        &lr.owner,
                        &lr.name,
                    )
                    .await?;
                }
            }
        }
    }

    // Move side files, then rewrite under the new folder.
    for name in ["workspace.yaml", "locks.yaml", "AGENTS.md"] {
        let old_f = old_dir.join(name);
        if old_f.exists() {
            fs::rename(&old_f, new_dir.join(name))?;
        }
    }

    save_workspace(root, &new_folder, &ws)?;
    if new_dir.join("locks.yaml").exists() {
        let content = fs::read_to_string(new_dir.join("locks.yaml"))?;
        let mut lock: WorkspaceLock = serde_yaml::from_str(&content)?;
        lock.id = ws.id.clone();
        fs::write(new_dir.join("locks.yaml"), serde_yaml::to_string(&lock)?)?;
    }
    generate_resume_file(root, &new_folder, &ws)?;
    generate_code_workspace(root, &new_folder, &ws.services)?;

    let old_cws = old_dir.join(format!("{}.code-workspace", folder));
    if old_cws.exists() {
        fs::remove_file(old_cws)?;
    }
    if old_dir.exists() {
        fs::remove_dir_all(&old_dir)?;
    }

    Ok((new_folder, ws))
}

async fn move_worktree(
    code_provider: &dyn CodeProvider,
    from: &Path,
    to: &Path,
    owner: &str,
    name: &str,
) -> Result<(), WorkspaceError> {
    if !from.exists() {
        warn!("Worktree {} missing; skipping move.", from.display());
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    code_provider
        .move_worktree(MoveWorktreeInput {
            owner: owner.to_string(),
            name: name.to_string(),
            from: from.to_string_lossy().into_owned(),
            to: to.to_string_lossy().into_owned(),
        })
        .await
}

// ==========================================
// AI Command Implementations
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceCreateInput {
    pub title: String,
    pub description: String,
    pub ticket: Option<String>,
    /// Explicit slug override; defaults to `kebab(title)`.
    pub id: Option<String>,
    pub services: Vec<String>,
    pub base_branch: Option<String>,
    pub create_branches: Option<bool>,
    pub editor: Option<String>,
}

pub struct WorkspaceCreateCommand;

#[async_trait]
impl AiCommand for WorkspaceCreateCommand {
    const ID: &'static str = "workspace.create";
    const DESCRIPTION: &'static str =
        "Create a per-feature multi-repo workspace (slug identity; ticket optional).";
    type Input = WorkspaceCreateInput;
    type Output = Workspace;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for workspace creation".to_string())
        })?;

        let mut services = Vec::new();
        for sid in &input.services {
            services.push(ws_catalog::get_service(&ctx.workspace_root, sid)?);
        }

        let base_branch = input.base_branch.unwrap_or_else(|| "main".to_string());
        let create_branches = input.create_branches.unwrap_or(true);
        let editor = input
            .editor
            .unwrap_or_else(|| ctx.config.editor.default.clone());
        let id = input.id.unwrap_or_else(|| slugify(&input.title));

        create_epic_workspace(
            &ctx.workspace_root,
            code_provider.as_ref(),
            CreateWorkspaceRequest {
                id,
                ticket: input.ticket.clone(),
                title: input.title.clone(),
                description: input.description.clone(),
                services,
                base_branch,
                create_branches,
                editor,
            },
        )
        .await
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceQueryInput {
    /// Matches workspace `id` (slug), `ticket`, folder name, or folder prefix.
    pub q: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAddServiceInput {
    pub q: String,
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
    const DESCRIPTION: &'static str =
        "Add a service to an active feature workspace (resolved by id/ticket/prefix).";
    type Input = WorkspaceAddServiceInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for adding a service".to_string())
        })?;

        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        let svc_catalog = ws_catalog::get_service(&ctx.workspace_root, &input.service_id)?;

        let ws = add_service_to_epic_workspace(
            &ctx.workspace_root,
            code_provider.as_ref(),
            &entry.folder,
            svc_catalog,
        )
        .await?;

        Ok(StatusOutput {
            success: true,
            message: format!(
                "Service {} added to workspace {} ({}).",
                input.service_id, ws.id, entry.folder
            ),
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAddTaskInput {
    pub q: String,
    /// Task key, e.g. `TASK-1`.
    pub key: String,
    /// Task slug for folder/branch names (`tasks/<slug>`, branch `<ws.id>-<slug>`).
    /// Defaults to `kebab(key)`.
    pub slug: Option<String>,
    /// Services to include; default = all workspace services.
    pub services: Option<Vec<String>>,
}

pub struct WorkspaceAddTaskCommand;

#[async_trait]
impl AiCommand for WorkspaceAddTaskCommand {
    const ID: &'static str = "workspace.add_task";
    const DESCRIPTION: &'static str =
        "Create per-task worktrees (tasks/<slug>/<repo>) for a feature workspace.";
    type Input = WorkspaceAddTaskInput;
    type Output = Workspace;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for task worktrees".to_string())
        })?;

        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        let slug = input.slug.unwrap_or_else(|| slugify(&input.key));
        let services = input.services.unwrap_or_else(|| entry.ws.services.clone());

        add_task_worktrees(
            &ctx.workspace_root,
            code_provider.as_ref(),
            &entry.folder,
            &input.key,
            &slug,
            &services,
        )
        .await
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAttachInput {
    pub q: String,
    /// The issue key to attach, e.g. `EPIC-123`.
    pub ticket: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAttachOutput {
    pub id: String,
    pub ticket: String,
    pub folder: String,
    pub path: String,
}

pub struct WorkspaceAttachCommand;

#[async_trait]
impl AiCommand for WorkspaceAttachCommand {
    const ID: &'static str = "workspace.attach";
    const DESCRIPTION: &'static str =
        "Attach a ticket to an unticketed workspace and move it to TICKET-slug.";
    type Input = WorkspaceAttachInput;
    type Output = WorkspaceAttachOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
            WorkspaceError::Config("No code provider configured for attach".to_string())
        })?;

        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        let (new_folder, ws) = attach_workspace(
            &ctx.workspace_root,
            code_provider.as_ref(),
            &entry.folder,
            &input.ticket,
        )
        .await?;

        Ok(WorkspaceAttachOutput {
            id: ws.id.clone(),
            ticket: ws.ticket.clone().unwrap_or_else(|| input.ticket.clone()),
            folder: new_folder.clone(),
            path: get_workspace_dir(&ctx.workspace_root, &new_folder)
                .to_string_lossy()
                .into_owned(),
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceStatusOutput {
    pub folder: String,
    pub id: String,
    pub ticket: Option<String>,
    pub title: String,
    pub description: String,
    pub services: Vec<String>,
    pub base_branch: String,
    pub create_branches: bool,
    pub editor: String,
    pub tasks: Vec<TaskStatusOutput>,
    pub repo_statuses: HashMap<String, RepoStatus>,
}

pub struct WorkspaceStatusCommand;

#[async_trait]
impl AiCommand for WorkspaceStatusCommand {
    const ID: &'static str = "workspace.status";
    const DESCRIPTION: &'static str =
        "Show status of a workspace (resolved by id/ticket/prefix) and repositories.";
    type Input = WorkspaceQueryInput;
    type Output = WorkspaceStatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        let ws = entry.ws;
        let lock = load_workspace_lock_or_default(&ctx.workspace_root, &entry.folder, &ws);

        let repo_statuses =
            collect_repo_statuses(&ctx.workspace_root, &entry.folder, &ws, &lock.repos);
        let tasks = collect_task_statuses(&ctx.workspace_root, &entry.folder, &ws, &lock.tasks);

        Ok(WorkspaceStatusOutput {
            folder: entry.folder,
            id: ws.id,
            ticket: ws.ticket,
            title: ws.title,
            description: ws.description,
            services: ws.services,
            base_branch: ws.base_branch,
            create_branches: ws.create_branches,
            editor: ws.editor,
            tasks,
            repo_statuses,
        })
    }
}

pub struct WorkspaceLockCommand;

#[async_trait]
impl AiCommand for WorkspaceLockCommand {
    const ID: &'static str = "workspace.lock";
    const DESCRIPTION: &'static str =
        "Retrieve workspace lockfile details (resolved by id/ticket/prefix).";
    type Input = WorkspaceQueryInput;
    type Output = WorkspaceLock;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        load_workspace_lock(&ctx.workspace_root, &entry.folder)
    }
}

pub struct WorkspaceGenerateEditorFilesCommand;

#[async_trait]
impl AiCommand for WorkspaceGenerateEditorFilesCommand {
    const ID: &'static str = "workspace.generate_editor_files";
    const DESCRIPTION: &'static str =
        "Regenerate editor-specific workspace configurations (resolved by id/ticket/prefix).";
    type Input = WorkspaceQueryInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let entry = resolve_workspace(&ctx.workspace_root, &input.q)?;
        generate_code_workspace(&ctx.workspace_root, &entry.folder, &entry.ws.services)?;
        Ok(StatusOutput {
            success: true,
            message: "Editor workspace files regenerated.".to_string(),
        })
    }
}
