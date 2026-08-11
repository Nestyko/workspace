mod assets;

use clap::{Parser, Subcommand};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use std::fs;
use std::io::IsTerminal;

use ws_core::command::{AiCommand, CommandRegistry};
use ws_core::context::CommandContext;
use ws_core::editors::EditorAdapter;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    CatalogDoc, CatalogIssueTracking, CatalogRepo, LocalConfig, ProductAgent, ProductCatalog,
    ProductServices, ServiceCatalog, TeamCatalog, Workspace,
};
use ws_core::providers::{CodeProvider, DocProvider, IssueProvider};

// Provider imports
use ws_provider_dex::DexProvider;
use ws_provider_github_gh::GitHubGhProvider;
use ws_provider_jira::confluence::ConfluenceProvider;
use ws_provider_jira::JiraProvider;

// Editor imports
use ws_editors::{
    CursorEditorAdapter, EditorOpenCommand, VSCodeEditorAdapter, VimEditorAdapter, ZedEditorAdapter,
};

// Catalog imports
use ws_catalog::{
    CatalogProductAddCommand, CatalogProductGetCommand, CatalogProductListCommand,
    CatalogServiceAddCommand, CatalogServiceGetCommand, CatalogServiceListCommand,
    CatalogServiceUpdateCommand, CatalogTeamAddCommand, CatalogTeamGetCommand,
    CatalogTeamListCommand, CatalogValidateCommand, ContextResolveCommand,
};

// Workspace imports
use ws_workspace::{
    WorkspaceAddServiceCommand, WorkspaceAddTaskCommand, WorkspaceAttachCommand,
    WorkspaceCreateCommand, WorkspaceGenerateEditorFilesCommand, WorkspaceLockCommand,
    WorkspaceStatusCommand,
};

// Repo imports
use ws_repo::{
    RepoFixLoopPromptCommand, RepoHealthcheckCommand, RepoRunCommand, RepoUnderstandVerifyCommand,
    RepoVerifyCommand,
};

// Dev imports
use ws_dev::{dev_install, dev_purge, dev_uninstall, DevInstallInput, DEFAULT_DEV_NAME};

// Provider command imports
use ws_providers::{
    PrCreateCommand, ProviderCodeCheckAuthCommand, ProviderCodeGetRepoCommand,
    ProviderCodeListRecentReposCommand, ProviderConfigGetInstructionsCommand,
    ProviderConfigSyncInstructionsCommand, ProviderConfigSyncInstructionsInput,
    ProviderDocCheckAuthCommand, ProviderDocCreatePageCommand, ProviderDocGetPageCommand,
    ProviderDocUpdatePageCommand, ProviderIssueCheckAuthCommand, ProviderIssueCommentCommand,
    ProviderIssueCreateEpicCommand, ProviderIssueCreateIssueCommand, ProviderIssueGetIssueCommand,
    ProviderIssueLinkCommand,
};

#[derive(Parser, Clone, Debug)]
#[command(name = "ws", version)]
#[command(about = "Rust Multi-Repo AI Workspace CLI", long_about = None)]
struct Cli {
    #[arg(short, long, global = true, help = "Verbose logging output")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    #[command(about = "Initialize the workspace, config, and catalog structure")]
    Init,

    #[command(about = "Manage local configuration")]
    Config {
        #[command(subcommand)]
        config_sub: Option<ConfigSub>,
    },

    #[command(about = "Discover recently updated repositories and add to catalog")]
    Discover(DiscoverArgs),

    #[command(about = "Add repository, product, or team manually to the catalog")]
    Add {
        #[command(subcommand)]
        add_sub: AddSub,
    },

    #[command(about = "Open an implementation workspace or specific service in an editor")]
    Open(OpenArgs),

    #[command(about = "Show status of the workspace, catalog, or a specific epic")]
    Status(StatusArgs),

    #[command(about = "List, inspect and select open feature workspaces (the resume CLI)")]
    Tasks(TasksArgs),

    #[command(about = "Attach a ticket to an unticketed workspace (moves it to TICKET-slug)")]
    Attach(AttachArgs),

    #[command(about = "Create Pull Requests for the active workspace")]
    Pr {
        #[command(subcommand)]
        pr_sub: PrSub,
    },

    #[command(about = "AI-facing command plane")]
    Ai {
        #[command(subcommand)]
        ai_sub: AiSub,
    },

    #[command(about = "Knowledge base: scaffold the catalog/knowledge/ wiki")]
    Kb {
        #[command(subcommand)]
        kb_sub: KbSub,
    },

    #[command(about = "Install a `ws` dev binary built from a GitHub Pull Request")]
    DevInstall(DevInstallArgs),

    #[command(about = "Uninstall every `ws` dev binary installed from a PR")]
    DevUninstall(DevUninstallArgs),

    #[command(about = "Remove all installed `ws` dev binaries")]
    DevPurge(DevPurgeArgs),
}

#[derive(clap::Args, Clone, Debug)]
struct DevInstallArgs {
    #[arg(help = "GitHub PR URL (e.g. https://github.com/Nestyko/workspace/pull/14)")]
    pr_url: String,

    #[arg(
        long,
        help = "Name for the dev binary (default: ws-dev). Refuses 'ws'."
    )]
    name: Option<String>,

    #[arg(long, help = "Replace an existing binary at the destination path")]
    force: bool,
}

#[derive(clap::Args, Clone, Debug)]
struct DevUninstallArgs {
    #[arg(help = "GitHub PR URL of the dev install(s) to remove")]
    pr_url: String,
}

#[derive(clap::Args, Clone, Debug)]
struct DevPurgeArgs {
    #[arg(long, short = 'y', help = "Do not prompt for confirmation")]
    yes: bool,
}

#[derive(clap::Args, Clone, Debug)]
struct DiscoverArgs {
    #[arg(long, help = "Limit the number of repositories fetched (default 50)")]
    limit: Option<usize>,
}

#[derive(clap::Args, Clone, Debug)]
struct OpenArgs {
    #[arg(help = "Workspace query: id (slug), ticket, folder name, or folder prefix")]
    epic_key: String,

    #[arg(long, help = "Specify the editor (cursor, vscode, zed, vim)")]
    editor: Option<String>,

    #[arg(long, help = "Specify the service name to open directly")]
    service: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
struct StatusArgs {
    #[arg(help = "The Jira Epic key (e.g. ACME-123)")]
    epic_key: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
struct TasksArgs {
    #[arg(value_name = "TOKEN", help = "Shorthand for `ws tasks open <TOKEN>`")]
    token: Option<String>,

    #[command(subcommand)]
    sub: Option<TasksSub>,
}

#[derive(Subcommand, Clone, Debug)]
enum TasksSub {
    #[command(about = "List open workspaces (and their tasks) as scan-able one-liners")]
    List,
    #[command(about = "Expand a single workspace/task row into full detail")]
    Show { token: String },
    #[command(about = "Print the path of a workspace/task so you can cd into it")]
    Open { token: String },
}

#[derive(clap::Args, Clone, Debug)]
struct AttachArgs {
    #[arg(
        value_name = "QUERY",
        help = "Workspace id (slug), ticket, folder name, or folder prefix"
    )]
    q: String,

    #[arg(long, help = "The issue key to attach (e.g. EPIC-123)")]
    ticket: String,
}

#[derive(Subcommand, Clone, Debug)]
enum ConfigSub {
    #[command(about = "Print the entire configuration")]
    Get,
    #[command(about = "Set a configuration parameter (e.g. 'editor cursor')")]
    Set { key: String, value: String },
}

#[derive(Subcommand, Clone, Debug)]
enum AddSub {
    #[command(about = "Add a repository to the catalog as a service")]
    Repo { name: String },
    #[command(about = "Add a product to the catalog")]
    Product { name: String },
    #[command(about = "Add a team to the catalog")]
    Team { name: String },
}

