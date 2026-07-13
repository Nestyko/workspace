---
name: ws-self-heal
description: >-
  Run the ws repository quality loop in two modes. Mode A - all repositories is autonomous
  and report-only (no fixes); it emits a consolidated report of what is lacking and what to
  fix. Mode B - single repository is interactive, human-guided fixing to converge one
  service's healthcheck to green via the 2-subagent fix-loop ws defines. ws is the
  deterministic oracle (measures, executes primitives, validates writes); this skill is
  the harness that operates it. Manually invoked only - run it via the harness skill
  command (in pi: /skill:ws-self-heal). On invocation it immediately starts the chosen
  mode's procedure; no natural-language trigger phrase is required.
disable-model-invocation: true
---

# Skill — `ws-self-heal` (Quality Loop & Fix Orchestration)

> Run the `ws` repo quality loop in one of two modes:
> **A — all repositories (autonomous, report-only)** or
> **B — single repository (interactive, human-guided fixing)**.
> `ws` is the deterministic oracle (measures, executes primitives, validates writes);
> this skill is the harness that operates it.

## Invocation

Manual invocation only (the skill is hidden from the system prompt - it never
auto-activates on natural language). Run it through your harness's skill command:

- in pi: `/skill:ws-self-heal`
- generic: `skill:ws-self-heal`

On invocation, **start immediately**: choose the mode below (A = all-repos report,
B = single-repo fix - ask the human which one if not supplied as an argument), then
execute that mode's procedure from its first step. Do not wait for a trigger phrase like
"healthcheck repos", and do not ask whether you should activate - the invocation *is* the
activation signal. Arguments after the command are appended as `User: <args>`, so
`/skill:ws-self-heal <service-id>` may be used to pre-select Mode B and the target.

## The two modes (choose at the start)

| Mode | When | Human in loop? | Fixes? |
|---|---|---|---|
| **A — All repos** | user wants a workspace-wide quality report | no (autonomous) | **No — report only** |
| **B — Single repo** | user wants one repo converged to top quality | yes — guides each fix | Yes |

> The user's contract: running across **all** repositories is autonomous and produces a
> report of *what is lacking and what to fix*; running on a **single** repository expects a
> human present to guide and fix each gap.

---

## Prerequisites

- `ws` installed with the quality commands present: `ws ai manifest | grep -c '"id"'` → expect 39
  (if lower, `cargo install --path crates/ws-cli` from the `ws` source repo).
- A workspace already initialized via the `ws-init` skill (`.ws/config.yaml` + `catalog/` exist).
- **Repo checkouts:** `ws` never clones. For each service you need a working checkout path
  (`repo_path`). Sources: an active workspace worktree under `workspaces/<epic>/repos/<id>`,
  an existing local clone, or a fresh clone you make (`git clone <repo.url>` into a temp dir).

## Sharp edge — field naming (read first)
The `ws` commands use **two different field names** for the service identifier. Do not
waste turns trial-and-erroring:
- `repo.healthcheck`, `repo.run`, `repo.verify`, `repo.fix_loop.prompt`, `repo.understand.verify` → use **`service_id`**.
- `catalog.service.get`, `catalog.service.update` → use **`id`**.

---

## MODE A — All repositories (autonomous, report-only)

Goal: healthcheck every catalog service and emit one consolidated report of gaps. **Do not
run any fix, do not run `repo.run`, do not edit the catalog.** Report only.

### A.1 Enumerate services
```bash
ws ai run catalog.service.list --input '{}'
# → array of services, each with {id, name, repo:{url, default_branch, ...}, ...}
```

### A.2 Obtain a checkout per service
For each service, resolve a `repo_path`:
1. Prefer an existing worktree/clone the human points at.
2. Otherwise clone autonomously: `git clone <repo.url> <tmp>/healthcheck/<id>` and
   `git -C <path> checkout <default_branch>`.
Skip services whose `repo.url` is unreachable and note them in the report as "unable to check out".

### A.3 Healthcheck every service
For each service:
```bash
ws ai run repo.healthcheck --input '{"service_id":"<id>","repo_path":"<path>","check":"all"}'
# → {rows:[{check_id,title,status,blocking,evidence,run_hint}...], summary:{blocking_failures, ...}}
```

### A.4 Produce the consolidated report
Output a single markdown report:

```
# ws Workspace Quality Report

## Summary
| Service | blocking_failures | green? |
|---|---|---|
| svc-a | 7 | ✗ |
| svc-b | 0 | ✓ |

Total services: N · Green: M · With blocking gaps: K
```

