use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;
use ws_core::models::{
    CatalogDoc, CatalogIssueTracking, DeployConfig, KnowledgeCatalog, ProductCatalog,
    ProductKnowledgeSource, ServiceCatalog, TeamCatalog, UnderstandAnythingConfig,
};

pub fn get_catalog_dir(root: &Path) -> PathBuf {
    root.join("catalog")
}

pub fn get_kind_dir(root: &Path, kind: &str) -> PathBuf {
    get_catalog_dir(root).join(kind)
}

pub fn ensure_catalog_dirs(root: &Path) -> Result<(), WorkspaceError> {
    let base = get_catalog_dir(root);
    fs::create_dir_all(base.join("services"))?;
    fs::create_dir_all(base.join("products"))?;
    fs::create_dir_all(base.join("teams"))?;
    fs::create_dir_all(base.join("knowledge"))?;
    Ok(())
}

pub fn add_service(root: &Path, service: &ServiceCatalog) -> Result<(), WorkspaceError> {
    ensure_catalog_dirs(root)?;
    let path = get_kind_dir(root, "services").join(format!("{}.yaml", service.id));
    let content = serde_yaml::to_string(service)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_product(root: &Path, product: &ProductCatalog) -> Result<(), WorkspaceError> {
    ensure_catalog_dirs(root)?;
    let path = get_kind_dir(root, "products").join(format!("{}.yaml", product.id));
    let content = serde_yaml::to_string(product)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_team(root: &Path, team: &TeamCatalog) -> Result<(), WorkspaceError> {
    ensure_catalog_dirs(root)?;
    let path = get_kind_dir(root, "teams").join(format!("{}.yaml", team.id));
    let content = serde_yaml::to_string(team)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_knowledge(root: &Path, knowledge: &KnowledgeCatalog) -> Result<(), WorkspaceError> {
    ensure_catalog_dirs(root)?;
    let path = get_kind_dir(root, "knowledge").join(format!("{}.yaml", knowledge.id));
    let content = serde_yaml::to_string(knowledge)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn get_service(root: &Path, id: &str) -> Result<ServiceCatalog, WorkspaceError> {
    let path = get_kind_dir(root, "services").join(format!("{}.yaml", id));
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Service '{}' not found in catalog",
            id
        )));
    }
    let content = fs::read_to_string(path)?;
    let service: ServiceCatalog = serde_yaml::from_str(&content)?;
    Ok(service)
}

pub fn get_product(root: &Path, id: &str) -> Result<ProductCatalog, WorkspaceError> {
    let path = get_kind_dir(root, "products").join(format!("{}.yaml", id));
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Product '{}' not found in catalog",
            id
        )));
    }
    let content = fs::read_to_string(path)?;
    let product: ProductCatalog = serde_yaml::from_str(&content)?;
    Ok(product)
}

pub fn get_team(root: &Path, id: &str) -> Result<TeamCatalog, WorkspaceError> {
    let path = get_kind_dir(root, "teams").join(format!("{}.yaml", id));
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Team '{}' not found in catalog",
            id
        )));
    }
    let content = fs::read_to_string(path)?;
    let team: TeamCatalog = serde_yaml::from_str(&content)?;
    Ok(team)
}

pub fn get_knowledge(root: &Path, id: &str) -> Result<KnowledgeCatalog, WorkspaceError> {
    let path = get_kind_dir(root, "knowledge").join(format!("{}.yaml", id));
    if !path.exists() {
        return Err(WorkspaceError::NotFound(format!(
            "Knowledge source '{}' not found in catalog",
            id
        )));
    }
    let content = fs::read_to_string(path)?;
    let knowledge: KnowledgeCatalog = serde_yaml::from_str(&content)?;
    Ok(knowledge)
}

pub fn list_services(root: &Path) -> Result<Vec<ServiceCatalog>, WorkspaceError> {
    let dir = get_kind_dir(root, "services");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut services = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let content = fs::read_to_string(&path)?;
            if let Ok(service) = serde_yaml::from_str::<ServiceCatalog>(&content) {
                services.push(service);
            }
        }
    }
    Ok(services)
}