#[derive(Subcommand, Clone, Debug)]
enum PrSub {
    #[command(about = "Create a Pull Request for a specific service or all services")]
    Create {
        #[arg(help = "The Jira Epic key (e.g. ACME-123)")]
        epic_key: String,

        #[arg(long, help = "Create PR for a specific service")]
        service: Option<String>,

        #[arg(long, help = "Create PR for all services in the workspace")]
        all: bool,

        #[arg(long, help = "Draft Pull Request")]
        draft: bool,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum KbSub {
    #[command(about = "Scaffold the knowledge-base tree from embedded assets (skip-by-default)")]
    Init {
        #[arg(
            long,
            help = "Refresh a single embedded asset by relative path (e.g. SCHEMA.md)"
        )]
        reset: Option<String>,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum AiSub {
    #[command(about = "List all supported commands and metadata")]
    Manifest,

    #[command(about = "Generate AI documentation of the command APIs")]
    Docs {
        #[command(subcommand)]
        docs_sub: AiDocsSub,
    },

    #[command(about = "Output JSON schema of a command input or output")]
    Schema {
        #[arg(help = "The command ID (e.g. workspace.create)")]
        command_id: String,
        #[arg(help = "schema kind ('input' or 'output')")]
        kind: String,
    },

    #[command(about = "Run an AI command using a JSON input file")]
    Run {
        #[arg(help = "The command ID (e.g. workspace.create)")]
        command_id: String,
        #[arg(long, help = "Path to the input JSON file")]
        input: PathBuf,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum AiDocsSub {
    #[command(about = "Generate command-api.md under docs/")]
    Generate,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let args = Cli::parse();

    // Initialize Logging
    let filter = if args.verbose {
        EnvFilter::new("info,ws=debug")
    } else {
        EnvFilter::new("info")
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .ok();

    // Find Workspace Root
    let workspace_root = ws_config::find_workspace_root()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Load Local Config
    let config = ws_config::load_config(&workspace_root).unwrap_or_default();

    // Setup Providers
    let code_provider: Option<Arc<dyn CodeProvider>> = match config.code_provider.r#type.as_str() {
        "github-gh" => Some(Arc::new(GitHubGhProvider::new(
            config.code_provider.default_owner.clone(),
            config.code_provider.protocol.clone(),
        ))),
        _ => None,
    };

    let issue_provider: Option<Arc<dyn IssueProvider>> = match config.issue_provider.r#type.as_str()
    {
        "jira" => Some(Arc::new(JiraProvider::new(
            config.issue_provider.base_url.clone(),
            config.issue_provider.default_project.clone(),
        ))),
        "dex" => Some(Arc::new(DexProvider::new(
            config.issue_provider.default_project.clone(),
        ))),
        _ => None,
    };

    let doc_provider: Option<Arc<dyn DocProvider>> =
        config
            .doc_provider
            .as_ref()
            .and_then(|doc_cfg| match doc_cfg.r#type.as_str() {
                "confluence" => Some(Arc::new(ConfluenceProvider::new(
                    doc_cfg.base_url.clone(),
                    doc_cfg.default_space.clone(),
                )) as Arc<dyn DocProvider>),
                _ => None,
            });

    // Setup Editor Adapters
    let mut editor_adapters: HashMap<String, Arc<dyn EditorAdapter>> = HashMap::new();
    editor_adapters.insert("cursor".to_string(), Arc::new(CursorEditorAdapter));
    editor_adapters.insert("vscode".to_string(), Arc::new(VSCodeEditorAdapter));
    editor_adapters.insert("zed".to_string(), Arc::new(ZedEditorAdapter));
    editor_adapters.insert("vim".to_string(), Arc::new(VimEditorAdapter));

    let ctx = CommandContext::new(
        config.clone(),
        workspace_root.clone(),
        issue_provider.clone(),
        code_provider.clone(),
        doc_provider.clone(),
        editor_adapters.clone(),
    );

    // Setup Registry
    let mut registry = CommandRegistry::new();
    // Catalog commands
    registry.register(CatalogValidateCommand);
    registry.register(CatalogServiceAddCommand);
    registry.register(CatalogServiceUpdateCommand);
    registry.register(CatalogServiceGetCommand);
    registry.register(CatalogServiceListCommand);
    registry.register(CatalogProductAddCommand);
    registry.register(CatalogProductGetCommand);
    registry.register(CatalogProductListCommand);
    registry.register(CatalogTeamAddCommand);
    registry.register(CatalogTeamGetCommand);
    registry.register(CatalogTeamListCommand);
    registry.register(ContextResolveCommand);
    // Providers commands
    registry.register(ProviderCodeCheckAuthCommand);
    registry.register(ProviderCodeListRecentReposCommand);
    registry.register(ProviderCodeGetRepoCommand);
    registry.register(ProviderIssueCheckAuthCommand);
    registry.register(ProviderIssueGetIssueCommand);
    registry.register(ProviderIssueCreateEpicCommand);
    registry.register(ProviderIssueCreateIssueCommand);
    registry.register(ProviderIssueLinkCommand);
    registry.register(ProviderIssueCommentCommand);
    registry.register(PrCreateCommand);
    // Doc provider commands
    registry.register(ProviderDocCheckAuthCommand);
    registry.register(ProviderDocGetPageCommand);
    registry.register(ProviderDocCreatePageCommand);
    registry.register(ProviderDocUpdatePageCommand);
    registry.register(ProviderConfigGetInstructionsCommand);
    registry.register(ProviderConfigSyncInstructionsCommand);
    // Workspace commands
    registry.register(WorkspaceCreateCommand);
    registry.register(WorkspaceAddServiceCommand);
    registry.register(WorkspaceAddTaskCommand);
    registry.register(WorkspaceAttachCommand);
    registry.register(WorkspaceStatusCommand);
    registry.register(WorkspaceLockCommand);
    registry.register(WorkspaceGenerateEditorFilesCommand);
    // Editor command
    registry.register(EditorOpenCommand);
    // Repo commands
    registry.register(RepoHealthcheckCommand);
    registry.register(RepoRunCommand);
    registry.register(RepoVerifyCommand);
    registry.register(RepoFixLoopPromptCommand);
    registry.register(RepoUnderstandVerifyCommand);

    run_cli(&workspace_root, &config, &ctx, &registry, args.command)
        .await
        .map_err(|e| miette::Report::new(e))
}

async fn run_cli(
    workspace_root: &Path,
    config: &LocalConfig,
    ctx: &CommandContext,
    registry: &CommandRegistry,
    command: Commands,
) -> Result<(), WorkspaceError> {
    match command {
        Commands::Init => {
            handle_init(workspace_root, ctx).await?;
        }
        Commands::Config { config_sub } => {
            handle_config(workspace_root, config, config_sub)?;
        }
        Commands::Discover(args) => {
            handle_discover(workspace_root, ctx, args.limit).await?;
        }
        Commands::Add { add_sub } => {
            handle_add(workspace_root, ctx, add_sub).await?;
        }
        Commands::Open(args) => {
            handle_open(ctx.clone(), args.epic_key, args.editor, args.service).await?;
        }
        Commands::Status(args) => {
            handle_status(workspace_root, ctx.clone(), args.epic_key).await?;
        }
        Commands::Tasks(args) => {
            handle_tasks(workspace_root, args).await?;
        }
        Commands::Attach(args) => {
            handle_attach(ctx.clone(), args).await?;
        }
        Commands::Pr { pr_sub } => {
            handle_pr(ctx.clone(), pr_sub).await?;
        }
        Commands::Ai { ai_sub } => {
            handle_ai(ctx.clone(), registry, ai_sub).await?;
        }
        Commands::Kb { kb_sub } => {
            handle_kb(workspace_root, kb_sub)?;
        }
        Commands::DevInstall(args) => {
            handle_dev_install(args)?;
        }
        Commands::DevUninstall(args) => {
            handle_dev_uninstall(args)?;
        }
        Commands::DevPurge(args) => {
            handle_dev_purge(args)?;
        }
    }
    Ok(())
}

/// Write `README.md` into `dir` only when it does not already exist (never
/// clobber a user-authored README on re-runs of `ws init`).
fn write_folder_readme(dir: &Path, content: &str) -> Result<(), WorkspaceError> {
    fs::create_dir_all(dir)?;
    let readme = dir.join("README.md");
    if !readme.exists() {
        fs::write(&readme, content)?;
    }
    Ok(())
}

/// README.md seeded into `catalog/services/` by `ws init`.
const SERVICES_README: &str = r#"# catalog/services/

Each YAML file in this directory describes **one service** (a deployable repo
or library) in the workspace catalog, e.g. `catalog/services/payments.yaml`.

A service record tells the agent:

- **what** the service owns and when it is relevant (`owns`, `likely_relevant_when`),
- **how** to install, test, and lint it (`commands`),
- **where** its code lives (`repo`),
- **which team** owns it (`team`) and **which products** it belongs to (`products`),
- **how** issues are tracked for it (`issue_tracking`), and
- **where** its docs live (`docs`).

## How to populate it

1. **Auto-discover from your code provider** (fastest):
   ```bash
   ws discover --limit 10
   ```
2. **Add a single repo by `owner/name`:**
   ```bash
   ws add repo example-org/payments
   ```
3. **Author by hand:** copy `templates/service.yaml` to `catalog/services/<service-id>.yaml`
   and edit it.

See `templates/service.yaml` for the full annotated shape and `schemas/service.schema.json`
for the validation contract.
"#;

/// README.md seeded into `catalog/products/` by `ws init`.
const PRODUCTS_README: &str = r#"# catalog/products/

Each YAML file in this directory describes **one product** (a product area or
domain) in the workspace catalog, e.g. `catalog/products/acme.yaml`.

A product record tells the agent:

- **what** the product is (`description`),
- **which services** are primary vs. related to it (`services.primary`, `services.related`),
- **which external knowledge sources** (Confluence spaces, Jira projects) map to it
  (`knowledge_sources`), and
- **routing rules** that steer agent context to the right services (`routing_rules`).

## How to populate it

Add a product with:
```bash
ws add product acme
```
Then edit `catalog/products/acme.yaml` to wire in services and knowledge sources.

See `templates/product.yaml` for the full annotated shape and `schemas/product.schema.json`
for the validation contract.
"#;

/// README.md seeded into `catalog/teams/` by `ws init`.
const TEAMS_README: &str = r#"# catalog/teams/

Each YAML file in this directory describes **one team** in the workspace catalog,
e.g. `catalog/teams/platform.yaml`.

A team record tells the agent:

- the team's **name**, **description**, and **lead**, and
- the **members** (handles/usernames) on the team.

Teams are referenced by `service.team` so the agent knows who owns a service,
which is used for PR routing, review assignments, and context lookups.

## How to populate it

Add a team with:
```bash
ws add team platform
```
Then edit `catalog/teams/platform.yaml` to set the lead and members.

See `templates/team.yaml` for the full annotated shape and `schemas/team.schema.json`
for the validation contract.
"#;

/// The agent skills `ws init` auto-installs into the customer's repo. Only
/// `ws-repo-init` and `ws-self-heal` ship by default; `ws-init` is a
/// headless-bootstrap skill not needed after `ws init` has run.
const INIT_SKILLS: &[&str] = &["ws-repo-init", "ws-self-heal"];

/// Installs named skills from the embedded `skills/` tree into the customer's
/// repo, matching the layout the `skills` CLI produces so a later
/// `bunx skills add .` is a no-op for these entries:
///   - real files under `.agents/skills/<name>/`
///   - relative symlinks at `.pi/skills/<name>` and `.claude/skills/<name>`
///     (pointing at `../../.agents/skills/<name>`) so pi and Claude Code pick
///     them up.
///
/// Returns the names of the skills actually written. A skill missing from the
/// embedded tree is skipped (not fatal) so init never fails just because a
/// skill was renamed upstream — the caller narrates the difference.
fn install_repo_skills(root: &Path, names: &[&str]) -> Result<Vec<String>, WorkspaceError> {
    let mut installed = Vec::new();
    let skill_root = root.join(".agents").join("skills");
    for &name in names {
        let Some(dir) = assets::SKILLS.get_dir(name) else {
            continue;
        };
        fs::create_dir_all(skill_root.join(name))?;
        write_skill_tree(dir, &skill_root)?;

        let rel = format!("../../.agents/skills/{name}");
        for harness_dir in [".pi/skills", ".claude/skills"] {
            let link = root.join(harness_dir).join(name);
            symlink_skill(&rel, &link);
        }
        installed.push(name.to_string());
    }
    Ok(installed)
}

/// Recursively writes every file under the embedded skill `dir` into `skill_root`,
/// preserving each file's path relative to the embedded `skills/` root.
/// `include_dir::File::path()` is root-relative (e.g. `ws-repo-init/SKILL.md`),
/// so joining onto `skill_root` lands files at `.agents/skills/<name>/...`.
fn write_skill_tree(dir: &include_dir::Dir, skill_root: &Path) -> Result<(), WorkspaceError> {
    use include_dir::DirEntry;
    for entry in dir.entries() {
        match entry {
            DirEntry::File(f) => {
                let p = skill_root.join(f.path());
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&p, f.contents())?;
            }
            DirEntry::Dir(d) => {
                fs::create_dir_all(skill_root.join(d.path()))?;
                write_skill_tree(d, skill_root)?;
            }
        }
    }
    Ok(())
}

/// Creates a relative symlink at `link` pointing at `target`. Best-effort on
/// non-Unix hosts (falls back to copying the skill directory).
fn symlink_skill(target: &str, link: &Path) {
    if let Some(parent) = link.parent() {
        let _ = fs::create_dir_all(parent);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = fs::remove_file(link);
        let _ = symlink(target, link);
    }
    #[cfg(not(unix))]
    {
        //保守回退：直接复制一份（不标准，但保证文件可读）。
        let _ = (target, link);
    }
}

async fn handle_init(root: &Path, ctx: &CommandContext) -> Result<(), WorkspaceError> {
    println!("Welcome to AI Workspace.\n");

    let issue_provider = Select::new(
        "Select issue provider:",
        vec![
            "Jira",
            "Dex (local, no account needed)",
            "Linear (coming soon)",
            "GitHub (coming soon)",
        ],
    )
    .prompt()
    .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    // Dex needs no credentials/domain; Linear & GitHub are not implemented yet.
    let issue_provider_type = match issue_provider {
        "Jira" => "jira",
        "Dex (local, no account needed)" => "dex",
        other => {
            return Err(WorkspaceError::Other(format!(
                "Only Jira and Dex issue providers are currently supported; '{other}' is not yet available."
            )));
        }
    };

    let code_provider = Select::new(
        "Select code provider:",
        vec![
            "GitHub via gh",
            "GitLab (coming soon)",
            "Bitbucket (coming soon)",
        ],
    )
    .prompt()
    .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
    if code_provider != "GitHub via gh" {
        return Err(WorkspaceError::Other(
            "Only GitHub via gh is currently supported.".to_string(),
        ));
    }

    let default_editor = Select::new(
        "Select default editor:",
        vec!["cursor", "vscode", "zed", "vim"],
    )
    .prompt()
    .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let default_owner = Text::new("GitHub organization/user:")
        .prompt()
        .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    // Jira needs a base URL + project; Dex is local and needs neither.
    // We only prompt for these when the user actually selected Jira.
    let (jira_base_url, jira_project) = if issue_provider_type == "jira" {
        let base_url = Text::new("Jira Base URL (e.g. https://example.atlassian.net):")
            .prompt()
            .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
        let project = Text::new("Jira Default Project Key:")
            .prompt()
            .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
        (Some(base_url), Some(project))
    } else {
        (None, None)
    };

    // The document/knowledge provider is optional. The user may use Confluence
    // or skip it entirely (the workspace works without a doc provider).
    let doc_provider = Select::new(
        "Select document provider:",
        vec!["Confluence", "Skip (no document provider)"],
    )
    .prompt()
    .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let doc_provider_config: Option<ws_core::models::DocProviderConfig> =
        if doc_provider == "Confluence" {
            let confluence_base_url =
                Text::new("Confluence Base URL (e.g. https://example.atlassian.net):")
                    .prompt()
                    .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
            let confluence_space = Text::new("Confluence Default Space Key:")
                .prompt()
                .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
            Some(ws_core::models::DocProviderConfig {
                r#type: "confluence".to_string(),
                base_url: Some(confluence_base_url),
                default_space: Some(confluence_space),
            })
        } else {
            println!("Skipping document provider (no Confluence setup required).");
            None
        };

    // Validate GH Auth
    println!("\nValidating GitHub auth through gh...");
    let gh_provider = GitHubGhProvider::new(Some(default_owner.clone()), Some("ssh".to_string()));
    let auth = gh_provider.check_auth().await?;
    if auth.authenticated {
        println!(
            "✓ GitHub authentication successful. Username: {}",
            auth.username.unwrap_or_default()
        );
    } else {
        println!(
            "⚠ GitHub authentication failed: {}",
            auth.details.unwrap_or_default()
        );
    }

    // Build configuration
    let mut new_config = LocalConfig::default();
    new_config.code_provider.default_owner = Some(default_owner);
    new_config.editor.default = default_editor.to_string();
    new_config.issue_provider.r#type = issue_provider_type.to_string();
    new_config.issue_provider.base_url = jira_base_url;
    new_config.issue_provider.default_project = jira_project;
    new_config.doc_provider = doc_provider_config;

    ws_config::save_config(root, &new_config)?;
    println!(
        "\nSaved config to {}",
        ws_config::get_config_path(root).display()
    );

    // Create catalogs
    ws_catalog::ensure_catalog_dirs(root)?;

    // Scaffold the knowledge base (catalog/knowledge/); skip-by-default
    // preserves any existing user-edited wiki content on re-runs.
    ws_kb::scaffold(root, None)?;

    // Seed each catalog subfolder with a README.md (when absent) explaining its
    // purpose. The workspace is intentionally shipped empty of company-specific
    // sample entries — the user populates products/teams/services from their own
    // context following those READMEs.
    write_folder_readme(&ws_catalog::get_kind_dir(root, "services"), SERVICES_README)?;
    write_folder_readme(&ws_catalog::get_kind_dir(root, "products"), PRODUCTS_README)?;
    write_folder_readme(&ws_catalog::get_kind_dir(root, "teams"), TEAMS_README)?;

    // Scaffold a reference templates/service.yaml (when absent) the user can
    // mirror when authoring catalog/services/<id>.yaml entries by hand.
    fs::create_dir_all(root.join("templates"))?;
    let svc_template_path = root.join("templates").join("service.yaml");
    if !svc_template_path.exists() {
        let svc_template = r#"id: service-id
name: Service Name
kind: service
description: Service description
team: platform
products:
  - product-id
repo:
  provider: github
  owner: example-org
  name: service-repo
  url: git@github.com:example-org/service-repo.git
  default_branch: main
owns:
  - Feature A
likely_relevant_when:
  - query mentions feature A
commands:
  install: npm install
  test: npm test
issue_tracking:
  provider: dex
  project: PLATFORM
docs:
  - type: readme
    path: README.md
"#;
        fs::write(&svc_template_path, svc_template)?;
    }

    // Write the ws-managed root AGENTS.md: the compact harness contract pointing at the
    // knowledge base (catalog/knowledge/), the catalog (services/products/teams), and
    // where/how/when to start tasks. Runs on every init so a fresh workspace tells agents
    // where things live.
    let sync_ctx = CommandContext::new(
        new_config.clone(),
        root.to_path_buf(),
        None,
        None,
        None,
        HashMap::new(),
    );
    let sync_out = ProviderConfigSyncInstructionsCommand
        .run(sync_ctx, ProviderConfigSyncInstructionsInput {})
        .await?;
    println!("\nWrote root AGENTS.md at {}", sync_out.path);

    let start_discovery =
        Confirm::new("Would you like to discover repositories to add to the catalog?")
            .with_default(true)
            .prompt()
            .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    if start_discovery {
        let temp_ctx = CommandContext::new(
            new_config,
            root.to_path_buf(),
            None,
            Some(Arc::new(gh_provider)),
            None,
            HashMap::new(),
        );
        handle_discover(root, &temp_ctx, Some(50)).await?;
    }

    // Install the repo-level agent skills that `ws init` ships with.
    // Only `ws-repo-init` (bootstrap each cataloged repo) and `ws-self-heal`
    // (the healthcheck → fix → record loop) are auto-installed; `ws-init` is a
    // headless-bootstrap skill that is not needed once `ws init` has run.
    let installed_skills = install_repo_skills(root, INIT_SKILLS)?;

    println!("\nAI Workspace initialized successfully!");
    println!();
    if !installed_skills.is_empty() {
        println!("Installed agent skills (repo-level, under .agents/skills/):");
        if installed_skills.iter().any(|s| s == "ws-repo-init") {
            println!("  ✓ ws-repo-init — bootstrap each cataloged repo to ready-for-deep-pass.");
            println!("      Invoke it from your harness as `/ws-repo-init`.");
        }
        if installed_skills.iter().any(|s| s == "ws-self-heal") {
            println!("  ✓ ws-self-heal — run the per-repo healthcheck → fix → record loop.");
            println!("      Invoke it from your harness as `/ws-self-heal`.");
        }
        println!("  Symlinked into .pi/skills/ and .claude/skills/ for pi and Claude Code.");
        let missing = INIT_SKILLS
            .iter()
            .filter(|n| !installed_skills.iter().any(|s| s == *n))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            println!("  Note: not installed (missing from source tree): {missing:?}.");
        }
        println!();
    }
    println!("Next steps:");
    println!(
        "  1. Initialize your repos — with your harness run `/ws-repo-init` to bootstrap every cataloged repo from zero to \"cataloged + onboarding artifact + smoke-validated\" (see .agents/skills/ws-repo-init/SKILL.md)."
    );
    println!(
        "  2. Set up your products — follow catalog/products/README.md, then run `ws add product <name>`."
    );
    println!(
        "  3. Set up your teams — follow catalog/teams/README.md, then run `ws add team <name>`."
    );
    println!(
        "  4. (Optional) Add services — follow catalog/services/README.md, then run `ws discover` or `ws add repo <owner>/<name>`."
    );
    println!();
    println!("Tip: run `ws ai manifest` to see the full AI Command API.");
    Ok(())
}

fn handle_kb(root: &Path, kb_sub: KbSub) -> Result<(), WorkspaceError> {
    match kb_sub {
        KbSub::Init { reset } => {
            let summary = ws_kb::run_init(root, reset.as_deref())?;
            println!("{summary}");
        }
    }
    Ok(())
}

fn handle_dev_install(args: DevInstallArgs) -> Result<(), WorkspaceError> {
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| DEFAULT_DEV_NAME.to_string());
    let pr = ws_dev::parse_pr_url(&args.pr_url)?;
    println!(
        "Building dev binary `{name}` for PR {}/{}#{} ...",
        pr.owner, pr.repo, pr.pr_number
    );
    println!(
        "(this clones/fetches the repo and runs `cargo build --release`; first builds can take a while)"
    );
    let out = dev_install(DevInstallInput {
        pr_url: args.pr_url,
        name: Some(name.clone()),
        force: args.force,
    })?;
    println!();
    if out.overwritten {
        println!("\n✓ Replaced existing dev binary:");
    } else {
        println!("\n✓ Installed dev binary:");
    }
    println!("    name:        {}", out.install.name);
    println!("    path:        {}", out.install.binary_path);
    println!(
        "    pr:          {}/{}#{}",
        out.install.owner, out.install.repo, out.install.pr_number
    );
    println!(
        "    git sha:     {}",
        out.install.git_sha.as_deref().unwrap_or("(unknown)")
    );
    println!("    installed:   {}", out.install.installed_at);
    println!(
        "\nRun it with `{} --help` (it shares its interface with `ws`).",
        out.install.name
    );
    println!("Uninstall with: ws dev-uninstall {}", out.install.pr_url);
    Ok(())
}

