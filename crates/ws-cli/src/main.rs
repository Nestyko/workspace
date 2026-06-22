use clap::{Parser, Subcommand};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use std::fs;
use ws_core::command::{CommandRegistry, AiCommand};
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    CatalogDoc, CatalogIssueTracking, CatalogRepo, LocalConfig, ServiceCatalog,
    ProductCatalog, TeamCatalog, ProductAgent, ProductServices, Workspace,
};
use ws_core::providers::{CodeProvider, IssueProvider, DocProvider};
use ws_core::editors::EditorAdapter;

// Provider imports
use ws_provider_github_gh::GitHubGhProvider;
use ws_provider_jira::JiraProvider;
use ws_provider_jira::confluence::ConfluenceProvider;

// Editor imports
use ws_editors::{
    CursorEditorAdapter, EditorOpenCommand, VSCodeEditorAdapter, VimEditorAdapter, ZedEditorAdapter,
};

// Catalog imports
use ws_catalog::{
    CatalogProductAddCommand, CatalogProductGetCommand, CatalogProductListCommand,
    CatalogServiceAddCommand, CatalogServiceGetCommand, CatalogServiceListCommand,
    CatalogTeamAddCommand, CatalogTeamGetCommand, CatalogTeamListCommand, CatalogValidateCommand,
    ContextResolveCommand,
};

// Workspace imports
use ws_workspace::{
    WorkspaceAddServiceCommand, WorkspaceCreateCommand, WorkspaceGenerateEditorFilesCommand,
    WorkspaceLockCommand, WorkspaceStatusCommand,
};

// Provider command imports
use ws_providers::{
    PrCreateCommand, ProviderCodeCheckAuthCommand, ProviderCodeGetRepoCommand,
    ProviderCodeListRecentReposCommand, ProviderIssueCheckAuthCommand,
    ProviderIssueCommentCommand, ProviderIssueCreateEpicCommand, ProviderIssueCreateIssueCommand,
    ProviderIssueGetIssueCommand, ProviderIssueLinkCommand,
    ProviderDocCheckAuthCommand, ProviderDocGetPageCommand,
    ProviderDocCreatePageCommand, ProviderDocUpdatePageCommand,
    ProviderConfigGetInstructionsCommand,
};

#[derive(Parser, Clone, Debug)]
#[command(name = "ws")]
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
}

#[derive(clap::Args, Clone, Debug)]
struct DiscoverArgs {
    #[arg(long, help = "Limit the number of repositories fetched (default 10)")]
    limit: Option<usize>,
}

#[derive(clap::Args, Clone, Debug)]
struct OpenArgs {
    #[arg(help = "The Jira Epic key (e.g. COSELL-123)")]
    epic_key: String,

    #[arg(long, help = "Specify the editor (cursor, vscode, zed, vim)")]
    editor: Option<String>,

    #[arg(long, help = "Specify the service name to open directly")]
    service: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
struct StatusArgs {
    #[arg(help = "The Jira Epic key (e.g. COSELL-123)")]
    epic_key: Option<String>,
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
        #[arg(help = "The Jira Epic key (e.g. COSELL-123)")]
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
    let workspace_root = ws_config::find_workspace_root().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

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

    let issue_provider: Option<Arc<dyn IssueProvider>> = match config.issue_provider.r#type.as_str() {
        "jira" => Some(Arc::new(JiraProvider::new(
            config.issue_provider.base_url.clone(),
            config.issue_provider.default_project.clone(),
        ))),
        _ => None,
    };

    let doc_provider: Option<Arc<dyn DocProvider>> = config.doc_provider.as_ref().and_then(|doc_cfg| {
        match doc_cfg.r#type.as_str() {
            "confluence" => Some(Arc::new(ConfluenceProvider::new(
                doc_cfg.base_url.clone(),
                doc_cfg.default_space.clone(),
            )) as Arc<dyn DocProvider>),
            _ => None,
        }
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
    // Workspace commands
    registry.register(WorkspaceCreateCommand);
    registry.register(WorkspaceAddServiceCommand);
    registry.register(WorkspaceStatusCommand);
    registry.register(WorkspaceLockCommand);
    registry.register(WorkspaceGenerateEditorFilesCommand);
    // Editor command
    registry.register(EditorOpenCommand);

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
        Commands::Pr { pr_sub } => {
            handle_pr(ctx.clone(), pr_sub).await?;
        }
        Commands::Ai { ai_sub } => {
            handle_ai(ctx.clone(), registry, ai_sub).await?;
        }
    }
    Ok(())
}

