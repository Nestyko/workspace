//! Integration tests for feature workspaces, driving the compiled `ws-test`
//! binary (see the `[[bin]] ws-test` target in crates/ws-cli/Cargo.toml).
//!
//! Two layers:
//! - read-only CLI surface (`ws tasks` list/show/open, `ws status`, `ws attach`)
//!   exercised against hand-authored `workspaces/*/workspace.yaml` fixtures;
//! - a full offline end-to-end (`workspace.create` → `ws tasks` → attach →
//!   `workspace.add_task`) backed by a local bare git remote, so no network or
//!   provider auth is needed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ws-test")
}

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn git")
}

fn git_ok(dir: &Path, args: &[&str]) {
    let out = git(dir, args);
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .current_dir(root)
        .output()
        .expect("spawn ws-test")
}

fn run_ok(root: &Path, args: &[&str]) -> Output {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "ws-test {:?} failed\nstatus: {}\nstdout: {}\nstderr: {}",
        args,
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, content).unwrap();
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Author a fake (no real git worktrees) workspace.yaml fixture.
fn write_workspace(
    root: &Path,
    folder: &str,
    id: &str,
    ticket: Option<&str>,
    title: &str,
    description: &str,
    services: &[&str],
) {
    let services_yaml = services
        .iter()
        .map(|s| format!("  - {}", s))
        .collect::<Vec<_>>()
        .join("\n");
    let ticket_line = match ticket {
        Some(t) => format!("ticket: {}\n", t),
        None => String::new(),
    };
    write(
        root,
        &format!("workspaces/{}/workspace.yaml", folder),
        &format!(
            "id: {}\n{}title: {}\ndescription: {}\nservices:\n{}\nbase_branch: main\ncreate_branches: true\neditor: cursor\n",
            id, ticket_line, title, description, services_yaml
        ),
    );
}

// ==========================================
// Minimal local config + catalog service
// ==========================================

fn write_config(root: &Path) {
    write(
        root,
        ".ws/config.yaml",
        "issue_provider:\n  type: dex\ncode_provider:\n  type: github-gh\n  default_owner: testowner\n  protocol: https\neditor:\n  default: cursor\npaths:\n  cache_dir: .cache/repos\n  workspaces_dir: workspaces\n",
    );
}

fn write_service(root: &Path, remote: &Path) {
    write(
        root,
        "catalog/services/api.yaml",
        &format!(
            "id: api\nname: API\nkind: service\ndescription: test service\nteam: platform\nproducts: []\nrepo:\n  provider: github\n  owner: testowner\n  name: api-repo\n  url: \"{}\"\n  default_branch: main\nowns: []\nlikely_relevant_when: []\ncommands: {{}}\nissue_tracking:\n  provider: dex\n  project: PLATFORM\ndocs: []\n",
            remote.display()
        ),
    );
}

/// Seed an offline remote: a working repo with one commit on `main`, cloned
/// bare into `root/remotes/<name>.git`.
fn seed_remote(root: &Path, name: &str) -> PathBuf {
    let src = root.join("src").join(name);
    git_ok(root, &["init", "-b", "main", src.to_str().unwrap()]);
    write(&src, "hello.txt", &format!("{}-1\n", name));
    git_ok(&src, &["config", "user.email", "test@example.com"]);
    git_ok(&src, &["config", "user.name", "Test User"]);
    git_ok(&src, &["add", "."]);
    git_ok(&src, &["commit", "-m", "initial commit"]);

    let remote = root.join("remotes").join(format!("{}.git", name));
    git_ok(
        root,
        &[
            "clone",
            "--bare",
            src.to_str().unwrap(),
            remote.to_str().unwrap(),
        ],
    );
    remote
}

// ==========================================
// ws tasks — list / show / open (fixture based, read-only)
// ==========================================

#[test]
fn tasks_list_lists_unticketed_and_attached() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_workspace(
        root,
        "realtime-chat",
        "realtime-chat",
        None,
        "Realtime Chat",
        "A realtime chat agent for the web",
        &["api", "web"],
    );
    write_workspace(
        root,
        "EPIC-123-notifications",
        "notifications",
        Some("EPIC-123"),
        "Notifications",
        "Push notification pipeline",
        &["api"],
    );

    let out = run_ok(root, &["tasks"]);
    let s = stdout(&out);
    assert!(
        s.contains("realtime-chat"),
        "list should include workspace id: {}",
        s
    );
    assert!(
        s.contains("unticketed"),
        "unticketed marker expected: {}",
        s
    );
    assert!(s.contains("EPIC-123"), "ticket must appear: {}", s);
    assert!(
        s.contains("A realtime chat agent for the web"),
        "description should travel with the row: {}",
        s
    );
    // `tasks list` is an explicit alias.
    let out = run_ok(root, &["tasks", "list"]);
    assert_eq!(stdout(&out), s);
}

#[test]
fn tasks_empty_root_prints_guidance() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_ok(tmp.path(), &["tasks"]);
    let s = stdout(&out);
    assert!(s.contains("No open feature workspaces"), "{}", s);
}

