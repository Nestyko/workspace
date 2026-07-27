//! `ws-dev` — install / uninstall / purge `ws` dev binaries built from GitHub
//! Pull Requests.
//!
//! Lets you test an in-flight PR of the `ws` CLI on your laptop without
//! disturbing your stable `ws` install:
//!
//! ```text
//! ws dev-install   https://github.com/Nestyko/workspace/pull/14
//! ws dev-uninstall https://github.com/Nestyko/workspace/pull/14
//! ws dev-purge
//! ```
//!
//! Dev binaries are installed under your cargo bin dir (`~/.cargo/bin` by
//! default, overridable via `WS_DEV_BIN_DIR`) as `ws-dev` — override with
//! `--name`. A registry of PR → binary mappings is persisted at
//! `~/.ws/dev-installs.json` so `dev-uninstall` can remove every binary that
//! belongs to a PR (a PR may have several dev binaries under different names)
//! and `dev-purge` can wipe them all.
//!
//! These are plain user-facing CLI commands (not AI commands): they have no
//! JSON schema and are intentionally not part of the `ws ai` manifest.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use ws_core::error::WorkspaceError;

/// Default name used for a dev binary when `--name` is not given.
pub const DEFAULT_DEV_NAME: &str = "ws-dev";
const REGISTRY_DIR: &str = ".ws";
const REGISTRY_FILE: &str = "dev-installs.json";
/// Subdir of `~/.ws/` that holds cached source checkouts used for dev builds.
const BUILD_CACHE_DIR: &str = "dev-builds";
/// Local ref namespace used to fetch PR heads into cached checkouts.
const PR_REF_NAMESPACE: &str = "ws-dev-pr";

// =========================================================================
// Data model
// =========================================================================

/// One installed dev binary, tracked in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevInstall {
    pub pr_url: String,
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub name: String,
    /// Absolute path to the installed dev binary.
    pub binary_path: String,
    pub git_sha: Option<String>,
    pub installed_at: String,
}

/// On-disk registry of every dev binary installed via `ws dev-install`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevRegistry {
    #[serde(default)]
    pub installs: Vec<DevInstall>,
}

/// Parsed GitHub PR reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRef {
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
}

#[derive(Debug, Clone)]
pub struct DevInstallInput {
    pub pr_url: String,
    pub name: Option<String>,
    /// When true, replace an existing binary at the destination path.
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct DevInstallOutput {
    pub install: DevInstall,
    /// True if a previous binary at the same path was replaced.
    pub overwritten: bool,
}

#[derive(Debug, Clone)]
pub struct DevUninstallOutput {
    pub removed: Vec<DevInstall>,
}

#[derive(Debug, Clone)]
pub struct DevPurgeOutput {
    pub removed: Vec<DevInstall>,
}

// =========================================================================
// Paths
// =========================================================================

fn home_dir() -> Result<PathBuf, WorkspaceError> {
    BaseDirs::new()
        .map(|b| b.home_dir().to_path_buf())
        .ok_or_else(|| WorkspaceError::Config("Could not determine home directory".into()))
}

/// Path to the persisted PR→binary registry (`~/.ws/dev-installs.json`).
pub fn registry_path() -> Result<PathBuf, WorkspaceError> {
    Ok(home_dir()?.join(REGISTRY_DIR).join(REGISTRY_FILE))
}

fn build_cache_root() -> Result<PathBuf, WorkspaceError> {
    Ok(home_dir()?.join(REGISTRY_DIR).join(BUILD_CACHE_DIR))
}

/// Directory dev binaries are installed into.
///
/// Resolution order:
/// 1. `WS_DEV_BIN_DIR` env var
/// 2. `$CARGO_HOME/bin`
/// 3. `~/.cargo/bin`
pub fn dev_bin_dir() -> Result<PathBuf, WorkspaceError> {
    if let Ok(dir) = std::env::var("WS_DEV_BIN_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    let base = match std::env::var("CARGO_HOME") {
        Ok(ch) if !ch.is_empty() => PathBuf::from(ch),
        _ => home_dir()?.join(".cargo"),
    };
    Ok(base.join("bin"))
}

// =========================================================================
// Registry I/O
// =========================================================================

/// Load the dev-binary registry. Returns an empty registry if the file does
/// not exist (and tolerates a corrupt file by starting fresh).
pub fn load_registry() -> Result<DevRegistry, WorkspaceError> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(DevRegistry::default());
    }
    let content = fs::read_to_string(&path)?;
    let reg: DevRegistry = serde_json::from_str(&content).unwrap_or_default();
    Ok(reg)
}

