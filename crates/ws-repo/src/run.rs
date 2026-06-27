//! `repo.run` — deterministic single-command executor primitive.
//!
//! Two modes (chosen by the `command` key):
//! - **exit-commands** (`install`, `test`, `test_integration`, `agent_verify`,
//!   `verify_run`): run foreground, expect exit 0.
//! - **serve-commands** (`dev`, `run`): background-start, poll
//!   `commands.verify_run` until it passes (exit 0) or the timeout elapses,
//!   then kill the serve process and report.
//!
//! Default timeout: 180s (3 min), configurable via `timeout` (seconds).
//! `deploy` is NEVER executed here — `repo.run` refuses it (locked dangerous).
//!
//! Returns a structured report. This is the shared primitive both `repo.verify`
//! and the harness fix-loop call.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ws_catalog::get_service;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::error::WorkspaceError;

const DEFAULT_TIMEOUT_SECS: u64 = 180;
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const TAIL_BYTES: usize = 4096;

const EXIT_COMMANDS: &[&str] = &[
    "install",
    "test",
    "test_integration",
    "agent_verify",
    "verify_run",
];
const SERVE_COMMANDS: &[&str] = &["dev", "run"];
/// `deploy` is explicitly forbidden — locked dangerous. Never executed by repo.run.
const FORBIDDEN_COMMANDS: &[&str] = &["deploy"];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoRunInput {
    pub service_id: String,
    /// Absolute or relative path to a working checkout of the repo.
    pub repo_path: String,
    /// Catalog command key to run: install | test | test_integration | agent_verify |
    /// verify_run | dev | run. `deploy` is refused.
    pub command: String,
    /// Timeout in seconds (default 180).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoRunOutput {
    /// Command key that was run.
    pub command: String,
    /// Actual shell string from the catalog.
    pub command_value: String,
    /// "exit" or "serve".
    pub mode: String,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub duration_secs: f64,
    pub timed_out: bool,
    /// For exit mode: exit_code == 0. For serve mode: verify_run probe passed.
    pub smoke_passed: bool,
}

pub struct RepoRunCommand;

#[async_trait]
impl AiCommand for RepoRunCommand {
    const ID: &'static str = "repo.run";
    const DESCRIPTION: &'static str = "Deterministic single-command executor (exit or serve+poll). Never runs deploy.";
    type Input = RepoRunInput;
    type Output = RepoRunOutput;

    async fn run(&self, ctx: CommandContext, input: Self::Input) -> Result<Self::Output, WorkspaceError> {
        let service = get_service(&ctx.workspace_root, &input.service_id)?;
        let repo_root = Path::new(&input.repo_path);
        if !repo_root.exists() {
            return Err(WorkspaceError::NotFound(format!(
                "repo_path '{}' does not exist; ws never clones — pass a working checkout.",
                input.repo_path
            )));
        }

        if FORBIDDEN_COMMANDS.contains(&input.command.as_str()) {
            return Err(WorkspaceError::Validation(format!(
                "refusing to run forbidden command '{}'; deploy is never executed by repo.run.",
                input.command
            )));
        }

        let command_value = service
            .commands
            .get(&input.command)
            .ok_or_else(|| {
                WorkspaceError::Validation(format!(
                    "commands.{} is not declared in catalog for service '{}'",
                    input.command, input.service_id
                ))
            })?
            .clone();

        let timeout = Duration::from_secs(input.timeout.unwrap_or(DEFAULT_TIMEOUT_SECS));

        let (mode, output) = if SERVE_COMMANDS.contains(&input.command.as_str()) {
            let probe = service.commands.get("verify_run").cloned();
            (
                "serve",
                run_serve(&command_value, probe, repo_root, timeout)?,
            )
        } else if EXIT_COMMANDS.contains(&input.command.as_str()) {
            ("exit", run_exit(&command_value, repo_root, timeout)?)
        } else {
            return Err(WorkspaceError::Validation(format!(
                "Unknown command key '{}'. Valid: {:?} (serve) or {:?} (exit).",
                input.command, SERVE_COMMANDS, EXIT_COMMANDS
            )));
        };

        Ok(RepoRunOutput {
            command: input.command,
            command_value,
            mode: mode.to_string(),
            exit_code: output.exit_code,
            stdout_tail: output.stdout_tail,
            stderr_tail: output.stderr_tail,
            duration_secs: output.duration_secs,
            timed_out: output.timed_out,
            smoke_passed: output.smoke_passed,
        })
    }
}

struct RunResult {
    exit_code: Option<i32>,
    stdout_tail: String,
    stderr_tail: String,
    duration_secs: f64,
    timed_out: bool,
    smoke_passed: bool,
}

fn run_exit(cmd: &str, cwd: &Path, timeout: Duration) -> Result<RunResult, WorkspaceError> {
    let start = Instant::now();
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd).current_dir(cwd);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| WorkspaceError::Command(format!("failed to spawn '{}': {}", cmd, e)))?;

    // Collect output in background threads so we can apply a hard timeout.
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let othr = thread::spawn(move || read_all(stdout));
    let ethr = thread::spawn(move || read_all(stderr));

    let (timed_out, exit_code) = match child.wait_timeout(timeout) {
        Ok(status) => (false, status.code()),
        Err(_) => {
            // timed out — kill the whole process group.
            kill_group(&mut child);
            let _ = child.wait();
            (true, None)
        }
    };

    let stdout_tail = tail(othr.join().unwrap_or_default());
    let stderr_tail = tail(ethr.join().unwrap_or_default());
    let smoke_passed = !timed_out && exit_code == Some(0);

    Ok(RunResult {
        exit_code,
        stdout_tail,
        stderr_tail,
        duration_secs: start.elapsed().as_secs_f64(),
        timed_out,
        smoke_passed,
    })
}