#[test]
fn tasks_show_expands_a_row() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_workspace(
        root,
        "realtime-chat",
        "realtime-chat",
        None,
        "Realtime Chat",
        "A realtime chat agent for the web",
        &["api"],
    );

    let out = run_ok(root, &["tasks", "show", "realtime-chat"]);
    let s = stdout(&out);
    assert!(s.contains("Workspace: realtime-chat"), "{}", s);
    assert!(s.contains("unticketed"), "{}", s);
    assert!(s.contains("realtime-chat"), "folder: {}", s);
    assert!(
        s.contains("/workspaces/realtime-chat"),
        "absolute path: {}",
        s
    );
    assert!(s.contains("base_branch:  main"), "{}", s);
}

#[test]
fn tasks_open_prints_path_for_cd() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_workspace(
        root,
        "realtime-chat",
        "realtime-chat",
        None,
        "Realtime Chat",
        "A realtime chat agent for the web",
        &["api"],
    );
    write_workspace(
        root,
        "EPIC-123-notifications",
        "notifications",
        Some("EPIC-123"),
        "Notifications",
        "Push notification pipeline",
        &["api"],
    );

    // By id.
    let out = run_ok(root, &["tasks", "open", "realtime-chat"]);
    let path = stdout(&out).trim().to_string();
    assert!(
        path.ends_with("/workspaces/realtime-chat/repos/api"),
        "open by id: {}",
        path
    );
    // `ws tasks <token>` is a shorthand for `tasks open`.
    let out = run_ok(root, &["tasks", "realtime-chat"]);
    assert_eq!(stdout(&out).trim(), path);
    // By ticket prefix fallback.
    let out = run_ok(root, &["tasks", "open", "EPIC-123"]);
    let path = stdout(&out).trim().to_string();
    assert!(
        path.ends_with("/workspaces/EPIC-123-notifications/repos/api"),
        "open by ticket: {}",
        path
    );
}

#[test]
fn tasks_open_matches_description_semantically() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Oddly-named workspace: id does NOT contain the feature word.
    write_workspace(
        root,
        "odd-name",
        "odd-name",
        None,
        "RTC",
        "A realtime chat agent for the web",
        &["api"],
    );

    let out = run_ok(root, &["tasks", "open", "realtime"]);
    let path = stdout(&out).trim().to_string();
    assert!(
        path.ends_with("/workspaces/odd-name/repos/api"),
        "description match must resolve the workspace: {}",
        path
    );
}

#[test]
fn tasks_open_resolves_by_list_index() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_workspace(
        root,
        "alpha",
        "alpha",
        None,
        "Alpha",
        "first fixture workspace",
        &["api"],
    );
    write_workspace(
        root,
        "beta",
        "beta",
        None,
        "Beta",
        "second fixture workspace",
        &["api"],
    );

    // Row 1 is `alpha` (sorted by folder name).
    let out = run_ok(root, &["tasks", "open", "1"]);
    let path = stdout(&out).trim().to_string();
    assert!(path.ends_with("/workspaces/alpha/repos/api"), "{}", path);
}

#[test]
fn tasks_ambiguous_token_lists_candidates_and_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Two workspaces whose descriptions both mention "payment".
    write_workspace(
        root,
        "one",
        "one",
        None,
        "One",
        "payment checkout flows",
        &["api"],
    );
    write_workspace(
        root,
        "two",
        "two",
        None,
        "Two",
        "payment payout flows",
        &["api"],
    );

    let out = run(root, &["tasks", "open", "payment"]);
    assert!(
        !out.status.success(),
        "ambiguous token should fail ({})",
        stdout(&out)
    );
    let combined = format!("{} {}", stdout(&out), String::from_utf8_lossy(&out.stderr));
    assert!(
        combined.to_lowercase().contains("ambiguous") && combined.contains("2 candidates"),
        "candidate list expected: {}",
        combined
    );
}

#[test]
fn status_reports_unticketed_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_workspace(
        root,
        "realtime-chat",
        "realtime-chat",
        None,
        "Realtime Chat",
        "A realtime chat agent for the web",
        &["api"],
    );

    let out = run_ok(root, &["status", "realtime-chat"]);
    let s = stdout(&out);
    assert!(s.contains("unticketed"), "{}", s);
    assert!(s.contains("Workspace realtime-chat"), "{}", s);
}

// ==========================================
// Full offline end-to-end: create → tasks → attach → add_task
// ==========================================