/// Persist the registry to disk (pretty-printed for human grep-ability).
pub fn save_registry(reg: &DevRegistry) -> Result<(), WorkspaceError> {
    let path = registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(reg)?;
    fs::write(&path, json)?;
    Ok(())
}

// =========================================================================
// URL parsing
// =========================================================================

/// Parse a GitHub PR URL into `(owner, repo, pr_number)`.
///
/// Accepts common variants:
/// - `https://github.com/Nestyko/workspace/pull/14`
/// - `https://github.com/Nestyko/workspace/pull/14/`
/// - `https://github.com/Nestyko/workspace/pull/14/files`
/// (also honor `pulls` as a synonym for `pull`).
pub fn parse_pr_url(url: &str) -> Result<PrRef, WorkspaceError> {
    let trimmed = url.trim();
    let path_part = trimmed.split("github.com/").nth(1).ok_or_else(|| {
        WorkspaceError::Validation(format!(
            "Expected a GitHub PR URL like https://github.com/<owner>/<repo>/pull/<number>, got: {url}"
        ))
    })?;

    let segs: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
    let pull_idx = segs
        .iter()
        .position(|s| *s == "pull" || *s == "pulls")
        .ok_or_else(|| {
            WorkspaceError::Validation(format!("PR URL is missing '/pull/<number>': {url}"))
        })?;

    if pull_idx < 2 {
        return Err(WorkspaceError::Validation(format!(
            "PR URL is missing owner/repo before '/pull': {url}"
        )));
    }

    let owner = segs[pull_idx - 2].to_string();
    let repo = segs[pull_idx - 1].to_string();
    let pr_number = segs
        .get(pull_idx + 1)
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| {
            WorkspaceError::Validation(format!("Could not read PR number from URL: {url}"))
        })?;

    Ok(PrRef {
        owner,
        repo,
        pr_number,
    })
}

// =========================================================================
// Core operations
// =========================================================================

/// Install a dev binary built from the given PR.
///
/// Steps:
/// 1. Parse the PR URL.
/// 2. Clone the repo into a cached checkout (`~/.ws/dev-builds/<owner>-<repo>`)
///    or reuse an existing clone.
/// 3. Fetch the PR head ref and check it out (detached HEAD).
/// 4. `cargo build --release` (output streams live).
/// 5. Copy `target/release/ws` to `<bin_dir>/<name>` and record the mapping.
pub fn dev_install(input: DevInstallInput) -> Result<DevInstallOutput, WorkspaceError> {
    let pr = parse_pr_url(&input.pr_url)?;
    let name = input
        .name
        .clone()
        .unwrap_or_else(|| DEFAULT_DEV_NAME.to_string());

    // Guard: never clobber the stable `ws` install — we only manage dev bins.
    if name == "ws" {
        return Err(WorkspaceError::Validation(
            "Refusing to overwrite the stable `ws` binary — pick a dev name (e.g. `ws-dev`) via --name.".into(),
        ));
    }

    let bin_dir = dev_bin_dir()?;
    fs::create_dir_all(&bin_dir)?;
    let bin_path = bin_dir.join(&name);

    let overwritten = bin_path.exists();
    if overwritten && !input.force {
        return Err(WorkspaceError::Validation(format!(
            "A binary named `{name}` already exists at {}. Pass --force to replace it.",
            bin_path.display()
        )));
    }

    // Source checkout (cached across installs for incremental cargo builds).
    let clone_dir = build_cache_root()?.join(format!("{}-{}", pr.owner, pr.repo));
    ensure_clone(&pr, &clone_dir)?;
    fetch_and_checkout_pr(&pr, &clone_dir)?;
    let git_sha = current_git_sha(&clone_dir)?;

    // Build — streams to the terminal since first builds can take minutes.
    run_cargo_build(&clone_dir)?;

    let built_bin = clone_dir.join("target").join("release").join("ws");
    if !built_bin.exists() {
        return Err(WorkspaceError::Other(format!(
            "Build completed but `{}` was not produced. The PR may rename the binary or the build failed.",
            built_bin.display()
        )));
    }

    fs::copy(&built_bin, &bin_path)?;
    make_executable(&bin_path)?;

    let install = DevInstall {
        pr_url: input.pr_url.trim().to_string(),
        owner: pr.owner.clone(),
        repo: pr.repo.clone(),
        pr_number: pr.pr_number,
        name: name.clone(),
        binary_path: bin_path.to_string_lossy().to_string(),
        git_sha: git_sha.clone(),
        installed_at: Utc::now().to_rfc3339(),
    };

    // Refresh the registry: drop any prior entry for the same (PR, name) so we
    // keep the latest git_sha/installed_at, then append the new record.
    let mut reg = load_registry()?;
    reg.installs.retain(|e| {
        !(e.owner == pr.owner && e.repo == pr.repo && e.pr_number == pr.pr_number && e.name == name)
    });
    reg.installs.push(install.clone());
    save_registry(&reg)?;

    Ok(DevInstallOutput {
        install,
        overwritten,
    })
}