fn handle_dev_uninstall(args: DevUninstallArgs) -> Result<(), WorkspaceError> {
    let out = dev_uninstall(&args.pr_url)?;
    if out.removed.is_empty() {
        println!("No dev binaries found for this PR.");
        return Ok(());
    }
    println!(
        "Removed {} dev binary/binary for this PR:",
        out.removed.len()
    );
    for e in &out.removed {
        println!("  - {} ({})", e.name, e.binary_path);
    }
    Ok(())
}

fn handle_dev_purge(args: DevPurgeArgs) -> Result<(), WorkspaceError> {
    let existing = ws_dev::load_registry()?;
    if existing.installs.is_empty() {
        println!("No dev binaries to purge.");
        return Ok(());
    }
    if !args.yes {
        let proceed = Confirm::new(&format!(
            "Remove all {} dev binary/binary(ies)? This cannot be undone.",
            existing.installs.len()
        ))
        .with_default(false)
        .prompt()
        .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
        if !proceed {
            println!("Aborted.");
            return Ok(());
        }
    }
    let out = dev_purge()?;
    println!("Purged {} dev binary/binary(ies):", out.removed.len());
    for e in &out.removed {
        println!("  - {} ({})", e.name, e.binary_path);
    }
    Ok(())
}

fn handle_config(
    root: &Path,
    config: &LocalConfig,
    sub: Option<ConfigSub>,
) -> Result<(), WorkspaceError> {
    match sub {
        None | Some(ConfigSub::Get) => {
            println!(
                "Config Path: {}",
                ws_config::get_config_path(root).display()
            );
            let yaml = serde_yaml::to_string(config).map_err(WorkspaceError::Yaml)?;
            println!("\n{}", yaml);
        }
        Some(ConfigSub::Set { key, value }) => {
            let mut new_config = config.clone();
            match key.as_str() {
                "editor" => {
                    new_config.editor.default = value.clone();
                }
                "issue-provider" => {
                    new_config.issue_provider.r#type = value.clone();
                }
                "doc-provider" => {
                    let lower = value.to_lowercase();
                    if lower == "none" || lower == "skip" || lower == "disabled" {
                        new_config.doc_provider = None;
                    } else if lower == "confluence" {
                        if new_config.doc_provider.is_none() {
                            new_config.doc_provider = Some(ws_core::models::DocProviderConfig {
                                r#type: "confluence".to_string(),
                                base_url: None,
                                default_space: None,
                            });
                        } else {
                            new_config.doc_provider.as_mut().unwrap().r#type =
                                "confluence".to_string();
                        }
                    } else {
                        return Err(WorkspaceError::Config(format!(
                            "Unknown document provider '{value}'. Supported: confluence, none."
                        )));
                    }
                }
                "code-provider" => {
                    new_config.code_provider.r#type = value.clone();
                }
                "code-owner" => {
                    new_config.code_provider.default_owner = Some(value.clone());
                }
                "jira-url" => {
                    new_config.issue_provider.base_url = Some(value.clone());
                }
                "jira-project" => {
                    new_config.issue_provider.default_project = Some(value.clone());
                }
                "confluence-url" => {
                    if new_config.doc_provider.is_none() {
                        new_config.doc_provider = Some(ws_core::models::DocProviderConfig {
                            r#type: "confluence".to_string(),
                            base_url: None,
                            default_space: None,
                        });
                    }
                    new_config.doc_provider.as_mut().unwrap().base_url = Some(value.clone());
                }
                "confluence-space" => {
                    if new_config.doc_provider.is_none() {
                        new_config.doc_provider = Some(ws_core::models::DocProviderConfig {
                            r#type: "confluence".to_string(),
                            base_url: None,
                            default_space: None,
                        });
                    }
                    new_config.doc_provider.as_mut().unwrap().default_space = Some(value.clone());
                }
                _ => {
                    return Err(WorkspaceError::Config(format!(
                        "Unknown configuration key: {}",
                        key
                    )));
                }
            }
            ws_config::save_config(root, &new_config)?;
            println!("Updated configuration '{}' to '{}'", key, value);
        }
    }
    Ok(())
}