#[test]
fn create_attach_add_task_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_config(root);
    let remote = seed_remote(root, "api-repo");
    write_service(root, &remote);

    // workspace.create — title-derived slug, no ticket required.
    write(
        root,
        "create.json",
        r#"{"title": "Realtime Chat", "description": "A realtime chat agent for the web", "services": ["api"]}"#,
    );
    let out = run_ok(
        root,
        &["ai", "run", "workspace.create", "--input", "create.json"],
    );
    assert!(stdout(&out).contains("realtime-chat"), "{}", stdout(&out));

    let wdir = root.join("workspaces").join("realtime-chat");
    assert!(wdir.join("workspace.yaml").is_file());
    assert!(wdir.join("locks.yaml").is_file());
    assert!(
        wdir.join("AGENTS.md").is_file(),
        "resume file must be written"
    );
    assert!(wdir.join("repos").join("api").join(".git").exists());
    // Slug is the branch until a ticket is attached.
    let w6branch = git(
        &wdir.join("repos").join("api"),
        &["rev-parse", "--abbrev-ref", "HEAD"],
    );
    assert_eq!(stdout(&w6branch).trim(), "realtime-chat");
    let workspace_yaml = std::fs::read_to_string(wdir.join("workspace.yaml")).unwrap();
    assert!(workspace_yaml.contains("ticket:"), "ticket key present");

    // ws tasks lists it as unticketed.
    let out = run_ok(root, &["tasks"]);
    let s = stdout(&out);
    assert!(
        s.contains("realtime-chat") && s.contains("unticketed"),
        "{}",
        s
    );

    // ws tasks open / status resolve by id.
    let out = run_ok(root, &["tasks", "open", "realtime-chat"]);
    let open_path = stdout(&out).trim().to_string();
    assert!(
        open_path.ends_with("/workspaces/realtime-chat/repos/api"),
        "{}",
        open_path
    );
    assert!(Path::new(&open_path).is_dir(), "open path must exist");

    // attach — moves folder to TICKET-slug, moves worktrees with git metadata.
    let out = run_ok(root, &["attach", "realtime-chat", "--ticket", "EPIC-123"]);
    assert!(stdout(&out).contains("EPIC-123"), "{}", stdout(&out));

    let newdir = root.join("workspaces").join("EPIC-123-realtime-chat");
    assert!(!wdir.exists(), "old folder removed after attach");
    assert!(
        newdir.join("workspace.yaml").is_file(),
        "new folder created"
    );
    assert!(
        newdir.join("AGENTS.md").is_file(),
        "resume file moved with the folder"
    );
    assert!(
        newdir.join("repos").join("api").join(".git").exists(),
        "worktree moved with git metadata"
    );
    // The moved worktree must still be a working git worktree.
    let branch = git(
        &newdir.join("repos").join("api"),
        &["rev-parse", "--abbrev-ref", "HEAD"],
    );
    assert_eq!(stdout(&branch).trim(), "realtime-chat");
    let status = git(
        &newdir.join("repos").join("api"),
        &["status", "--porcelain"],
    );
    assert!(status.status.success(), "worktree usable after move");

    // status resolves by ticket prefix and no longer reports unticketed.
    let out = run_ok(root, &["status", "EPIC-123"]);
    let s = stdout(&out);
    assert!(s.contains("EPIC-123") && !s.contains("unticketed"), "{}", s);

    // workspace.add_task — per-task worktree on branch <id>-<slug>.
    write(
        root,
        "addtask.json",
        r#"{"q": "EPIC-123", "key": "TASK-A", "slug": "task-a", "services": ["api"]}"#,
    );
    let out = run_ok(
        root,
        &["ai", "run", "workspace.add_task", "--input", "addtask.json"],
    );
    assert!(stdout(&out).contains("task-a"), "{}", stdout(&out));

    let taskdir = newdir.join("tasks").join("task-a").join("api");
    assert!(taskdir.join(".git").exists(), "task worktree exists");
    let branch = git(&taskdir, &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(stdout(&branch).trim(), "realtime-chat-task-a");
    let locks = std::fs::read_to_string(newdir.join("locks.yaml")).unwrap();
    assert!(
        locks.contains("task-a"),
        "task baseline in locks.yaml: {}",
        locks
    );

    // ws tasks surfaces the task row with branch + task open target.
    let out = run_ok(root, &["tasks"]);
    let s = stdout(&out);
    assert!(s.contains("realtime-chat/task-a"), "{}", s);
    assert!(s.contains("realtime-chat-task-a"), "task branch: {}", s);

    let out = run_ok(root, &["tasks", "open", "task-a"]);
    let task_open = stdout(&out).trim().to_string();
    assert!(
        task_open.ends_with("/tasks/task-a/api"),
        "task open path: {}",
        task_open
    );

    let out = run_ok(root, &["tasks", "show", "EPIC-123"]);
    let s = stdout(&out);
    assert!(s.contains("TASK-A"), "show expands the task: {}", s);
    assert!(
        s.contains("realtime-chat-task-a"),
        "show expands task branch: {}",
        s
    );
}

/// Duplicate slug resolution must be rejected, not silently clobbered.
#[test]
fn create_refuses_existing_slug() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_config(root);
    let remote = seed_remote(root, "api-repo");
    write_service(root, &remote);

    write(
        root,
        "create.json",
        r#"{"title": "Realtime Chat", "description": "A realtime chat agent", "services": ["api"]}"#,
    );
    run_ok(
        root,
        &["ai", "run", "workspace.create", "--input", "create.json"],
    );

    let out = run(
        root,
        &["ai", "run", "workspace.create", "--input", "create.json"],
    );
    assert!(!out.status.success(), "re-creating a slug must fail");
    let combined = format!("{} {}", stdout(&out), String::from_utf8_lossy(&out.stderr));
    assert!(
        combined.contains("already exists"),
        "duplicate guard message expected: {}",
        combined
    );
}