Then a per-service section listing every **blocking** check with its `status`, `evidence`,
and the `run_hint` (the remediation the fix would follow). Non-blocking checks (#3
agent-doc, #7 integration) are listed collapsed but not flagged.

End the autonomous run with:
> To fix a service's gaps with a human in the loop, run **Mode B** on that service id.

That is the end of Mode A. Do not fix anything.

---

## MODE B — Single repository (interactive, human-guided fix)

Goal: converge one service's healthcheck to `summary.blocking_failures == 0`, with a human
guiding and approving each fix.

### B.1 Identify the target
Get the `service_id` from the human (or from `catalog.service.list`) and its `repo_path`
(existing checkout or clone on demand). Confirm both with the human before proceeding.

### B.2 Baseline healthcheck
```bash
ws ai run repo.healthcheck --input '{"service_id":"<id>","repo_path":"<path>","check":"all"}'
```
Show the human the full row table so they see every gap and its remediation hint.

### B.3 Fetch the fix-loop spec
```bash
ws ai run repo.fix_loop.prompt --input '{"service_id":"<id>"}'
# → markdown spec: 2-subagent roles + ordered command list (install→dev→test→test_integration→agent_verify; deploy EXCLUDED)
```
The spec defines the **runner** (calls `repo.run`, judges pass/fail) and **implementor**
(edits the repo on failure; never edits the catalog). N=4 attempts per command (locked cap).

### B.4 Remediate per healthcheck point
Not all points are fixed the same way. Walk them in this order with the human:

**Declaration-only points** (record, then re-healthcheck → row goes green):
- **#4 Install** — identify the package manager from the lockfile (`package.json`→`npm install`,
  `Cargo.toml`→`cargo build`, `go.mod`→`go mod download`, `pyproject.toml`→`pip install -e .`
  /`uv sync`, `Gemfile`→`bundle install`). Record:
  ```bash
  ws ai run catalog.service.update --input '{"id":"<id>","commands":{"install":"<cmd>"}}'
  ```
  Note: `commands` is a per-key merge — adding `install` won't touch `test`.
- **#6 Unit tests** — same pattern, key `test` (e.g. `npm test`, `cargo test`, `go test ./...`).
- **#10 Agent-verification** — author `./scripts/verify-change.sh` (re-run unit tests +
  targeted check of the changed surface), record key `agent_verify`.
- **#7 Integration tests** (optional, never blocks) — record `test_integration` if applicable;
  if a pure library/CLI, leave undeclared.
- **#3 AGENT.md/CLAUDE.md** (informational, never blocks) — record via
  `{"id":"<id>","docs":[{"type":"agent","path":"AGENT.md"}]}` if the repo has one.
- **#8 Deploy** — either a plain command string `{"id":"<id>","deploy":"<cmd>"}`, or for a
  library/CLI with no deploy: `{"id":"<id>","deploy":{"skip":true,"reason":"library, no deploy target"}}`.
  Envs/when/triggers are company-level in `workflows/deploy.md`, **not** a catalog field.

**Authoring points:**
- **#2 README.md** — write `README.md` at the repo root: project name, one-paragraph
  purpose, install/test/run commands (pull from `commands.*` once declared), link to
  CONTRIBUTING.md if present. Gate is structural (exists, non-empty, ≥1 `#` heading).
- **#5 Run locally e2e** (three declarations, all blocking):
  - `dev` — local dev start (may not exit).
  - `run` — production start.
  - `verify_run` — one-liner confirming the service came up (e.g.
    `curl -fsS localhost:8080/healthz`). The human may point this at a script they own
    (`./scripts/healthcheck.sh`) — author that script if they want one.
  Record all three; then validate via `repo.run` serve mode (see B.5).

**Human-in-the-loop point (OUT OF SCOPE to fix autonomously):**
- **#1 Understand-Anything** — needs a GitHub Action with an LLM provider + an API-key
  secret + a committed artifact + a PR + merge + trigger-flip. Do **not** attempt this
  autonomously. Surface it to the human with the remediation steps from
  `workflows/repo-init.md` §1; record the intent with
  `{"id":"<id>","understand_anything":{"enabled":true}}` only after the human confirms
  the secret/provider path. Optionally verify later with:
  ```bash
  ws ai run repo.understand.verify --input '{"service_id":"<id>","repo_path":"<path>","pr_number":<n>,"run_id":<n>}'
  ```
  (requires `gh` / `GITHUB_TOKEN`). Until then, `repo.healthcheck` will keep flagging #1
  as blocking — that is expected and the human accepts the deferred gap.

