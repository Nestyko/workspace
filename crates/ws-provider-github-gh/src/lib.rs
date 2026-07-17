use async_trait::async_trait;
use duct::cmd;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use ws_core::error::WorkspaceError;
use ws_core::models::{
    AuthStatus, CreatePullRequestInput, CreateWorktreeInput, EnsureRepoCacheInput,
    ListRecentReposInput, PullRequest, PushBranchInput, RepoCache, RepoDetails, RepoRef,
    RepoSummary, Worktree,
};
use ws_core::providers::CodeProvider;

pub struct GitHubGhProvider {
    pub default_owner: Option<String>,
    pub protocol: String, // "ssh" or "https"
    #[cfg(test)]
    pub mock_runner:
        Option<std::sync::Arc<dyn Fn(&[&str]) -> Result<String, WorkspaceError> + Send + Sync>>,
}

impl GitHubGhProvider {
    pub fn new(default_owner: Option<String>, protocol: Option<String>) -> Self {
        Self {
            default_owner,
            protocol: protocol.unwrap_or_else(|| "ssh".to_string()),
            #[cfg(test)]
            mock_runner: None,
        }
    }

    fn run_gh(&self, args: &[&str], dir: Option<&Path>) -> Result<String, WorkspaceError> {
        #[cfg(test)]
        if let Some(runner) = &self.mock_runner {
            return runner(args);
        }

        let mut command = cmd("gh", args);
        if let Some(d) = dir {
            command = command.dir(d);
        }
        let output = command.read().map_err(|e| {
            WorkspaceError::provider(
                "github-gh",
                format!("Failed to run gh command {:?}: {}", args, e),
            )
        })?;
        Ok(output)
    }

    fn run_git(&self, args: &[&str], dir: Option<&Path>) -> Result<String, WorkspaceError> {
        let mut command = cmd("git", args);
        if let Some(d) = dir {
            command = command.dir(d);
        }
        let output = command.read().map_err(|e| {
            WorkspaceError::Git(format!("Failed to run git command {:?}: {}", args, e))
        })?;
        Ok(output)
    }
}

#[derive(Deserialize)]
struct GhRepoOwner {
    login: String,
}

#[derive(Deserialize)]
struct GhRepoBranchRef {
    name: String,
}

#[derive(Deserialize)]
struct GhRepo {
    name: String,
    owner: GhRepoOwner,
    url: String,
    sshUrl: String,
    defaultBranchRef: Option<GhRepoBranchRef>,
    description: Option<String>,
    updatedAt: Option<String>,
}