async fn handle_init(root: &Path, ctx: &CommandContext) -> Result<(), WorkspaceError> {
    println!("Welcome to AI Workspace.\n");

    let issue_provider = Select::new("Select issue provider:", vec!["Jira", "Linear (coming soon)", "GitHub (coming soon)"]).prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
    if issue_provider != "Jira" {
        return Err(WorkspaceError::Other("Only Jira issue provider is currently supported.".to_string()));
    }

    let code_provider = Select::new("Select code provider:", vec!["GitHub via gh", "GitLab (coming soon)", "Bitbucket (coming soon)"]).prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
    if code_provider != "GitHub via gh" {
        return Err(WorkspaceError::Other("Only GitHub via gh is currently supported.".to_string()));
    }

    let default_editor = Select::new("Select default editor:", vec!["cursor", "vscode", "zed", "vim"]).prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let default_owner = Text::new("GitHub organization/user:").prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let jira_base_url = Text::new("Jira Base URL (e.g. https://example.atlassian.net):").prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let jira_project = Text::new("Jira Default Project Key:").prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let confluence_base_url = Text::new("Confluence Base URL (e.g. https://example.atlassian.net):").prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    let confluence_space = Text::new("Confluence Default Space Key:").prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    // Validate GH Auth
    println!("\nValidating GitHub auth through gh...");
    let gh_provider = GitHubGhProvider::new(Some(default_owner.clone()), Some("ssh".to_string()));
    let auth = gh_provider.check_auth().await?;
    if auth.authenticated {
        println!("✓ GitHub authentication successful. Username: {}", auth.username.unwrap_or_default());
    } else {
        println!("⚠ GitHub authentication failed: {}", auth.details.unwrap_or_default());
    }

    // Build configuration
    let mut new_config = LocalConfig::default();
    new_config.code_provider.default_owner = Some(default_owner);
    new_config.editor.default = default_editor.to_string();
    new_config.issue_provider.base_url = Some(jira_base_url);
    new_config.issue_provider.default_project = Some(jira_project);
    new_config.doc_provider = Some(ws_core::models::DocProviderConfig {
        r#type: "confluence".to_string(),
        base_url: Some(confluence_base_url),
        default_space: Some(confluence_space),
    });

    ws_config::save_config(root, &new_config)?;
    println!("\nSaved config to {}", ws_config::get_config_path(root).display());

    // Create catalogs
    ws_catalog::ensure_catalog_dirs(root)?;

    // Create example catalog entries if empty
    let services_dir = ws_catalog::get_kind_dir(root, "services");
    if fs::read_dir(&services_dir).map(|d| d.count()).unwrap_or(0) == 0 {
        fs::create_dir_all(root.join("templates"))?;
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
  provider: jira
  project: PLATFORM
docs:
  - type: readme
    path: README.md
"#;
        fs::write(root.join("templates").join("service.yaml"), svc_template)?;

        let sample_product = ProductCatalog {
            id: "cosell".to_string(),
            name: "Cosell".to_string(),
            kind: "product".to_string(),
            description: "Partner collaboration and agent-assisted workflows.".to_string(),
            agent: ProductAgent {
                name: "Cosell Agent".to_string(),
                instructions: "Understand Cosell product context.".to_string(),
            },
            knowledge_sources: vec![],
            services: ProductServices {
                primary: vec![],
                related: vec![],
            },
            routing_rules: vec![],
        };
        ws_catalog::add_product(root, &sample_product)?;

        let sample_team = TeamCatalog {
            id: "platform".to_string(),
            name: "Platform Team".to_string(),
            kind: "team".to_string(),
            description: "Core infrastructure, templates, and libraries.".to_string(),
            lead: Some("alice".to_string()),
            members: vec!["bob".to_string(), "charlie".to_string()],
        };
        ws_catalog::add_team(root, &sample_team)?;
    }

    let start_discovery = Confirm::new("Would you like to discover repositories to add to the catalog?").with_default(true).prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;

    if start_discovery {
        let temp_ctx = CommandContext::new(
            new_config,
            root.to_path_buf(),
            None,
            Some(Arc::new(gh_provider)),
            None,
            HashMap::new(),
        );
        handle_discover(root, &temp_ctx, Some(10)).await?;
    }

    println!("\nAI Workspace initialized successfully!");
    Ok(())
}