pub fn list_products(root: &Path) -> Result<Vec<ProductCatalog>, WorkspaceError> {
    let dir = get_kind_dir(root, "products");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut products = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let content = fs::read_to_string(&path)?;
            if let Ok(product) = serde_yaml::from_str::<ProductCatalog>(&content) {
                products.push(product);
            }
        }
    }
    Ok(products)
}

pub fn list_teams(root: &Path) -> Result<Vec<TeamCatalog>, WorkspaceError> {
    let dir = get_kind_dir(root, "teams");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut teams = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let content = fs::read_to_string(&path)?;
            if let Ok(team) = serde_yaml::from_str::<TeamCatalog>(&content) {
                teams.push(team);
            }
        }
    }
    Ok(teams)
}

pub fn list_knowledge(root: &Path) -> Result<Vec<KnowledgeCatalog>, WorkspaceError> {
    let dir = get_kind_dir(root, "knowledge");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == "yaml" || ext == "yml")
        {
            let content = fs::read_to_string(&path)?;
            if let Ok(item) = serde_yaml::from_str::<KnowledgeCatalog>(&content) {
                items.push(item);
            }
        }
    }
    Ok(items)
}

pub fn validate_catalog(root: &Path) -> Result<(), WorkspaceError> {
    let base = get_catalog_dir(root);
    if !base.exists() {
        return Ok(());
    }
    let service_dir = base.join("services");
    if service_dir.exists() {
        for entry in fs::read_dir(service_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                let content = fs::read_to_string(&path)?;
                serde_yaml::from_str::<ServiceCatalog>(&content).map_err(|e| {
                    WorkspaceError::Catalog(format!(
                        "Invalid service yaml in {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }
    }
    let product_dir = base.join("products");
    if product_dir.exists() {
        for entry in fs::read_dir(product_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                let content = fs::read_to_string(&path)?;
                serde_yaml::from_str::<ProductCatalog>(&content).map_err(|e| {
                    WorkspaceError::Catalog(format!(
                        "Invalid product yaml in {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }
    }
    let team_dir = base.join("teams");
    if team_dir.exists() {
        for entry in fs::read_dir(team_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                let content = fs::read_to_string(&path)?;
                serde_yaml::from_str::<TeamCatalog>(&content).map_err(|e| {
                    WorkspaceError::Catalog(format!(
                        "Invalid team yaml in {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }
    }
    let knowledge_dir = base.join("knowledge");
    if knowledge_dir.exists() {
        for entry in fs::read_dir(knowledge_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                let content = fs::read_to_string(&path)?;
                serde_yaml::from_str::<KnowledgeCatalog>(&content).map_err(|e| {
                    WorkspaceError::Catalog(format!(
                        "Invalid knowledge yaml in {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }
    }
    Ok(())
}

// ==========================================
// AI Command Implementations
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EmptyInput {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StatusOutput {
    pub success: bool,
    pub message: String,
}

pub struct CatalogValidateCommand;

#[async_trait]
impl AiCommand for CatalogValidateCommand {
    const ID: &'static str = "catalog.validate";
    const DESCRIPTION: &'static str = "Validate all catalog files.";
    type Input = EmptyInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        validate_catalog(&ctx.workspace_root)?;
        Ok(StatusOutput {
            success: true,
            message: "All catalog files are valid.".to_string(),
        })
    }
}

pub struct CatalogServiceAddCommand;

#[async_trait]
impl AiCommand for CatalogServiceAddCommand {
    const ID: &'static str = "catalog.service.add";
    const DESCRIPTION: &'static str = "Add a service to the catalog as a separate file.";
    type Input = ServiceCatalog;
    type Output = ServiceCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        add_service(&ctx.workspace_root, &input)?;
        Ok(input)
    }
}

// =========================================
// AI Command: catalog.service.update
// =========================================

/// Strict partial-merge patch for a service catalog entry.
///
/// Semantics (locked by design):
/// - `commands` is a map → **per-key merge** (set `commands.dev` without touching
///   `commands.test`).
/// - Every other present field is a **top-level replace**.
/// - `#[serde(deny_unknown_fields)]` makes unknown keys fail fast (strict mode).
/// - After writing, `validate_catalog` is re-run; the command errors if validation fails.
///
/// Note: catalog entries have no `locks.yaml` (those are per-epic workspace locks), so
/// no lockfile is touched here.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CatalogServiceUpdateInput {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owns: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likely_relevant_when: Option<Vec<String>>,
    /// Per-key merge into the existing `commands` map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_tracking: Option<CatalogIssueTracking>,
    /// Replace whole value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<Vec<CatalogDoc>>,
    /// Replace whole value (Point #1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub understand_anything: Option<UnderstandAnythingConfig>,
    /// Replace whole value (Point #8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy: Option<DeployConfig>,
}

pub struct CatalogServiceUpdateCommand;

#[async_trait]
impl AiCommand for CatalogServiceUpdateCommand {
    const ID: &'static str = "catalog.service.update";
    const DESCRIPTION: &'static str =
        "Strict partial-merge patch into a service catalog entry; re-validates the catalog.";
    type Input = CatalogServiceUpdateInput;
    type Output = ServiceCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let root = &ctx.workspace_root;
        let mut service = get_service(root, &input.id)?;

        // Top-level replaces.
        if let Some(name) = input.name {
            service.name = name;
        }
        if let Some(description) = input.description {
            service.description = description;
        }
        if let Some(team) = input.team {
            service.team = team;
        }
        if let Some(products) = input.products {
            service.products = products;
        }
        if let Some(owns) = input.owns {
            service.owns = owns;
        }
        if let Some(likely) = input.likely_relevant_when {
            service.likely_relevant_when = likely;
        }
        if let Some(issue_tracking) = input.issue_tracking {
            service.issue_tracking = issue_tracking;
        }
        if let Some(docs) = input.docs {
            service.docs = docs;
        }
        if let Some(understand_anything) = input.understand_anything {
            service.understand_anything = Some(understand_anything);
        }
        if let Some(deploy) = input.deploy {
            service.deploy = Some(deploy);
        }
        // `commands` is a map → per-key merge.
        if let Some(commands_patch) = input.commands {
            for (k, v) in commands_patch {
                service.commands.insert(k, v);
            }
        }

        add_service(root, &service)?;
        // Re-validate the whole catalog; fail fast on any drift.
        validate_catalog(root)?;
        Ok(service)
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CatalogGetInput {
    pub id: String,
}

pub struct CatalogServiceGetCommand;

#[async_trait]
impl AiCommand for CatalogServiceGetCommand {
    const ID: &'static str = "catalog.service.get";
    const DESCRIPTION: &'static str = "Retrieve a service catalog by ID.";
    type Input = CatalogGetInput;
    type Output = ServiceCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        get_service(&ctx.workspace_root, &input.id)
    }
}

pub struct CatalogServiceListCommand;

#[async_trait]
impl AiCommand for CatalogServiceListCommand {
    const ID: &'static str = "catalog.service.list";
    const DESCRIPTION: &'static str = "List all service catalogs.";
    type Input = EmptyInput;
    type Output = Vec<ServiceCatalog>;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        list_services(&ctx.workspace_root)
    }
}

pub struct CatalogProductAddCommand;

#[async_trait]
impl AiCommand for CatalogProductAddCommand {
    const ID: &'static str = "catalog.product.add";
    const DESCRIPTION: &'static str = "Add a product to the catalog.";
    type Input = ProductCatalog;
    type Output = ProductCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        add_product(&ctx.workspace_root, &input)?;
        Ok(input)
    }
}

pub struct CatalogProductGetCommand;

#[async_trait]
impl AiCommand for CatalogProductGetCommand {
    const ID: &'static str = "catalog.product.get";
    const DESCRIPTION: &'static str = "Retrieve a product catalog by ID.";
    type Input = CatalogGetInput;
    type Output = ProductCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        get_product(&ctx.workspace_root, &input.id)
    }
}

pub struct CatalogProductListCommand;

#[async_trait]
impl AiCommand for CatalogProductListCommand {
    const ID: &'static str = "catalog.product.list";
    const DESCRIPTION: &'static str = "List all product catalogs.";
    type Input = EmptyInput;
    type Output = Vec<ProductCatalog>;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        list_products(&ctx.workspace_root)
    }
}

pub struct CatalogTeamAddCommand;

#[async_trait]
impl AiCommand for CatalogTeamAddCommand {
    const ID: &'static str = "catalog.team.add";
    const DESCRIPTION: &'static str = "Add a team to the catalog.";
    type Input = TeamCatalog;
    type Output = TeamCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        add_team(&ctx.workspace_root, &input)?;
        Ok(input)
    }
}

pub struct CatalogTeamGetCommand;

#[async_trait]
impl AiCommand for CatalogTeamGetCommand {
    const ID: &'static str = "catalog.team.get";
    const DESCRIPTION: &'static str = "Retrieve a team catalog by ID.";
    type Input = CatalogGetInput;
    type Output = TeamCatalog;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        get_team(&ctx.workspace_root, &input.id)
    }
}

pub struct CatalogTeamListCommand;

#[async_trait]
impl AiCommand for CatalogTeamListCommand {
    const ID: &'static str = "catalog.team.list";
    const DESCRIPTION: &'static str = "List all team catalogs.";
    type Input = EmptyInput;
    type Output = Vec<TeamCatalog>;

    async fn run(
        &self,
        ctx: CommandContext,
        _input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        list_teams(&ctx.workspace_root)
    }
}

// ==========================================
// AI Command: context.resolve
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ContextResolveInput {
    pub query: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RecommendedService {
    pub id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ContextResolveOutput {
    pub query: String,
    pub products: Vec<String>,
    pub recommended_services: Vec<RecommendedService>,
    pub knowledge_sources: Vec<ProductKnowledgeSource>,
}

pub struct ContextResolveCommand;

#[async_trait]
impl AiCommand for ContextResolveCommand {
    const ID: &'static str = "context.resolve";
    const DESCRIPTION: &'static str =
        "Resolve products, recommended services, and knowledge sources based on a query.";
    type Input = ContextResolveInput;
    type Output = ContextResolveOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let query_lower = input.query.to_lowercase();
        let tokens: Vec<&str> = query_lower
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .collect();

        let products = list_products(&ctx.workspace_root)?;
        let services = list_services(&ctx.workspace_root)?;

        let mut matched_products = Vec::new();
        let mut recommended_services = Vec::new();
        let mut knowledge_sources = Vec::new();

        // Match products
        for p in products {
            let mut matched = false;
            if tokens.contains(&p.id.as_str()) || p.name.to_lowercase().contains(&query_lower) {
                matched = true;
            }
            for token in &tokens {
                if p.description.to_lowercase().contains(token) {
                    matched = true;
                    break;
                }
            }
            if matched {
                matched_products.push(p.id.clone());
                // Add its knowledge sources
                knowledge_sources.extend(p.knowledge_sources.clone());
            }
        }

        // Match services
        for s in services {
            let mut match_reason = None;

            if tokens.contains(&s.id.as_str()) || s.name.to_lowercase().contains(&query_lower) {
                match_reason = Some(format!("Service name or ID matches query."));
            } else {
                // Check likely_relevant_when
                for item in &s.likely_relevant_when {
                    let item_lower = item.to_lowercase();
                    let matches_count = tokens
                        .iter()
                        .filter(|t| !t.is_empty() && item_lower.contains(*t))
                        .count();
                    if matches_count >= 2
                        || (tokens.len() == 1
                            && !tokens[0].is_empty()
                            && item_lower.contains(tokens[0]))
                    {
                        match_reason = Some(format!("Matches typical scenario: '{}'", item));
                        break;
                    }
                }

                if match_reason.is_none() {
                    // Check owns
                    for item in &s.owns {
                        if query_lower.contains(&item.to_lowercase()) {
                            match_reason = Some(format!("Service owns feature: '{}'", item));
                            break;
                        }
                    }
                }

                if match_reason.is_none() {
                    // General description search
                    for token in &tokens {
                        if !token.is_empty() && s.description.to_lowercase().contains(token) {
                            match_reason =
                                Some(format!("Query contains description term: '{}'", token));
                            break;
                        }
                    }
                }
            }

            if let Some(reason) = match_reason {
                recommended_services.push(RecommendedService {
                    id: s.id.clone(),
                    reason,
                });
            }
        }

        Ok(ContextResolveOutput {
            query: input.query,
            products: matched_products,
            recommended_services,
            knowledge_sources,
        })
    }
}