#[async_trait]
impl CodeProvider for GitHubGhProvider {
    fn kind(&self) -> &'static str {
        "github-gh"
    }

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError> {
        // gh auth status exit code is 0 if logged in, non-zero if not.
        // Wait, standard gh auth status output is on stderr. Let's just run it.
        let status = cmd("gh", &["auth", "status"]).run();
        match status {
            Ok(output) if output.status.success() => {
                // Try to get current username using gh api user
                let username =
                    if let Ok(user_json) = self.run_gh(&["api", "user", "--jq", ".login"], None) {
                        Some(user_json.trim().to_string())
                    } else {
                        None
                    };
                Ok(AuthStatus {
                    authenticated: true,
                    username,
                    details: Some("GitHub CLI authenticated".to_string()),
                })
            }
            _ => Ok(AuthStatus {
                authenticated: false,
                username: None,
                details: Some(
                    "GitHub CLI not authenticated. Please run 'gh auth login'.".to_string(),
                ),
            }),
        }
    }

    async fn list_recent_repos(
        &self,
        input: ListRecentReposInput,
    ) -> Result<Vec<RepoSummary>, WorkspaceError> {
        let limit_val = input.limit.unwrap_or(50);
        let page_val = input.page.unwrap_or(1);
        let gh_limit = (limit_val * page_val).to_string();

        let mut args = vec![
            "repo",
            "list",
            "--limit",
            &gh_limit,
            "--json",
            "name,owner,url,sshUrl,defaultBranchRef,description,updatedAt",
        ];

        if let Some(owner) = &self.default_owner {
            args.push(owner);
        }

        let output = self.run_gh(&args, None)?;
        let gh_repos: Vec<GhRepo> = serde_json::from_str(&output).map_err(|e| {
            WorkspaceError::provider("github-gh", format!("Failed to parse repo list: {}", e))
        })?;

        let skip_count = limit_val * (page_val - 1);
        let summaries = gh_repos
            .into_iter()
            .skip(skip_count)
            .take(limit_val)
            .map(|r| RepoSummary {
                provider: "github".to_string(),
                owner: r.owner.login.clone(),
                name: r.name.clone(),
                full_name: format!("{}/{}", r.owner.login, r.name),
                url: r.url,
                ssh_url: r.sshUrl,
                default_branch: r
                    .defaultBranchRef
                    .map(|b| b.name)
                    .unwrap_or_else(|| "main".to_string()),
                description: r.description,
                updated_at: r.updatedAt,
            })
            .collect();

        Ok(summaries)
    }

    async fn get_repo(&self, input: RepoRef) -> Result<RepoDetails, WorkspaceError> {
        let repo_str = format!("{}/{}", input.owner, input.name);
        let output = self.run_gh(
            &[
                "repo",
                "view",
                &repo_str,
                "--json",
                "name,owner,url,sshUrl,defaultBranchRef,description,updatedAt",
            ],
            None,
        )?;

        let r: GhRepo = serde_json::from_str(&output).map_err(|e| {
            WorkspaceError::provider("github-gh", format!("Failed to parse repo view: {}", e))
        })?;

        Ok(RepoDetails {
            summary: RepoSummary {
                provider: "github".to_string(),
                owner: r.owner.login.clone(),
                name: r.name.clone(),
                full_name: repo_str,
                url: r.url,
                ssh_url: r.sshUrl,
                default_branch: r
                    .defaultBranchRef
                    .map(|b| b.name)
                    .unwrap_or_else(|| "main".to_string()),
                description: r.description,
                updated_at: r.updatedAt,
            },
        })
    }

    async fn ensure_repo_cache(
        &self,
        input: EnsureRepoCacheInput,
    ) -> Result<RepoCache, WorkspaceError> {
        // Cache directory structure: .cache/repos/<owner>/<name>.git
        let cache_base = Path::new(".cache").join("repos").join(&input.owner);
        fs::create_dir_all(&cache_base)?;
        let cache_path = cache_base.join(format!("{}.git", input.name));

        if cache_path.exists() {
            // Already cloned, run git fetch
            info!("Fetching updates in repo cache: {}", cache_path.display());
            self.run_git(&["fetch", "origin"], Some(&cache_path))?;
        } else {
            // Clone bare repository
            info!(
                "Cloning bare repository to cache: {} from {}",
                cache_path.display(),
                input.url
            );
            // Choose ssh or https based on setting/input URL
            let repo_url = if self.protocol == "ssh" {
                &input.url // Use SSH URL (git@github.com:...)
            } else {
                // Parse and build HTTPS URL if needed, or fallback to the provided URL
                &input.url
            };
            self.run_git(
                &["clone", "--bare", repo_url, &cache_path.to_string_lossy()],
                None,
            )?;
        }

        Ok(RepoCache {
            path: cache_path.to_string_lossy().to_string(),
        })
    }

    async fn create_worktree(
        &self,
        input: CreateWorktreeInput,
    ) -> Result<Worktree, WorkspaceError> {
        let cache_path = Path::new(".cache")
            .join("repos")
            .join(&input.owner)
            .join(format!("{}.git", input.name));

        if !cache_path.exists() {
            return Err(WorkspaceError::Git(format!(
                "Repo cache does not exist at {}",
                cache_path.display()
            )));
        }

        let worktree_dir = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);

        if worktree_dir.exists() {
            warn!(
                "Worktree path {} already exists, using it.",
                worktree_dir.display()
            );
            return Ok(Worktree {
                path: worktree_dir.to_string_lossy().to_string(),
                service_id: input.service_id,
                branch: input.branch,
            });
        }

        fs::create_dir_all(worktree_dir.parent().unwrap())?;

        // Run git worktree add
        // git --git-dir=<cache_path> worktree add -b <branch> <worktree_dir> <base_branch>
        // First ensure latest updates are fetched in cache
        self.run_git(
            &[
                "fetch",
                "origin",
                &format!("{}:{}", input.base_branch, input.base_branch),
            ],
            Some(&cache_path),
        )
        .unwrap_or_default(); // ignore fetch failure if branch is already up-to-date locally

        let worktree_path_str = worktree_dir.to_string_lossy().to_string();
        info!(
            "Creating git worktree at {} from cache {} on branch {}",
            worktree_path_str,
            cache_path.display(),
            input.branch
        );

        // Check if branch already exists in the bare repo cache
        let branch_exists = self
            .run_git(
                &[
                    "show-ref",
                    "--verify",
                    &format!("refs/heads/{}", input.branch),
                ],
                Some(&cache_path),
            )
            .is_ok();

        if branch_exists {
            // Just add worktree checking out the existing branch
            self.run_git(
                &[
                    "--git-dir",
                    &cache_path.to_string_lossy(),
                    "worktree",
                    "add",
                    &worktree_path_str,
                    &input.branch,
                ],
                None,
            )?;
        } else {
            // Create a new branch based on base_branch
            self.run_git(
                &[
                    "--git-dir",
                    &cache_path.to_string_lossy(),
                    "worktree",
                    "add",
                    "-b",
                    &input.branch,
                    &worktree_path_str,
                    &input.base_branch,
                ],
                None,
            )?;
        }

        // Set upstream for push operations. Wait, in git worktrees created from bare clones,
        // we might want to specify the remote configuration or push upstream when we push.
        // We can do it during push.

        Ok(Worktree {
            path: worktree_path_str,
            service_id: input.service_id,
            branch: input.branch,
        })
    }

    async fn push_branch(&self, input: PushBranchInput) -> Result<(), WorkspaceError> {
        let worktree_dir = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);

        if !worktree_dir.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "Worktree for service {} not found at {}",
                input.service_id,
                worktree_dir.display()
            )));
        }

        info!(
            "Pushing branch {} for service {} in workspace {}",
            input.branch, input.service_id, input.epic_key
        );

        // Run git push origin <branch> inside the worktree dir
        // We can use -u to set upstream
        self.run_git(
            &["push", "-u", "origin", &input.branch],
            Some(&worktree_dir),
        )?;

        Ok(())
    }

    async fn create_pull_request(
        &self,
        input: CreatePullRequestInput,
    ) -> Result<PullRequest, WorkspaceError> {
        let worktree_dir = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);

        if !worktree_dir.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "Worktree for service {} not found at {}",
                input.service_id,
                worktree_dir.display()
            )));
        }

        info!(
            "Creating Pull Request for service {} on branch {}",
            input.service_id, input.branch
        );

        let mut args = vec![
            "pr",
            "create",
            "--title",
            &input.title,
            "--body",
            &input.body,
            "--head",
            &input.branch,
        ];

        if input.draft {
            args.push("--draft");
        }

        let output = self.run_gh(&args, Some(&worktree_dir))?;
        // gh pr create returns the PR URL on stdout
        let url = output.trim().to_string();

        // Extract PR number from URL (e.g. https://github.com/owner/repo/pull/123)
        let number = url
            .split('/')
            .last()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        Ok(PullRequest {
            number,
            url,
            state: "open".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use ws_core::models::ListRecentReposInput;

    fn make_mock_repos(count: usize) -> String {
        let repos: Vec<serde_json::Value> = (1..=count)
            .map(|i| {
                serde_json::json!({
                    "name": format!("repo{}", i),
                    "owner": { "login": "testowner" },
                    "url": format!("https://github.com/testowner/repo{}", i),
                    "sshUrl": format!("git@github.com:testowner/repo{}.git", i),
                    "defaultBranchRef": { "name": "main" },
                    "description": format!("Repo {} description", i),
                    "updatedAt": "2026-06-22T08:00:00Z"
                })
            })
            .collect();
        serde_json::to_string(&repos).unwrap()
    }

    #[tokio::test]
    async fn test_list_recent_repos_defaults_to_50() {
        let provider = GitHubGhProvider {
            default_owner: Some("testowner".to_string()),
            protocol: "ssh".to_string(),
            mock_runner: Some(Arc::new(|args| {
                assert_eq!(args[0], "repo");
                assert_eq!(args[1], "list");
                assert_eq!(args[2], "--limit");
                assert_eq!(args[3], "50");
                Ok(make_mock_repos(50))
            })),
        };
        let res = provider
            .list_recent_repos(ListRecentReposInput {
                limit: None,
                page: None,
            })
            .await
            .unwrap();
        assert_eq!(res.len(), 50);
        assert_eq!(res[0].name, "repo1");
        assert_eq!(res[49].name, "repo50");
    }

    #[tokio::test]
    async fn test_list_recent_repos_pagination_page_2() {
        let provider = GitHubGhProvider {
            default_owner: Some("testowner".to_string()),
            protocol: "ssh".to_string(),
            mock_runner: Some(Arc::new(|args| {
                assert_eq!(args[0], "repo");
                assert_eq!(args[1], "list");
                assert_eq!(args[2], "--limit");
                assert_eq!(args[3], "20"); // limit 10 * page 2
                Ok(make_mock_repos(20))
            })),
        };
        let res = provider
            .list_recent_repos(ListRecentReposInput {
                limit: Some(10),
                page: Some(2),
            })
            .await
            .unwrap();
        assert_eq!(res.len(), 10);
        assert_eq!(res[0].name, "repo11");
        assert_eq!(res[9].name, "repo20");
    }
}
