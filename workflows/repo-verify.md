# Workflow — Repo Verify & Fix-Loop

> **Scope:** How the harness (Claude Code, etc.) executes the **2-subagent fix-loop**
> during repo setup, and the **per-change verify loop** after a change is implemented.
> `ws` is the deterministic oracle; the harness owns the orchestration. The harness
> consumes `repo.fix_loop.prompt` (the spec emitter) and `repo.run` (the executor
> primitive) defined in this workspace.

---

## 1. The `ws` / harness / customer split (invariant)

- **`ws`** — deterministic. Reads (`repo.healthcheck`), executes a single command
  (`repo.run`), emits specs (`repo.fix_loop.prompt`), validates writes
  (`catalog.service.update`, strict). **Never fixes, never judges, never owns an LLM.**
- **Customer's harness** — the agent. Runs the fix-loop, spawns subagents, authors
  `verify_run` / `agent_verify` scripts, calls `catalog.service.update` to record changes.
- **Customer (human)** — fills gaps the harness can't (probe-script content, picks deploy
  envs in `workflows/deploy.md`, decides integration-test applicability for libraries).

---

## 2. The setup fix-loop (two subagents)

`ws ai run repo.fix_loop.prompt --input {service_id: <id>}` emits the harness-readable
spec. The harness then executes the following for each declared command in order
(`install` → `dev` → `test` → `test_integration` → `agent_verify`; **`deploy` is excluded**):

### 2.1 Roles
- **Runner subagent** — calls `ws ai run repo.run --input {service_id, repo_path, command, timeout}`,
  reads the structured result (`{exit_code, smoke_passed, timed_out, stdout_tail, stderr_tail, duration_secs}`),
  decides pass/fail.
- **Implementor subagent** — spawned by the runner **only on failure** to edit the repo
  (scripts / config / code) so the next run passes. Reports a short diff summary back to
  the runner. It edits **the repo**, never the catalog.

### 2.2 Per-command contract
1. Runner calls `repo.run` for the command.
2. **Pass condition:** `smoke_passed == true && timed_out == false`.
3. **On failure:** runner spawns the implementor subagent with (a) the failing command key
   and value, (b) `stdout_tail` / `stderr_tail`, and (c) the relevant remediation template
   from `workflows/repo-init.md`.
4. Runner re-runs `repo.run`. Repeat up to **N = 4 attempts** per command (locked cap).
   If still failing after the cap, the runner **reports the gap and halts** — it must not
   silently mark the command as passing.
5. **On pass:** runner records any newly-added declaration via `catalog.service.update`,
   then advances to the next command.

### 2.3 Mode notes (from `repo.run`)
- `install`, `test`, `test_integration`, `agent_verify`, `verify_run` → **exit mode**
  (foreground, expect exit 0).
- `dev`, `run` → **serve mode**: background-start, poll `commands.verify_run` until exit 0
  or the 3-min (default, configurable via `timeout`) deadline, then **kill the process
  group** and report. `smoke_passed` reflects the probe.

---

## 3. The per-change verify loop (after a harness edit)

When the harness implements a change against an issue, it follows this loop (NOT the setup
loop above; this is the normal "implement → verify → fix → PR" cycle):

1. Harness edits code in the repo worktree.
2. Harness runs `ws ai run repo.run --input {…, command: "test"}` and `command: "agent_verify"`.
3. **Pass:** both green → harness opens a PR (`ws ai run pr.create`).
4. **Fail:** the harness runs a **mini** 2-subagent fix-loop (roles as above) capped at N=4;
   on green → PR; on cap-exhausted → halt and report.
5. After merge, `ws ai run repo.verify` is the **post-setup deterministic confirmation**
   that the whole declared toolchain runs end-to-end (stops at first failure, excludes
   `deploy`).

---

## 4. The two loops, summarised

| Loop | When | Commands | Excludes |
|---|---|---|---|
| **Discover → fix → record → re-discover** | per repo, during setup | `repo.healthcheck` → harness fixes repo → `catalog.service.update` → `repo.healthcheck` green | (uses the healthcheck row set, not repo.run) |
| **Implement → verify → fix → PR** | per change | harness edit → `repo.run` (test + agent_verify) → on fail mini fix-loop → on green `pr.create` → post-merge `repo.verify` | `deploy` (locked) |

---

## 5. Guardrails

- The harness **never** edits the catalog from inside the implementor subagent; only the
  runner records declarations, and only after a command passes.
- `deploy` is **never** invoked by `repo.run`, `repo.verify`, or the fix-loop. Envs and
  triggers for deploy live in `workflows/deploy.md`.
- If `gh` / `GITHUB_TOKEN` is needed (`repo.understand.verify`), the harness ensures the
  credential context exists before invoking.