fn handle_config(
    root: &Path,
    config: &LocalConfig,
    sub: Option<ConfigSub>,
) -> Result<(), WorkspaceError> {
    match sub {
        None | Some(ConfigSub::Get) => {
            println!("Config Path: {}", ws_config::get_config_path(root).display());
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
                    return Err(WorkspaceError::Config(format!("Unknown configuration key: {}", key)));
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

    let mut page = 1;
    loop {
        println!("Fetching updated repositories (Page {})...", page);
        let repos = code_provider.list_recent_repos(ws_core::models::ListRecentReposInput {
            limit,
            page: Some(page),
        }).await?;

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
                    description: repo.description.clone().unwrap_or_else(|| format!("Service for {}", repo.name)),
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
                        provider: "jira".to_string(),
                        project: ctx.config.issue_provider.default_project.clone().unwrap_or_else(|| "PLATFORM".to_string()),
                        component: None,
                    },
                    docs: vec![CatalogDoc {
                        r#type: "readme".to_string(),
                        path: "README.md".to_string(),
                    }],
                };
                ws_catalog::add_service(root, &service)?;
                println!("✓ Added service {} to catalog/services/{}.yaml", repo.name, repo.name);
            }
        }

        let next = Confirm::new("Show next 10 repositories?").with_default(false).prompt().map_err(|_| WorkspaceError::Other("Cancelled".to_string()))?;
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
                let default_owner = ctx.config.code_provider.default_owner.clone().ok_or_else(|| {
                    WorkspaceError::Validation("Specify full name <owner>/<repo> or set default code-owner config.".to_string())
                })?;
                (default_owner, name)
            };

            println!("Resolving repository info for {}/{}...", owner, repo_name);
            let details = code_provider.get_repo(ws_core::models::RepoRef {
                owner: owner.clone(),
                name: repo_name.clone(),
            }).await?;

            let service = ServiceCatalog {
                id: details.summary.name.clone(),
                name: details.summary.name.clone(),
                kind: "service".to_string(),
                description: details.summary.description.clone().unwrap_or_else(|| format!("Service for {}", details.summary.name)),
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
                    provider: "jira".to_string(),
                    project: ctx.config.issue_provider.default_project.clone().unwrap_or_else(|| "PLATFORM".to_string()),
                    component: None,
                },
                docs: vec![CatalogDoc {
                    r#type: "readme".to_string(),
                    path: "README.md".to_string(),
                }],
            };
            ws_catalog::add_service(root, &service)?;
            println!("✓ Added service {} to catalog/services/{}.yaml", details.summary.name, details.summary.name);
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
            println!("✓ Added product {} to catalog/products/{}.yaml", name, product.id);
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
    epic_key: String,
    editor: Option<String>,
    service: Option<String>,
) -> Result<(), WorkspaceError> {
    let cmd = EditorOpenCommand;
    let input = ws_editors::EditorOpenInput {
        epic_key,
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

            let workspaces_dir = root.join(&ctx.config.paths.workspaces_dir);
            println!("Local active workspaces:");
            if workspaces_dir.exists() {
                let mut found = false;
                for entry in fs::read_dir(workspaces_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            println!("  - {}", name);
                            found = true;
                        }
                    }
                }
                if !found {
                    println!("  *(None found)*");
                }
            } else {
                println!("  *(None found)*");
            }
        }
        Some(key) => {
            let cmd = WorkspaceStatusCommand;
            let output = cmd.run(ctx, ws_workspace::WorkspaceGetInput { epic_key: key }).await?;
            println!("Workspace status for epic {}:", output.epic_key);
            println!("  Base branch: {}", output.base_branch);
            println!("  Created branches: {}", output.create_branches);
            println!("  Preferred editor: {}", output.editor);
            println!("\nRepository worktree details:");
            for (service_id, status) in output.repo_statuses {
                println!("  - service: {}", service_id);
                println!("    branch:  {}", status.branch);
                println!("    current: {}", status.current_commit);
                println!("    changes: {}", if status.has_changes { "Yes (uncommitted files)" } else { "No" });
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
                let ws_path = ctx.workspace_root
                    .join("workspaces")
                    .join(&epic_key)
                    .join("workspace.yaml");
                if !ws_path.exists() {
                    return Err(WorkspaceError::NotFound(format!("Workspace {} not found", epic_key)));
                }
                let content = fs::read_to_string(ws_path)?;
                let ws: Workspace = serde_yaml::from_str(&content)?;
                ws.services
            } else {
                return Err(WorkspaceError::Validation("Specify --service <name> or --all to create Pull Requests.".to_string()));
            };

            println!("Creating Pull Requests for epic {}...", epic_key);
            let cmd = PrCreateCommand;
            let output = cmd.run(ctx, ws_providers::PrCreateInput {
                workspace_id: epic_key,
                services,
                title: format!("[PR] Work for epic"),
                body: "Pull request generated automatically via AI Workspace CLI.".to_string(),
                draft,
            }).await?;

            println!("\nPull requests successfully created:");
            for (svc_id, url) in output.prs {
                println!("  {}: {}", svc_id, url);
            }
        }
    }
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
        AiSub::Docs { docs_sub } => {
            match docs_sub {
                AiDocsSub::Generate => {
                    let docs_md = ws_ai_docs::generate_command_docs(registry);
                    let docs_dir = ctx.workspace_root.join("docs");
                    fs::create_dir_all(&docs_dir)?;
                    let path = docs_dir.join("command-api.md");
                    fs::write(&path, docs_md)?;
                    println!("✓ Documentation generated at {}", path.display());
                }
            }
        }
        AiSub::Schema { command_id, kind } => {
            let cmd = registry.get(&command_id).ok_or_else(|| {
                WorkspaceError::NotFound(format!("Command '{}' not found", command_id))
            })?;
            let schema = if kind == "input" {
                cmd.input_schema()
            } else if kind == "output" {
                cmd.output_schema()
            } else {
                return Err(WorkspaceError::Validation("Specify 'input' or 'output' schema kind.".to_string()));
            };
            let pretty = serde_json::to_string_pretty(&schema)?;
            println!("{}", pretty);
        }
        AiSub::Run { command_id, input } => {
            let cmd = registry.get(&command_id).ok_or_else(|| {
                WorkspaceError::NotFound(format!("Command '{}' not found", command_id))
            })?;

            if !input.exists() {
                return Err(WorkspaceError::NotFound(format!("Input file '{}' not found", input.display())));
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