async fn handle_discover(
    root: &Path,
    ctx: &CommandContext,
    limit: Option<usize>,
) -> Result<(), WorkspaceError> {
    let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
        WorkspaceError::Config("No code provider configured for discovery".to_string())
    })?;

    let limit_val = limit.unwrap_or(50);
    let mut page = 1;
    loop {
        println!("Fetching updated repositories (Page {})...", page);
        let repos = code_provider
            .list_recent_repos(ws_core::models::ListRecentReposInput {
                limit: Some(limit_val),
                page: Some(page),
            })
            .await?;

        if repos.is_empty() {
            println!("No repositories found.");
            break;
        }

        let repo_names: Vec<String> = repos.iter().map(|r| r.full_name.clone()).collect();
        let selected = MultiSelect::new("Select repositories to add to catalog:", repo_names)
            .prompt()
            .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

        for sel in &selected {
            if let Some(repo) = repos.iter().find(|r| r.full_name == *sel) {
                let service = ServiceCatalog {
                    id: repo.name.clone(),
                    name: repo.name.clone(),
                    kind: "service".to_string(),
                    description: repo
                        .description
                        .clone()
                        .unwrap_or_else(|| format!("Service for {}", repo.name)),
                    team: "platform".to_string(),
                    products: vec![],
                    repo: CatalogRepo {
                        provider: "github".to_string(),
                        owner: repo.owner.clone(),
                        name: repo.name.clone(),
                        url: repo.ssh_url.clone(),
                        default_branch: repo.default_branch.clone(),
                    },
                    owns: vec![],
                    likely_relevant_when: vec![],
                    commands: {
                        let mut map = HashMap::new();
                        map.insert("install".to_string(), "npm install".to_string());
                        map.insert("test".to_string(), "npm test".to_string());
                        map
                    },
                    issue_tracking: CatalogIssueTracking {
                        provider: ctx.config.issue_provider.r#type.clone(),
                        project: ctx
                            .config
                            .issue_provider
                            .default_project
                            .clone()
                            .unwrap_or_default(),
                        component: None,
                    },
                    docs: vec![CatalogDoc {
                        r#type: "readme".to_string(),
                        path: "README.md".to_string(),
                    }],
                    understand_anything: None,
                    deploy: None,
                };
                ws_catalog::add_service(root, &service)?;
                println!(
                    "✓ Added service {} to catalog/services/{}.yaml",
                    repo.name, repo.name
                );
            }
        }

        let next = Confirm::new(&format!("Show next {} repositories?", limit_val))
            .with_default(false)
            .prompt()
            .map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
        if !next {
            break;
        }
        page += 1;
    }

    Ok(())
}