/// Uninstall every dev binary installed from the given PR.
///
/// Matching is by `(owner, repo, pr_number)` so different URL spellings of the
/// same PR collapse to one uninstall.
pub fn dev_uninstall(pr_url: &str) -> Result<DevUninstallOutput, WorkspaceError> {
    let pr = parse_pr_url(pr_url)?;
    let mut reg = load_registry()?;

    let matching: Vec<DevInstall> = reg
        .installs
        .iter()
        .filter(|e| e.owner == pr.owner && e.repo == pr.repo && e.pr_number == pr.pr_number)
        .cloned()
        .collect();

    for e in &matching {
        let _ = fs::remove_file(&e.binary_path);
    }

    reg.installs
        .retain(|e| !(e.owner == pr.owner && e.repo == pr.repo && e.pr_number == pr.pr_number));
    save_registry(&reg)?;

    Ok(DevUninstallOutput { removed: matching })
}

/// Remove **all** dev binaries and clear the registry.
pub fn dev_purge() -> Result<DevPurgeOutput, WorkspaceError> {
    let mut reg = load_registry()?;
    let removed = reg.installs.clone();
    for e in &removed {
        let _ = fs::remove_file(&e.binary_path);
    }
    reg.installs.clear();
    save_registry(&reg)?;
    Ok(DevPurgeOutput { removed })
}

// =========================================================================
// Git / cargo glue
// =========================================================================

fn ensure_clone(pr: &PrRef, dir: &Path) -> Result<(), WorkspaceError> {
    if dir.join(".git").exists() {
        // Refresh remote refs so a transferred/renamed repo still resolves and
        // we always see the latest PR head on fetch.
        let _ = run_capture(dir, "git", &["fetch", "--all", "--prune"]);
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    let parent = dir.parent().unwrap_or_else(|| Path::new("."));
    let url = format!("https://github.com/{}/{}.git", pr.owner, pr.repo);
    run_inherit(parent, "git", &["clone", &url, &dir.to_string_lossy()])
}

fn fetch_and_checkout_pr(pr: &PrRef, dir: &Path) -> Result<(), WorkspaceError> {
    let refspec = format!(
        "pull/{n}/head:refs/{ns}/{n}",
        n = pr.pr_number,
        ns = PR_REF_NAMESPACE
    );
    run_capture(dir, "git", &["fetch", "origin", &refspec])?;

    let local_ref = format!("refs/{ns}/{n}", ns = PR_REF_NAMESPACE, n = pr.pr_number);
    // `--force` here only discards local commits in a scratch build checkout —
    // never user work, since this clone is owned entirely by `ws dev-install`.
    run_inherit(dir, "git", &["checkout", "--force", &local_ref])
}

fn current_git_sha(dir: &Path) -> Result<Option<String>, WorkspaceError> {
    match run_capture(dir, "git", &["rev-parse", "HEAD"]) {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
        Err(_) => Ok(None),
    }
}

fn run_cargo_build(dir: &Path) -> Result<(), WorkspaceError> {
    run_inherit(dir, "cargo", &["build", "--release"])
}

/// Run a command, inheriting stdio (so long-running output streams live).
fn run_inherit(dir: &Path, program: &str, args: &[&str]) -> Result<(), WorkspaceError> {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| WorkspaceError::Command(format!("failed to run `{program}`: {e}")))?;
    if !status.success() {
        return Err(WorkspaceError::Command(format!(
            "`{} {}` failed in {}{}",
            program,
            args.join(" "),
            dir.display(),
            status
                .code()
                .map(|c| format!(" (exit {c})"))
                .unwrap_or_default()
        )));
    }
    Ok(())
}