fn run_serve(
    serve_cmd: &str,
    probe_cmd: Option<String>,
    cwd: &Path,
    timeout: Duration,
) -> Result<RunResult, WorkspaceError> {
    let start = Instant::now();
    let mut command = Command::new("sh");
    command.arg("-c").arg(serve_cmd).current_dir(cwd);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| WorkspaceError::Command(format!("failed to spawn serve '{}': {}", serve_cmd, e)))?;

    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    // Accumulate the full serve output in shared buffers while we poll.
    let stdout_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let stderr_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let othr = {
        let b = stdout_buf.clone();
        thread::spawn(move || stream_into(stdout, b))
    };
    let ethr = {
        let b = stderr_buf.clone();
        thread::spawn(move || stream_into(stderr, b))
    };

    // Poll verify_run until pass/timeout.
    let mut smoke_passed = false;
    let mut timed_out = false;
    let mut probe_log = String::new();

    let deadline = start + timeout;
    if probe_cmd.is_none() {
        // No verify_run declared — cannot confirm the service came up.
        // Treat as not-verified; let the harness decide. smoke stays false.
    }

    while Instant::now() < deadline {
        if let Some(pc) = &probeCmd(&probe_cmd) {
            match Command::new("sh")
                .arg("-c")
                .arg(pc)
                .current_dir(cwd)
                .output()
            {
                Ok(out) => {
                    if out.status.success() {
                        smoke_passed = true;
                        break;
                    } else {
                        probe_log.push_str(&format!(
                            "[probe exit {}] {}",
                            out.status.code().unwrap_or(-1),
                            String::from_utf8_lossy(&out.stdout)
                        ));
                    }
                }
                Err(e) => {
                    probe_log.push_str(&format!("[probe spawn error: {}]\n", e));
                }
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
    if !smoke_passed && Instant::now() >= deadline {
        timed_out = true;
    }

    // Kill the whole serve process group so any grandchildren (e.g. the actual dev
    // server) die and the piped stdout closes — letting the reader threads join.
    kill_group(&mut child);
    let _ = child.wait();
    // Join collector threads (return fast once the group is killed and pipes close).
    let _ = othr.join();
    let _ = ethr.join();

    let stdout_tail = tail(String::from_utf8_lossy(&stdout_buf.lock().unwrap()).to_string());
    let mut stderr_tail = tail(String::from_utf8_lossy(&stderr_buf.lock().unwrap()).to_string());
    if !probe_log.is_empty() {
        stderr_tail.push_str("\n--- verify_run probe log ---\n");
        stderr_tail.push_str(&probe_log);
    }

    Ok(RunResult {
        exit_code: None, // serve processes are long-running; no meaningful exit code.
        stdout_tail,
        stderr_tail,
        duration_secs: start.elapsed().as_secs_f64(),
        timed_out,
        smoke_passed,
    })
}

/// Small helper to borrow the optional probe command, working around borrow rules.
fn probeCmd(probe: &Option<String>) -> Option<&String> {
    probe.as_ref()
}

fn read_all(mut r: impl Read) -> String {
    let mut s = String::new();
    let _ = r.read_to_string(&mut s);
    s
}

fn stream_into(mut r: impl Read, buf: Arc<Mutex<Vec<u8>>>) {
    let mut chunk = [0u8; 4096];
    loop {
        match r.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if let Ok(mut b) = buf.lock() {
                    b.extend_from_slice(&chunk[..n]);
                }
            }
            Err(_) => break,
        }
    }
}

fn tail(s: String) -> String {
    if s.len() <= TAIL_BYTES {
        s
    } else {
        let start = s.len() - TAIL_BYTES;
        let mut t = String::with_capacity(TAIL_BYTES);
        t.push_str("...[truncated]...\n");
        t.push_str(&s[start..]);
        t
    }
}

// wait_timeout polyfill: std::process::Child has no wait_timeout on stable, so implement
// a simple poll-around with try_wait.
trait ChildWaitTimeoutExt {
    fn wait_timeout(&mut self, timeout: Duration) -> std::io::Result<std::process::ExitStatus>;
}

impl ChildWaitTimeoutExt for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> std::io::Result<std::process::ExitStatus> {
        let start = Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            if start.elapsed() >= timeout {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out"));
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Kill the child and any grandchildren it spawned. The child is launched in its own
/// process group (`process_group(0)`), so sending the signal to `-pgid` reaps the
/// whole tree — important for serve-mode commands like `npm run dev` that fork workers,
/// and so the piped stdout closes promptly (letting reader threads join).
#[cfg(unix)]
fn kill_group(child: &mut std::process::Child) {
    let pid = child.id() as i32;
    // SIGKILL to the process group (negative pid).
    unsafe {
        libc::kill(-pid, libc::SIGKILL);
    }
    // Fallback to direct kill if the group signal failed (e.g. child already gone).
    let _ = child.kill();
}

#[cfg(not(unix))]
fn kill_group(child: &mut std::process::Child) {
    let _ = child.kill();
}