async fn handle_add(
    root: &Path,
    ctx: &CommandContext,
    add_sub: AddSub,
) -> Result<(), WorkspaceError> {
    match add_sub {
        AddSub::Repo { name } => {
            let code_provider = ctx.code_provider.as_ref().ok_or_else(|| {
                WorkspaceError::Config("No code provider configured for addition".to_string())
            })?;
            let parts: Vec<&str> = name.split('/').collect();
            let (owner, repo_name) = if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                let default_owner =
                    ctx.config
                        .code_provider
                        .default_owner
                        .clone()
                        .ok_or_else(|| {
                            WorkspaceError::Validation(
                            "Specify full name <owner>/<repo> or set default code-owner config."
                                .to_string(),
                        )
                        })?;
                (default_owner, name)
            };

            println!("Resolving repository info for {}/{}...", owner, repo_name);
            let details = code_provider
                .get_repo(ws_core::models::RepoRef {
                    owner: owner.clone(),
                    name: repo_name.clone(),
                })
                .await?;

            let service = ServiceCatalog {
                id: details.summary.name.clone(),
                name: details.summary.name.clone(),
                kind: "service".to_string(),
                description: details
                    .summary
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("Service for {}", details.summary.name)),
                team: "platform".to_string(),
                products: vec![],
                repo: CatalogRepo {
                    provider: "github".to_string(),
                    owner: details.summary.owner.clone(),
                    name: details.summary.name.clone(),
                    url: details.summary.ssh_url.clone(),
                    default_branch: details.summary.default_branch.clone(),
                },
                owns: vec![],
                likely_relevant_when: vec![],
                commands: {
                    let mut map = HashMap::new();
                    map.insert("install".to_string(), "npm install".to_string());
                    map.insert("test".to_string(), "npm test".to_string());
                    map
                },
                issue_tracking: CatalogIssueTracking {
                    provider: ctx.config.issue_provider.r#type.clone(),
                    project: ctx
                        .config
                        .issue_provider
                        .default_project
                        .clone()
                        .unwrap_or_default(),
                    component: None,
                },
                docs: vec![CatalogDoc {
                    r#type: "readme".to_string(),
                    path: "README.md".to_string(),
                }],
                understand_anything: None,
                deploy: None,
            };
            ws_catalog::add_service(root, &service)?;
            println!(
                "✓ Added service {} to catalog/services/{}.yaml",
                details.summary.name, details.summary.name
            );
        }
        AddSub::Product { name } => {
            let product = ProductCatalog {
                id: name.to_lowercase(),
                name: name.clone(),
                kind: "product".to_string(),
                description: format!("Product workspace for {}", name),
                agent: ProductAgent {
                    name: format!("{} Agent", name),
                    instructions: format!("Orchestrate workflows related to {}", name),
                },
                knowledge_sources: vec![],
                services: ProductServices {
                    primary: vec![],
                    related: vec![],
                },
                routing_rules: vec![],
            };
            ws_catalog::add_product(root, &product)?;
            println!(
                "✓ Added product {} to catalog/products/{}.yaml",
                name, product.id
            );
        }
        AddSub::Team { name } => {
            let team = TeamCatalog {
                id: name.to_lowercase(),
                name: name.clone(),
                kind: "team".to_string(),
                description: format!("Team {}", name),
                lead: None,
                members: vec![],
            };
            ws_catalog::add_team(root, &team)?;
            println!("✓ Added team {} to catalog/teams/{}.yaml", name, team.id);
        }
    }
    Ok(())
}

async fn handle_open(
    ctx: CommandContext,
    q: String,
    editor: Option<String>,
    service: Option<String>,
) -> Result<(), WorkspaceError> {
    // Lookup is content-based: the folder name is a derived detail.
    let entry = ws_workspace::resolve_workspace(&ctx.workspace_root, &q)?;
    let cmd = EditorOpenCommand;
    let input = ws_editors::EditorOpenInput {
        epic_key: entry.folder,
        service_id: service,
        editor,
    };
    cmd.run(ctx, input).await?;
    Ok(())
}