/// Run a command and capture its output.
fn run_capture(
    dir: &Path,
    program: &str,
    args: &[&str],
) -> Result<std::process::Output, WorkspaceError> {
    let out = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| WorkspaceError::Command(format!("failed to run `{program}`: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Err(WorkspaceError::Command(format!(
            "`{} {}` failed in {}{}{}",
            program,
            args.join(" "),
            dir.display(),
            if stdout.is_empty() {
                String::new()
            } else {
                format!("\nstdout:\n{stdout}")
            },
            if stderr.is_empty() {
                String::new()
            } else {
                format!("\nstderr:\n{stderr}")
            }
        )));
    }
    Ok(out)
}

#[cfg(unix)]
fn make_executable(p: &Path) -> Result<(), WorkspaceError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(p)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(p, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_p: &Path) -> Result<(), WorkspaceError> {
    // On non-unix there is no executable bit to set; `fs::copy` already placed
    // the binary. dev-install is unix-oriented (cargo + shell bins) anyway.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_pr_url() {
        let pr = parse_pr_url("https://github.com/Nestyko/workspace/pull/14").unwrap();
        assert_eq!(
            pr,
            PrRef {
                owner: "Nestyko".into(),
                repo: "workspace".into(),
                pr_number: 14
            }
        );
    }

    #[test]
    fn parses_trailing_slash_and_segments() {
        let pr = parse_pr_url("https://github.com/Nestyko/workspace/pull/14/files").unwrap();
        assert_eq!(pr.pr_number, 14);
        assert_eq!(pr.owner, "Nestyko");
        assert_eq!(pr.repo, "workspace");

        let pr2 = parse_pr_url("https://github.com/Nestyko/workspace/pull/14/").unwrap();
        assert_eq!(pr2.pr_number, 14);
    }

    #[test]
    fn parses_pulls_synonym() {
        let pr = parse_pr_url("https://github.com/Nestyko/workspace/pulls/7").unwrap();
        assert_eq!(pr.pr_number, 7);
    }

    #[test]
    fn rejects_non_github_url() {
        let err = parse_pr_url("https://gitlab.com/Nestyko/workspace/pull/14").unwrap_err();
        assert!(err.to_string().contains("GitHub"));
    }

    #[test]
    fn rejects_missing_pr_number() {
        let err = parse_pr_url("https://github.com/Nestyko/workspace/pull/").unwrap_err();
        assert!(err.to_string().contains("PR number"));
    }

    #[test]
    fn registry_round_trips_with_temp_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Redirect HOME so registry_path() lands in the temp dir.
        // (BaseDirs derives from $HOME on unix/macOS.)
        std::env::set_var("HOME", tmp.path());

        let reg = load_registry().unwrap();
        assert!(reg.installs.is_empty(), "registry should start empty");

        let mut reg = reg;
        reg.installs.push(DevInstall {
            pr_url: "https://github.com/Owner/Repo/pull/1".into(),
            owner: "Owner".into(),
            repo: "Repo".into(),
            pr_number: 1,
            name: "ws-dev".into(),
            binary_path: "/tmp/ws-dev".into(),
            git_sha: Some("deadbeef".into()),
            installed_at: "2024-01-01T00:00:00+00:00".into(),
        });
        save_registry(&reg).unwrap();

        let loaded = load_registry().unwrap();
        assert_eq!(loaded.installs.len(), 1);
        assert_eq!(loaded.installs[0].name, "ws-dev");
        assert_eq!(loaded.installs[0].pr_number, 1);

        // uninstall_by_pr matches owner/repo/pr_number
        let out = dev_uninstall("https://github.com/Owner/Repo/pull/1").unwrap();
        assert_eq!(out.removed.len(), 1);
        assert!(load_registry().unwrap().installs.is_empty());
    }
}
