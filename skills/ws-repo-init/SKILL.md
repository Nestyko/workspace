---
name: ws-repo-init
description: >-
  Bootstrap a cataloged-but-uninitialized repo from zero to "cataloged + Understand-Anything
  artifact + commands smoke-validated" in one fast pass. Runs inside the customer's harness:
  clones + onboarding worktree, invokes the /understand skill locally to produce the
  knowledge-graph artifact (Point #1), derives catalog fields from the graph + lockfile, and
  records them via catalog.service.update. Then runs a 30s repo.run smoke per command,
  recording declarations only on pass (mini fix-loop capped at 4 attempts). A --full flag
  lifts the 30s cap and delegates deep convergence to the full ws-self-heal Mode B loop.
  Prefer .ua/ artifact dir with .understand-anything/ as legacy fallback. Manually invoked
  only - run it via the harness skill command (in pi: /skill:ws-repo-init). On invocation it
  executes the full procedure immediately; no natural-language trigger phrase is required.
disable-model-invocation: true
---

# Skill — `ws-repo-init` (Per-Repo Bootstrap into the Catalog)

> Bridge the gap between **`ws-init`** (scaffold config + catalog) and **`ws-self-heal`**
> (deep convergence of an *already-healthchecked* repo). This skill takes a
> cataloged-but-uninitialized repo from zero to **cataloged + Understand-Anything artifact
> committed + commands smoke-validated** in one fast, mostly-autonomous pass.
>
> `ws` is the deterministic oracle (reads `repo.healthcheck`, executes `repo.run`, validates
> `catalog.service.update` strictly, provides `repo.fix_loop.prompt`). This skill is the
> harness that operates it: it clones, creates the onboarding worktree, runs `/understand`,
> derives catalog fields, and smoke-validates each declared command.

## Invocation

Manual invocation only (the skill is hidden from the system prompt — it never
auto-activates on natural language). Run it through your harness's skill command:

- in pi: `/skill:ws-repo-init`
- generic: `skill:ws-repo-init`

On invocation, **execute the procedure below immediately**, starting at Step 0. Do not wait
for the user to say a trigger phrase like "init repo", and do not ask whether you should
activate — the invocation *is* the activation signal. Arguments after the command are
appended as `User: <args>` (pi behavior), so `/ws:repo-init engine-service --full` is a
valid invocation:
- Parse `--full` flag (default: `false`). If present → Step 7 delegates to the full
  `ws-self-heal` Mode B loop instead of the 30s smoke.
- Remaining tokens = repo selectors (fuzzy-match targets against the catalog).

## Why this skill exists

The workspace has two skills today: `ws-init` (scaffold config + catalog) and `ws-self-heal`
(deep 2-subagent fix-loop convergence). Bootstrapping a **cataloged-but-uninitialized** repo
— clone, onboarding worktree, `/understand` artifact, derive catalog fields, smoke the
commands — is not covered as a single fast operation. This skill is that operation: a
deliberately **fast** (30s timeout) per-repo bootstrap that leaves the repo ready for the
deep self-heal pass.

**Scope boundary:** by default this is a *smoke* — 30s per command, declarations recorded only
on pass, a 4-attempt mini fix-loop, cap-exhaustion halts that repo and reports. `--full` opts
into deep convergence and delegates to `ws-self-heal` Mode B. Init = catalog + artifact +
smoke; deep green = `ws-self-heal`.

**Sharp edge — field naming (carry over from `ws-self-heal`):** the `ws` commands use two
different field names for the service identifier:
- `repo.healthcheck`, `repo.run`, `repo.verify`, `repo.fix_loop.prompt`, `repo.understand.verify` → **`service_id`**.
- `catalog.service.get`, `catalog.service.add`, `catalog.service.update` → **`id`**.

And `ws ai run --input` takes a **file path**, not inline JSON — write every input to a temp
file (e.g. `printf '%s' "$JSON" > /tmp/repo-init-<id>.json && ws ai run ... --input
/tmp/repo-init-<id>.json`).

---

## Prerequisites

- `ws` on PATH with the quality commands (expect 39 in `ws ai manifest`). If fewer,
  reinstall from this repo: `cargo install --path crates/ws-cli`.
- `gh` CLI authenticated (GitHub code provider).
- A workspace already initialized via `ws-init` (`.ws/config.yaml` + `catalog/` exist) with
  **at least one cataloged service** (added via `ws add repo <owner>/<name>`).
- The `/understand` skill installed in the harness that runs this skill (Point #1 artifact).

---

## Procedure

### Step 0 — Pre-flight
Confirm `ws` is installed and the quality commands are present:
```bash
ws ai manifest | grep -c '"id"'        # expect 39; if lower, reinstall from source
```

Parse the invocation args: `--full` flag (default false) and the remaining tokens as repo
selectors. Proceed to Step 1.

### Step 1 — Select repos

**No tokens** → list the catalog and let the customer pick:
```bash
ws ai run catalog.service.list --input '{}'   # → [{id, name, repo:{url, default_branch, ...}}, ...]
```
Present `{id, name, repo.name}` as a table in chat. If the catalog is **empty** → **FAIL**
with the exact message:
> No repos in catalog. Run `/skill:ws-init` first, then add at least one repo (`ws add repo <owner>/<name>`).
Do not proceed.

**Tokens present** → fuzzy-match each token against the catalog's `id`, `name`, and
`repo.name` (case-insensitive substring first, then Levenshtein ≤ 2 for typos). Unmatched →
list the closest candidates and ask the customer to confirm before proceeding. Collect the
resolved `{id, name, repo:{url, default_branch}}` for each selected repo.

### Step 2 — Already-initialized check (per repo)

"Initialized" = `commands.install` + `commands.test` + `description` are all non-empty in
the catalog entry:
```bash
printf '%s' '{"id":"<id>"}' > /tmp/repo-init-get-<id>.json
ws ai run catalog.service.get --input /tmp/repo-init-get-<id>.json
```
If already initialized → ask the customer: "repo `<id>` is already set up — re-run init and
**override (patch)** the derived fields?" Proceed on yes **only**. **Decision (b): override
is a patch, never a wipe** — `catalog.service.update` does a per-key merge; only changed
fields are written. Reuse the existing onboarding branch when present (Step 3).

### Step 3 — Clone + onboarding worktree (per repo)

Resolve the clone dir from `.ws/config.yaml` `paths.cache_dir` (default `.cache/repos`):
- **No clone present** → `git clone <repo.url> <cache_dir>/<id>`.
- `git -C <clone> fetch --quiet`.
- Onboarding worktree path: `<workspaces_dir>/repo-init/<id>` (resolved from
  `.ws/config.yaml` `paths.workspaces_dir`, default `workspaces`).
  - If the worktree already exists (prior run) and the customer confirmed override → reuse
    it (`git -C <worktree> pull --ff-only`).
  - Else create it:
    ```bash
    git -C <clone> worktree add <workspaces_dir>/repo-init/<id> \
        -b ws/repo-init/<id> <default_branch>
    ```

### Step 4 — Understand-Anything artifact (Point #1)

Work inside the worktree dir. The artifact dir preference (Decision #3): `.ua/` is the current
Understand-Anything default and the **preferred** location; `.understand-anything/` is the
**legacy fallback**. `repo.healthcheck` #1 and `repo.understand.verify` probe `.ua/` first.

- If `.ua/knowledge-graph.json` **OR** `.understand-anything/knowledge-graph.json` already
  exists → **skip** the `/understand` run (customer already has the artifact); go to Step 5.
- Else: invoke the pi `/understand` skill. Because a worktree would redirect to the main
  clone root (issue #133) and destroy the artifact path, run it with the redirect disabled:
  ```bash
  UNDERSTAND_NO_WORKTREE_REDIRECT=1   # set in env before invoking the /understand skill
  ```
  `/understand` writes to `.ua/` by default (fresh worktree → no legacy dir).
- If a legacy `.understand-anything/` is found alongside → **prompt the customer to
  migrate**: `git mv .understand-anything .ua` + update `.gitattributes`. Continue regardless
  of whether they migrate; do not force it.
- Apply diff-suppression for the artifact dir in use (from
  `templates/understand-anything.gitattributes`):
  ```gitattributes
  .ua/knowledge-graph.json binary -diff linguist-generated
  .ua/**                 linguist-generated
  ```
  (legacy repos keep the `.understand-anything/` form). Commit on the onboarding branch:
  ```bash
  git -C <worktree> add .gitattributes .ua/knowledge-graph.json
  git -C <worktree> commit -m "chore(understand-anything): commit knowledge-graph artifact"
  ```

### Step 5 — Derive catalog fields (per repo)

From the `.ua/knowledge-graph.json` (summaries / components / layers) + lockfile inspection,
derive — do not guess; if a field cannot be derived, leave it empty and flag it as deferred:

- `description` — one paragraph of what the repo does (from the graph summary).
- `commands.install` — lockfile → `package.json`=`npm install`, `Cargo.toml`=`cargo build`,
  `go.mod`=`go mod download`, `pyproject.toml`=`pip install -e .` / `uv sync`,
  `Gemfile`=`bundle install`.
- `commands.test` — runner from the manifest (`npm test`, `cargo test`, `go test ./...`,
  `pytest`, `vitest`/`jest`).
- `commands.dev` / `commands.run` — best-effort from package.json `scripts` / binary entry
  points. If none, leave empty (deferred to `ws-self-heal`).
- `commands.verify_run` — a one-liner confirming the service came up
  (`curl -fsS localhost:<port>/healthz`); leave empty for libraries/CLIs.
- `commands.agent_verify` — `./scripts/verify-change.sh`; author a stub if absent
  (re-run unit tests + a targeted check of the changed surface).
- `owns` — domains / components from the graph.
- `likely_relevant_when` — keywords / tags from the graph (this is what `context.resolve`
  consumes to decide "do we need this repo for a workspace prompt").
- `deploy` — command string, or `{skip: true, reason: "library, no deploy target"}` for
  library / CLI repos.
- `docs` — `[{type: readme, path: README.md}]`; add `{type: agent, path: AGENT.md}` (or
  `CLAUDE.md`) if present.
- `understand_anything: { enabled: true }`.

Record via `catalog.service.update` (field is **`id`**, NOT `service_id`):
```bash
printf '%s' '{"id":"<id>","description":"...","commands":{"install":"...","test":"...","agent_verify":"./scripts/verify-change.sh","dev":"...","run":"...","verify_run":"..."},"owns":["..."],"likely_relevant_when":["..."],"deploy":{"skip":true,"reason":"library, no deploy target"},"docs":[{"type":"readme","path":"README.md"}],"understand_anything":{"enabled":true}}' > /tmp/repo-init-update-<id>.json
ws ai run catalog.service.update --input /tmp/repo-init-update-<id>.json
```

**Repo not yet in catalog** (caller added via `ws add repo` but the entry is a minimal stub):
first `catalog.service.add` a minimal valid stub (all `required` fields), then
`catalog.service.update` the derived fields above.

### Step 6 — Fast smoke (30s) via `repo.run`

Per repo, run each executable command with a **30s timeout** (Decision #2 — this skill is
intentionally fast/smoke). Write each input to a temp file (field is **`service_id`**):
```bash
printf '%s' '{"service_id":"<id>","repo_path":"<worktree>","command":"install","timeout":30}' > /tmp/repo-init-run-<id>-install.json
ws ai run repo.run --input /tmp/repo-init-run-<id>-install.json
# → {mode, exit_code, smoke_passed, timed_out, stdout_tail, stderr_tail, duration_secs}
```

Commands to smoke (in order): `install`, `test`, `agent_verify` (exit mode); `dev`/`run`
(serve mode) with the `verify_run` probe. Skip a command if it was not derived in Step 5.

**Pass condition:** `smoke_passed == true && timed_out == false`. On pass → record the
declaration via `catalog.service.update` (only the runner records declarations, and only
after a pass). On fail → mini fix-loop:
1. Spawn the **implementor** subagent with the failing command key + value, the
   `stdout_tail` / `stderr_tail`, and the matching remediation template from
   `workflows/repo-init.md`. The implementor edits **the repo only** (scripts/config/code),
   never the catalog.
2. Re-run `repo.run`. Repeat up to **N=4 attempts**.
3. **Cap-exhausted → halt that repo**, record the `stderr_tail` + why, and report it in
   Step 8. Never silently mark a cap-exhausted command as passing.

### Step 7 — `--full` delegation (only if the flag was set)

If `--full` was passed, **skip the 30s cap** in Step 6 entirely. Instead, for each
selected repo, **execute** the full `ws-self-heal` Mode B procedure inline by invoking
`/skill:ws-self-heal <id>` (Mode B — single repository, human-guided fix). That loop runs
`repo.run` + `repo.verify` with **no timeout** and all declared commands must actually
pass; the 2-subagent fix-loop cap of N=4 still applies per command. This skill's bootstrap
produces the catalog fields + artifact that Mode B then converges to deep green.

Only the `--full` path delegates to the deep loop; the default path (Step 6) is the 30s
smoke and does **not** run Mode B. At the end of the `--full` run, follow the Step 8 hand-off
note for any human-in-the-loop gaps that remain (Point #1 CI Action + secret).

### Step 8 — Validate + hand off

Validate the catalog is still green:
```bash
ws ai run catalog.validate --input '{}'
# expect: {"success": true, "message": "All catalog files are valid."}
```

Report **per repo**: status, catalog fields filled, commands that passed / failed / timed
out at 30s, deferred gaps, and the onboarding branch (`ws/repo-init/<id>`) + worktree path.
End with:
> Run `/skill:ws-self-heal <id>` for full convergence (deep Mode B fix-loop).

Note that the CI Action + `ZEN_API_KEY` secret for Point #1's **production refresh** remains
a **human step** per `workflows/repo-init.md` §1: the onboarding PR's `on: pull_request`
Action must run green, the PR must merge, then the harness/customer flips the trigger to
production. This skill commits the artifact + `.gitattributes` + Action file on the
onboarding branch; it does **not** open/merge the PR or flip the trigger autonomously.

---

## Guardrails (locked — from `workflows/repo-verify.md` §5)

- The implementor subagent **never** edits the catalog. Only the runner records declarations
  via `catalog.service.update`, and only after a command passes.
- Override is a **patch** (Decision (b)): `catalog.service.update` does a per-key merge;
  never wipe a field to re-derive it unless the customer explicitly confirms.
- `deploy` is **never** invoked by `repo.run`; `deploy` is only declared/skipped in Step 5.
- If `gh` / `GITHUB_TOKEN` is needed (`repo.understand.verify`), ensure the credential
  context exists before invoking.
- Never silently succeed: a cap-exhausted command is reported as a gap, not passed.
- `ws ai run --input` takes a **file path**, not inline JSON — always write inputs to temp
  files.

## Command reference (verified shapes)

| Command | Input (key fields) | Key field |
|---|---|---|
| `catalog.service.list` | (none) | — |
| `catalog.service.get` | `id` | `id` |
| `catalog.service.add` | minimal required fields | `id` |
| `catalog.service.update` | `id` + any of `commands,deploy,docs,understand_anything,owns,likely_relevant_when,description,...` | `id` |
| `catalog.validate` | (none) | — |
| `repo.run` | `command`, `repo_path`, `service_id`, `timeout` | `service_id` |
| `repo.healthcheck` | `check:"all"` / single `"1".."10"`, `repo_path`, `service_id` | `service_id` |
| `repo.verify` | `repo_path`, `service_id`, `timeout` | `service_id` |
| `repo.understand.verify` | `pr_number`, `repo_path`, `run_id`, `service_id` | `service_id` |
| `repo.fix_loop.prompt` | `service_id` | `service_id` |

## What this skill does NOT do
- **Deep convergence** — that is `ws-self-heal` (Mode B). This skill is smoke + catalog + artifact.
- **Forced migration** of existing legacy `.understand-anything/` repos (prompt only).
- **The CI Action + secret for Point #1's production refresh** — human-in-the-loop, per
  `workflows/repo-init.md` §1 (onboarding PR → Action green → merge → trigger flip).
- **Fixing the drift** in the other two skills' docs (noted for a later cleanup pass).