async fn handle_status(
    root: &Path,
    ctx: CommandContext,
    epic_key: Option<String>,
) -> Result<(), WorkspaceError> {
    match epic_key {
        None => {
            println!("Configuration status:");
            println!("  Code provider: {}", ctx.config.code_provider.r#type);
            println!("  Issue provider: {}", ctx.config.issue_provider.r#type);
            if let Some(doc) = &ctx.config.doc_provider {
                println!("  Doc provider:   {}", doc.r#type);
            }
            println!("  Default editor: {}\n", ctx.config.editor.default);

            let services = ws_catalog::list_services(root)?;
            let products = ws_catalog::list_products(root)?;
            let teams = ws_catalog::list_teams(root)?;
            println!("Catalog database status:");
            println!("  Services: {} registered", services.len());
            println!("  Products: {} registered", products.len());
            println!("  Teams: {} registered\n", teams.len());

            println!("Local active workspaces:");
            let ws_entries = ws_workspace::list_workspaces(root)?;
            if ws_entries.is_empty() {
                println!("  *(None found)*");
            } else {
                for e in &ws_entries {
                    let ticket = e.ws.ticket.as_deref().unwrap_or("unticketed");
                    println!(
                        "  - {}  [{}]  {}",
                        e.folder,
                        ticket,
                        truncate(&e.ws.description, 48)
                    );
                }
            }
        }
        Some(key) => {
            let cmd = WorkspaceStatusCommand;
            let output = cmd
                .run(ctx, ws_workspace::WorkspaceQueryInput { q: key })
                .await?;
            println!("Workspace {} ({})", output.id, output.folder);
            println!(
                "  ticket:      {}",
                output.ticket.as_deref().unwrap_or("unticketed")
            );
            if !output.title.is_empty() {
                println!("  title:       {}", output.title);
            }
            if !output.description.is_empty() {
                println!("  description: {}", output.description);
            }
            println!("  Base branch:     {}", output.base_branch);
            println!("  Created branches: {}", output.create_branches);
            println!("  Preferred editor: {}", output.editor);
            println!("\nRepository worktree details:");
            for (service_id, status) in output.repo_statuses {
                println!("  - service: {}", service_id);
                println!("    branch:  {}", status.branch);
                println!("    current: {}", status.current_commit);
                println!(
                    "    unpushed: {}",
                    status
                        .unpushed_count
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "n/a".to_string())
                );
                println!(
                    "    changes: {}",
                    if status.has_changes {
                        "Yes (uncommitted files)"
                    } else {
                        "No"
                    }
                );
            }
            if !output.tasks.is_empty() {
                println!("\nTask worktrees:");
                for task in output.tasks {
                    println!("  - task: {} ({}) on {}", task.key, task.slug, task.branch);
                    for (service_id, status) in task.repo_statuses {
                        println!(
                            "      - service: {} | branch: {} | changes: {}",
                            service_id,
                            status.branch,
                            if status.has_changes { "yes" } else { "no" }
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_pr(ctx: CommandContext, pr_sub: PrSub) -> Result<(), WorkspaceError> {
    match pr_sub {
        PrSub::Create {
            epic_key,
            service,
            all,
            draft,
        } => {
            let services = if let Some(s) = service {
                vec![s]
            } else if all {
                let entry = ws_workspace::resolve_workspace(&ctx.workspace_root, &epic_key)?;
                let ws_path = ctx
                    .workspace_root
                    .join("workspaces")
                    .join(&entry.folder)
                    .join("workspace.yaml");
                if !ws_path.exists() {
                    return Err(WorkspaceError::NotFound(format!(
                        "Workspace {} not found",
                        epic_key
                    )));
                }
                let content = fs::read_to_string(ws_path)?;
                let ws: Workspace = serde_yaml::from_str(&content)?;
                ws.services
            } else {
                return Err(WorkspaceError::Validation(
                    "Specify --service <name> or --all to create Pull Requests.".to_string(),
                ));
            };

            println!("Creating Pull Requests for epic {}...", epic_key);
            let cmd = PrCreateCommand;
            let output = cmd
                .run(
                    ctx,
                    ws_providers::PrCreateInput {
                        workspace_id: epic_key,
                        services,
                        title: format!("[PR] Work for epic"),
                        body: "Pull request generated automatically via AI Workspace CLI."
                            .to_string(),
                        draft,
                    },
                )
                .await?;

            println!("\nPull requests successfully created:");
            for (svc_id, url) in output.prs {
                println!("  {}: {}", svc_id, url);
            }
        }
    }
    Ok(())
}

// ==========================================
// ws tasks — list / show / open (the resume CLI)
// ==========================================

#[derive(Clone)]
struct RowBranch {
    service: String,
    branch: String,
    dirty: bool,
}

#[derive(Clone)]
struct TaskRow {
    folder: String,
    id: String,
    ticket: Option<String>,
    description: String,
    /// `Some` → this row is a task slice; `None` → the workspace itself.
    task_slug: Option<String>,
    task_key: Option<String>,
    repos: Vec<String>,
    branches: Vec<RowBranch>,
    age: Option<std::time::SystemTime>,
}

fn build_task_rows(root: &Path) -> Result<Vec<TaskRow>, WorkspaceError> {
    let entries = ws_workspace::list_workspaces(root)?;
    let mut rows = Vec::new();

    for e in &entries {
        let lock = ws_workspace::load_workspace_lock_or_default(root, &e.folder, &e.ws);
        let repo_statuses =
            ws_workspace::collect_repo_statuses(root, &e.folder, &e.ws, &lock.repos);
        let task_statuses =
            ws_workspace::collect_task_statuses(root, &e.folder, &e.ws, &lock.tasks);
        let age = workspace_age(root, &e.folder);

        let branches: Vec<RowBranch> =
            e.ws.services
                .iter()
                .map(|svc| {
                    let rs = repo_statuses.get(svc);
                    RowBranch {
                        service: svc.clone(),
                        branch: rs
                            .map(|r| r.branch.clone())
                            .unwrap_or_else(|| "?".to_string()),
                        dirty: rs.map(|r| r.has_changes).unwrap_or(false),
                    }
                })
                .collect();

        rows.push(TaskRow {
            folder: e.folder.clone(),
            id: e.ws.id.clone(),
            ticket: e.ws.ticket.clone(),
            description: e.ws.description.clone(),
            task_slug: None,
            task_key: None,
            repos: e.ws.services.clone(),
            branches,
            age,
        });

        for ts in task_statuses {
            let branches: Vec<RowBranch> = ts
                .repo_statuses
                .iter()
                .map(|(svc, rs)| RowBranch {
                    service: svc.clone(),
                    branch: rs.branch.clone(),
                    dirty: rs.has_changes,
                })
                .collect();
            let mut repos: Vec<String> = ts.repo_statuses.keys().cloned().collect();
            repos.sort();
            rows.push(TaskRow {
                folder: e.folder.clone(),
                id: e.ws.id.clone(),
                ticket: e.ws.ticket.clone(),
                description: e.ws.description.clone(),
                task_slug: Some(ts.slug.clone()),
                task_key: Some(ts.key.clone()),
                repos,
                branches,
                age,
            });
        }
    }
    Ok(rows)
}

fn workspace_age(root: &Path, folder: &str) -> Option<std::time::SystemTime> {
    let p = ws_workspace::get_workspace_dir(root, folder).join("workspace.yaml");
    fs::metadata(p).and_then(|m| m.modified()).ok()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(3)).collect();
        out.push_str("...");
        out
    }
}

fn fmt_age(modified: std::time::SystemTime) -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(modified) else {
        return "now".to_string();
    };
    let secs = now.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn row_line(r: &TaskRow, index: Option<usize>) -> String {
    let token = match (&r.task_slug, &r.task_key) {
        (Some(s), Some(k)) => format!("{}/{} (task {})", r.id, s, k),
        _ => r.id.clone(),
    };
    let ticket = r.ticket.as_deref().unwrap_or("unticketed");
    let repos = r
        .branches
        .iter()
        .map(|rb| {
            format!(
                "{}:{}{}",
                rb.service,
                rb.branch,
                if rb.dirty { "*" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let desc = truncate(&r.description, 44);
    let age = r.age.map(fmt_age).unwrap_or_else(|| "?".to_string());
    match index {
        Some(i) => format!(
            "[{}] {} | {} | {} | {} | {}",
            i, token, ticket, repos, desc, age
        ),
        None => format!("{} | {} | {} | {} | {}", token, ticket, repos, desc, age),
    }
}

fn rows_empty_message(root: &Path) {
    println!(
        "No open feature workspaces found under {}.",
        root.join("workspaces").display()
    );
    println!(
        "Start one with: ws ai run workspace.create --input '{{\"title\": \"...\", \"description\": \"...\", \"services\": [...]}}'"
    );
}

fn print_rows(rows: &[TaskRow], root: &Path) {
    if rows.is_empty() {
        rows_empty_message(root);
        return;
    }
    for (i, r) in rows.iter().enumerate() {
        println!("{}", row_line(r, Some(i + 1)));
    }
}

fn desc_matches(desc: &str, token_lc: &str) -> bool {
    let t = token_lc.trim();
    if t.is_empty() {
        return false;
    }
    // Word or substring: the token appears inside any alphanumeric word.
    desc.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w.contains(t))
}

/// Semantic token matching (spec 4.3): index, id (slug), ticket, folder name,
/// folder prefix, task slug/key/full, then description word match.
///
/// This is the CLI-facing *superset* of the engine's `resolve_workspace`
/// (which only does identity + `{q}-` prefix over folders). It additionally
/// understands list indices and task rows and is the primary resume surface.
fn match_rows(rows: &[TaskRow], token: &str) -> Result<Vec<usize>, WorkspaceError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(WorkspaceError::Validation(
            "Empty ws tasks token.".to_string(),
        ));
    }

    // 0. 1-based index into the list (the `[N]` prefix).
    if let Ok(n) = token.parse::<usize>() {
        if n >= 1 && n <= rows.len() {
            return Ok(vec![n - 1]);
        }
    }

    let ws_rows: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.task_slug.is_none())
        .map(|(i, _)| i)
        .collect();

    // 1. exact id (slug)
    let bucket: Vec<usize> = ws_rows
        .iter()
        .copied()
        .filter(|&i| rows[i].id == token)
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }
    // 2. exact ticket
    let bucket: Vec<usize> = ws_rows
        .iter()
        .copied()
        .filter(|&i| rows[i].ticket.as_deref() == Some(token))
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }
    // 3. exact folder name
    let bucket: Vec<usize> = ws_rows
        .iter()
        .copied()
        .filter(|&i| rows[i].folder == token)
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }
    // 4. folder prefix `{token}-`
    let bucket: Vec<usize> = ws_rows
        .iter()
        .copied()
        .filter(|&i| rows[i].folder.starts_with(&format!("{}-", token)))
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }
    // 5. task rows: `id/slug`, `slug`, or `key`
    let bucket: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.task_slug.as_deref().is_some_and(|s| {
                let full = format!("{}/{}", r.id, s);
                s == token || r.task_key.as_deref() == Some(token) || full == token
            })
        })
        .map(|(i, _)| i)
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }
    // 6. description word match (workspace rows only)
    let token_lc = token.to_lowercase();
    let bucket: Vec<usize> = ws_rows
        .iter()
        .copied()
        .filter(|&i| desc_matches(&rows[i].description, &token_lc))
        .collect();
    if !bucket.is_empty() {
        return Ok(bucket);
    }

    Err(WorkspaceError::NotFound(format!(
        "Nothing matches '{}' — list open tasks with `ws tasks`.",
        token
    )))
}