### B.5 Execute the fix-loop for runnable commands
For each executable command (`install`, `dev`/`run` use serve mode, `test`,
`test_integration`, `agent_verify`), drive the 2-subagent loop:

**Runner** (decide pass/fail):
```bash
ws ai run repo.run --input '{"service_id":"<id>","repo_path":"<path>","command":"<key>","timeout":60}'
# → {mode, exit_code, smoke_passed, timed_out, stdout_tail, stderr_tail, duration_secs}
```
**Pass condition:** `smoke_passed == true && timed_out == false`.

**On failure:** spawn the **implementor** subagent. Give it: the failing command key +
value, the `stdout_tail`/`stderr_tail`, and the matching remediation template from
`workflows/repo-init.md`. The implementor edits **the repo only** (scripts/config/code),
never the catalog. Report a short diff summary back.

**Re-run** `repo.run`. Repeat up to **N=4 attempts**. On cap-exhaustion: **halt and report
the gap to the human** — never silently mark it passing.

**On pass:** if the command was newly declared, record it via `catalog.service.update`
(only the runner records declarations, and only after a pass). Advance to the next command.

**Mode notes (from `repo.run`):**
- `install`, `test`, `test_integration`, `agent_verify`, `verify_run` → **exit mode**
  (foreground, expect exit 0).
- `dev`, `run` → **serve mode**: background-start, poll `commands.verify_run` until exit 0
  or the ~3-min deadline (configurable via `timeout`), then **kill the process group**.
  `smoke_passed` reflects the probe.

**`deploy` is excluded from `repo.run`, `repo.verify`, and this loop — locked dangerous.**

### B.6 Converge
Loop: `repo.healthcheck` → fix gap in repo/record declaration → re-`repo.healthcheck`.
Repeat until `summary.blocking_failures == 0` (excluding #1 if the human deferred it, and
#3/#7 which never block). Keep the human informed at each iteration; ask before authoring
repo files (README, scripts) unless they've pre-approved.

### B.7 Final confirmation
```bash
ws ai run repo.verify --input '{"service_id":"<id>","repo_path":"<path>","timeout":180}'
# post-setup deterministic confirmation: whole declared toolchain end-to-end;
# stops at first failure; excludes deploy.
```
If green → report **"repo ready"** + list any deferred gaps (#1) with explicit next steps
for the human. If red → return to B.4/B.5 for the failing command.

---

## Guardrails (locked — from `workflows/repo-verify.md` §5)

- The implementor subagent **never** edits the catalog. Only the runner records
  declarations via `catalog.service.update`, and only after a command passes.
- `deploy` is **never** invoked by `repo.run`, `repo.verify`, or any fix-loop step.
  Envs/when/triggers live in `workflows/deploy.md` (company-level, ws-managed).
- If `gh` / `GITHUB_TOKEN` is needed (`repo.understand.verify`), ensure the credential
  context exists before invoking.
- Never silently succeed: a cap-exhausted command is reported as a gap, not passed.

## Command reference (verified shapes)

| Command | Input (key fields) | Key field |
|---|---|---|
| `repo.healthcheck` | `check:"all"` / single `"1".."10"`, `repo_path`, `service_id` | `service_id` |
| `repo.fix_loop.prompt` | `service_id` | `service_id` |
| `repo.run` | `command`, `repo_path`, `service_id`, `timeout` | `service_id` |
| `repo.verify` | `repo_path`, `service_id`, `timeout` | `service_id` |
| `repo.understand.verify` | `pr_number`, `repo_path`, `run_id`, `service_id` | `service_id` |
| `catalog.service.list` | (none) | — |
| `catalog.service.get` | `id` | `id` |
| `catalog.service.update` | `id` + any of `commands,deploy,docs,understand_anything,owns,likely_relevant_when,...` | `id` |
| `catalog.validate` | (none) | — |

## What is explicitly out of scope
- **Understand-Anything (#1)** full autonomy — needs a secret + provider the customer owns
  (human-in-the-loop by design). This skill surfaces it; the human completes it.
- **Continuous/ongoing quality** (drift watch, per-PR healthcheck in GitHub Actions) — a
  later feature; this skill is on-demand only.