fn pick_row(rows: &[TaskRow], token: &str) -> Result<usize, WorkspaceError> {
    let matches = match_rows(rows, token)?;
    if matches.len() == 1 {
        return Ok(matches[0]);
    }
    eprintln!("Token '{}' matched {} candidates:", token, matches.len());
    for &i in &matches {
        eprintln!("  {}", row_line(&rows[i], Some(i + 1)));
    }
    if std::io::stdin().is_terminal() {
        let options: Vec<String> = matches
            .iter()
            .map(|&i| row_line(&rows[i], Some(i + 1)))
            .collect();
        if let Ok(sel) = Select::new("Pick one workspace:", options).prompt() {
            for &i in &matches {
                if row_line(&rows[i], Some(i + 1)) == sel {
                    return Ok(i);
                }
            }
        }
    }
    Err(WorkspaceError::Validation(format!(
        "Token '{}' is ambiguous — re-run with a full slug, ticket, or the index above.",
        token
    )))
}

fn open_workspace_path(root: &Path, row: &TaskRow) -> String {
    let ws_dir = ws_workspace::get_workspace_dir(root, &row.folder);
    match &row.task_slug {
        Some(slug) => {
            // Q9: a task token targets tasks/<slug>/<repo>.
            let first = row.repos.first().cloned().unwrap_or_default();
            ws_dir
                .join("tasks")
                .join(slug)
                .join(first)
                .to_string_lossy()
                .into_owned()
        }
        None => match row.repos.first() {
            // Q9: `open` targets the main repos/<repo>; fall back to the folder when empty.
            Some(first) => ws_dir
                .join("repos")
                .join(first)
                .to_string_lossy()
                .into_owned(),
            None => ws_dir.to_string_lossy().into_owned(),
        },
    }
}

fn shorten_sha(s: &str) -> String {
    if s.chars().count() > 8 {
        s.chars().take(8).collect()
    } else {
        s.to_string()
    }
}

fn show_workspace_detail(root: &Path, row: &TaskRow) -> Result<(), WorkspaceError> {
    let e = ws_workspace::resolve_workspace(root, &row.folder)?;
    let lock = ws_workspace::load_workspace_lock_or_default(root, &e.folder, &e.ws);
    let repo_statuses = ws_workspace::collect_repo_statuses(root, &e.folder, &e.ws, &lock.repos);
    let task_statuses = ws_workspace::collect_task_statuses(root, &e.folder, &e.ws, &lock.tasks);
    let path = ws_workspace::get_workspace_dir(root, &e.folder);
    let age = row.age.map(fmt_age).unwrap_or_else(|| "?".to_string());

    println!("Workspace: {}", e.ws.id);
    if !e.ws.title.is_empty() {
        println!("  title:        {}", e.ws.title);
    }
    println!(
        "  ticket:       {}",
        e.ws.ticket.as_deref().unwrap_or("unticketed")
    );
    println!("  folder:       {}", e.folder);
    println!("  path:         {}", path.display());
    if !e.ws.description.is_empty() {
        println!("  description:  {}", e.ws.description);
    }
    println!(
        "  created:      {}",
        e.ws.created_at.as_deref().unwrap_or("n/a")
    );
    println!("  last changed: {}", age);
    println!("  base_branch:  {}", e.ws.base_branch);
    println!("  services:");
    for svc in &e.ws.services {
        let rs = repo_statuses.get(svc);
        let br = rs
            .map(|r| r.branch.clone())
            .unwrap_or_else(|| "?".to_string());
        let dirty = rs.map(|r| r.has_changes).unwrap_or(false);
        let unpushed = rs
            .and_then(|r| r.unpushed_count)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let baseline = rs
            .map(|r| shorten_sha(&r.baseline_commit))
            .unwrap_or_else(|| "unknown".to_string());
        println!(
            "    {}: path {}/repos/{} | branch {} | baseline {} | dirty {} | unpushed {}",
            svc,
            path.display(),
            svc,
            br,
            baseline,
            if dirty { "yes" } else { "no" },
            unpushed
        );
    }
    if !task_statuses.is_empty() {
        println!("  tasks:");
        for t in &task_statuses {
            println!("    {} ({}): branch {}", t.key, t.slug, t.branch);
            for (svc, rs) in &t.repo_statuses {
                println!(
                    "      {}: branch {} {}",
                    svc,
                    rs.branch,
                    if rs.has_changes { "(dirty)" } else { "" }
                );
            }
        }
    }
    if let Some(slug) = &row.task_slug {
        println!("  selected task: {}", slug);
    }
    Ok(())
}

async fn handle_tasks(root: &Path, args: TasksArgs) -> Result<(), WorkspaceError> {
    let rows = build_task_rows(root)?;
    match &args.sub {
        None => match &args.token {
            None => print_rows(&rows, root),
            Some(token) => {
                let idx = pick_row(&rows, token)?;
                println!("{}", open_workspace_path(root, &rows[idx]));
            }
        },
        Some(TasksSub::List) => print_rows(&rows, root),
        Some(TasksSub::Show { token }) => {
            let idx = pick_row(&rows, token)?;
            show_workspace_detail(root, &rows[idx])?;
        }
        Some(TasksSub::Open { token }) => {
            let idx = pick_row(&rows, token)?;
            println!("{}", open_workspace_path(root, &rows[idx]));
        }
    }
    Ok(())
}

async fn handle_attach(ctx: CommandContext, args: AttachArgs) -> Result<(), WorkspaceError> {
    let cmd = ws_workspace::WorkspaceAttachCommand;
    let output = cmd
        .run(
            ctx,
            ws_workspace::WorkspaceAttachInput {
                q: args.q,
                ticket: args.ticket,
            },
        )
        .await?;
    println!(
        "Attached {} to {} (folder {}).",
        output.id, output.ticket, output.folder
    );
    println!("{}", output.path);
    Ok(())
}

async fn handle_ai(
    ctx: CommandContext,
    registry: &CommandRegistry,
    ai_sub: AiSub,
) -> Result<(), WorkspaceError> {
    match ai_sub {
        AiSub::Manifest => {
            let mut command_list = Vec::new();
            for cmd in registry.list() {
                command_list.push(serde_json::json!({
                    "id": cmd.id(),
                    "description": cmd.description(),
                    "input_schema_command": format!("ws ai schema command {} input", cmd.id()),
                    "output_schema_command": format!("ws ai schema command {} output", cmd.id()),
                }));
            }
            let manifest = serde_json::json!({
                "version": "0.1.0",
                "commands": command_list,
            });
            let pretty = serde_json::to_string_pretty(&manifest)?;
            println!("{}", pretty);
        }
        AiSub::Docs { docs_sub } => match docs_sub {
            AiDocsSub::Generate => {
                let docs_md = ws_ai_docs::generate_command_docs(registry);
                let docs_dir = ctx.workspace_root.join("docs");
                fs::create_dir_all(&docs_dir)?;
                let path = docs_dir.join("command-api.md");
                fs::write(&path, docs_md)?;
                println!("✓ Documentation generated at {}", path.display());
            }
        },
        AiSub::Schema { command_id, kind } => {
            let cmd = registry.get(&command_id).ok_or_else(|| {
                WorkspaceError::NotFound(format!("Command '{}' not found", command_id))
            })?;
            let schema = if kind == "input" {
                cmd.input_schema()
            } else if kind == "output" {
                cmd.output_schema()
            } else {
                return Err(WorkspaceError::Validation(
                    "Specify 'input' or 'output' schema kind.".to_string(),
                ));
            };
            let pretty = serde_json::to_string_pretty(&schema)?;
            println!("{}", pretty);
        }
        AiSub::Run { command_id, input } => {
            let cmd = registry.get(&command_id).ok_or_else(|| {
                WorkspaceError::NotFound(format!("Command '{}' not found", command_id))
            })?;

            if !input.exists() {
                return Err(WorkspaceError::NotFound(format!(
                    "Input file '{}' not found",
                    input.display()
                )));
            }

            let input_content = fs::read_to_string(input)?;
            let input_json: serde_json::Value = serde_json::from_str(&input_content)
                .map_err(|e| WorkspaceError::Validation(format!("Invalid input JSON: {}", e)))?;

            println!("Running AI command '{}'...", command_id);
            let output_json = cmd.run_erased(ctx, input_json).await?;
            let pretty = serde_json::to_string_pretty(&output_json)?;
            println!("{}", pretty);
        }
    }
    Ok(())
}

#[cfg(test)]
mod kb_cli_tests {
    use super::*;
    use std::fs as stdfs;
    use tempfile::TempDir;

    /// Smoke test: `ws kb init` dispatches to ws-kb::scaffold and writes the
    /// complete knowledge-base tree. The CLI is a thin wrapper over the
    /// library (tested in ws-kb); this only verifies dispatch + that a real
    /// KB tree lands on disk under <root>/catalog/knowledge/.
    #[test]
    fn ws_kb_init_scaffolds_the_tree() {
        let tmp = TempDir::new().unwrap();
        handle_kb(tmp.path(), KbSub::Init { reset: None }).unwrap();

        let kb_root = tmp.path().join("catalog").join("knowledge");
        assert!(
            kb_root.is_dir(),
            "catalog/knowledge/ should exist after ws kb init"
        );
        assert!(
            kb_root.join("SCHEMA.md").is_file(),
            "SCHEMA.md should be scaffolded"
        );
        assert!(
            kb_root.join("wiki").join("index.md").is_file(),
            "wiki/index.md should be scaffolded"
        );
    }

    /// The `Kb::Init` subcommand parses from CLI args the same way other
    /// subcommands do — guards against accidental breakage of the clap wiring.
    #[test]
    fn kb_init_subcommand_parses() {
        let cli = Cli::try_parse_from(["ws", "kb", "init"]);
        assert!(cli.is_ok(), "ws kb init should parse: {:?}", cli.err());
        assert!(matches!(
            cli.unwrap().command,
            Commands::Kb {
                kb_sub: KbSub::Init { reset: None }
            }
        ));
    }

    #[test]
    fn kb_init_reset_subcommand_parses() {
        let cli = Cli::try_parse_from(["ws", "kb", "init", "--reset", "SCHEMA.md"]);
        assert!(cli.is_ok(), "ws kb init --reset SCHEMA.md should parse");
        assert!(matches!(
            cli.unwrap().command,
            Commands::Kb {
                kb_sub: KbSub::Init {
                    reset: Some(ref s)
                }
            } if s == "SCHEMA.md"
        ));
    }

    #[test]
    fn ws_kb_init_reset_refreshes_named_asset() {
        let tmp = TempDir::new().unwrap();
        handle_kb(tmp.path(), KbSub::Init { reset: None }).unwrap();
        let kb_root = tmp.path().join("catalog").join("knowledge");

        let target = "SCHEMA.md";
        let target_path = kb_root.join(target);
        let mutant = b"# mutated schema\n";
        stdfs::write(&target_path, mutant).unwrap();

        handle_kb(
            tmp.path(),
            KbSub::Init {
                reset: Some(target.to_string()),
            },
        )
        .unwrap();

        let embedded_bytes = ws_kb::KB_ASSETS.get_file(target).unwrap().contents();
        assert_eq!(
            stdfs::read(&target_path).unwrap().as_slice(),
            embedded_bytes,
            "reset should rewrite SCHEMA.md with embedded bytes"
        );
    }

    /// `ws --version` should be accepted by clap as the built-in version flag,
    /// which short-circuits printing `ws <CARGO_PKG_VERSION>`. We assert the
    /// flag is recognized (parse produces a clap DisplayVersion error) rather
    /// than a normal successful parse, guarding against removal of the
    /// `version` attribute on the `Cli` struct.
    #[test]
    fn version_flag_short_circuits() {
        let cli = Cli::try_parse_from(["ws", "--version"]);
        let err = cli.expect_err("--version should short-circuit, not parse OK");
        assert!(
            err.kind() == clap::error::ErrorKind::DisplayVersion,
            "expected DisplayVersion error, got {:?}",
            err.kind()
        );
    }

    #[test]
    fn ws_kb_init_reset_unknown_name_errors_with_valid_list() {
        let tmp = TempDir::new().unwrap();
        let err = handle_kb(
            tmp.path(),
            KbSub::Init {
                reset: Some("bogus".to_string()),
            },
        )
        .unwrap_err();

        let msg = err.to_string();
        assert!(
            msg.contains("bogus"),
            "error should mention the requested asset name: {msg}"
        );
        assert!(
            msg.contains("SCHEMA.md"),
            "error should list a valid asset name: {msg}"
        );
    }

    /// `ws dev-install` / `dev-uninstall` / `dev-purge` all wire into the clap
    /// command tree as top-level hyphenated subcommands — guards against
    /// accidental removal of the wiring.
    #[test]
    fn dev_subcommands_parse() {
        let install = Cli::try_parse_from([
            "ws",
            "dev-install",
            "https://github.com/Nestyko/workspace/pull/14",
        ]);
        assert!(
            install.is_ok(),
            "ws dev-install <url> should parse: {:?}",
            install.err()
        );
        match install.unwrap().command {
            Commands::DevInstall(args) => {
                assert_eq!(args.pr_url, "https://github.com/Nestyko/workspace/pull/14");
                assert_eq!(args.name, None);
                assert!(!args.force);
            }
            other => panic!("expected DevInstall, got {other:?}"),
        }

        let install_named = Cli::try_parse_from([
            "ws",
            "dev-install",
            "https://github.com/Nestyko/workspace/pull/14",
            "--name",
            "ws-pr14",
            "--force",
        ]);
        assert!(install_named.is_ok());
        if let Commands::DevInstall(args) = install_named.unwrap().command {
            assert_eq!(args.name.as_deref(), Some("ws-pr14"));
            assert!(args.force);
        }

        let uninstall = Cli::try_parse_from([
            "ws",
            "dev-uninstall",
            "https://github.com/Nestyko/workspace/pull/14",
        ]);
        assert!(uninstall.is_ok());
        assert!(matches!(
            uninstall.unwrap().command,
            Commands::DevUninstall(_)
        ));

        let purge = Cli::try_parse_from(["ws", "dev-purge"]);
        assert!(purge.is_ok());
        assert!(matches!(purge.unwrap().command, Commands::DevPurge(_)));
    }

    /// `ws dev-install --name ws` must be rejected before any git/cargo work —
    /// we never clobber the stable `ws` binary.
    #[test]
    fn dev_install_refuses_to_clobber_stable_ws() {
        let err = ws_dev::dev_install(ws_dev::DevInstallInput {
            pr_url: "https://github.com/Nestyko/workspace/pull/14".to_string(),
            name: Some("ws".to_string()),
            force: true,
        })
        .unwrap_err();
        assert!(err.to_string().contains("stable"), "got: {err}");
    }
}

#[cfg(test)]
mod skill_install_tests {
    use super::*;
    use tempfile::TempDir;

    /// `ws init` installs exactly the two curated skills (`ws-repo-init`,
    /// `ws-self-heal`) at the repo level, as real files under
    /// `.agents/skills/<name>/` and (on Unix) as relative symlinks under
    /// `.pi/skills/` and `.claude/skills/`.
    #[test]
    fn installs_curated_skills_at_repo_level() {
        let tmp = TempDir::new().unwrap();
        let installed = install_repo_skills(tmp.path(), INIT_SKILLS).unwrap();

        assert_eq!(
            installed,
            vec!["ws-repo-init".to_string(), "ws-self-heal".to_string()],
            "both curated skills should be installed and reported in order"
        );

        for name in INIT_SKILLS {
            let skill_md = tmp
                .path()
                .join(".agents")
                .join("skills")
                .join(name)
                .join("SKILL.md");
            assert!(
                skill_md.is_file(),
                ".agents/skills/{name}/SKILL.md should be written"
            );
            // The on-disk bytes must equal the embedded source bytes.
            let embedded = assets::SKILLS
                .get_file(format!("{name}/SKILL.md"))
                .expect("embedded skill file must exist");
            assert_eq!(
                std::fs::read(&skill_md).unwrap().as_slice(),
                embedded.contents(),
                "installed {name}/SKILL.md must match the embedded source"
            );

            #[cfg(unix)]
            {
                for harness_dir in [".pi/skills", ".claude/skills"] {
                    let link = tmp.path().join(harness_dir).join(name);
                    assert!(
                        link.is_symlink(),
                        "{harness_dir}/{name} should be a symlink"
                    );
                    assert_eq!(
                        std::fs::read_link(&link).unwrap().to_string_lossy(),
                        format!("../../.agents/skills/{name}"),
                        "symlink target must be the relative repo-relative path"
                    );
                    // Link must resolve to the real file.
                    assert!(
                        link.join("SKILL.md").is_file(),
                        "followed symlink should resolve to SKILL.md"
                    );
                }
            }
        }
    }

    /// A missing skill is skipped, not fatal — init must never fail just
    /// because a skill was renamed upstream.
    #[test]
    fn missing_skill_is_skipped_not_fatal() {
        let tmp = TempDir::new().unwrap();
        let installed =
            install_repo_skills(tmp.path(), &["ws-repo-init", "does-not-exist"]).unwrap();
        assert_eq!(
            installed,
            vec!["ws-repo-init".to_string()],
            "only the present skill should be reported"
        );
    }
}
