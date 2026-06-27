# Workflow — Repo-Init Healthcheck (orchestration for the harness)

> **Scope:** Orchestrate the customer's harness through the **10-point repo-init
> checklist** for a service repo. For each point: the `repo.healthcheck` entry it maps to,
> the remediation template the harness follows when the row is unsatisfied, and **when**
> `catalog.service.update` should be called to record a declaration.
>
> `ws` is the read-only deterministic oracle + the strict catalog-write path. The harness
> fills gaps, authors scripts, and drives fix-loops. These remediation templates are the
> prose the harness reads when `repo.healthcheck` reports `missing` / `not_declared` /
> `partial` for each point.

---

## 0. The invariant (read first)

- **`ws`** — deterministic oracle. Reads (`repo.healthcheck`), executes primitives
  (`repo.run`), emits specs (`repo.fix_loop.prompt`), validates writes
  (`catalog.service.update`, strict — rejects unknown keys). **Never fixes, never
  scaffolds, never judges, never owns an LLM.**
- **Harness** — the agent. Runs this workflow, picks provider for #1, performs the
  2-subagent fix-loop (`workflows/repo-verify.md`), authors `verify_run` / `agent_verify`
  scripts, flips the #1 GitHub Action trigger after green, calls `catalog.service.update`.
- **Customer (human)** — fills gaps the harness can't (probe-script content, deploy envs
  per `workflows/deploy.md`, decides integration-test applicability for libraries).

### The 10-point checklist (locked)

```
✅ #1  Understand-Anything       (CI-authored artifact, committed, diff-suppressed, refreshed on merge-to-default-branch)
✅ #2  README.md                 (structural-only gate: exists, non-empty, ≥1 # heading)
✅ #3  AGENT.md / CLAUDE.md      (per-repo: optional reference only, no validation; company AGENTS.md: ws-managed)
✅ #4  Install deps              (declaration-only: commands.install)
✅ #5  Run locally e2e           (commands.dev + commands.run + commands.verify_run)
✅ #6  Unit tests                (declaration-only: commands.test)
✅ #7  Integration tests         (commands.test_integration, declaration-only, dev-targeted, optional)
✅ #8  Deploy + envs + when      (commands.deploy plain string OR {skip:true}; envs/when → workflows/deploy.md, ws-managed)
~  #9  Best practices           ← DE-SCOPED (no catalog field, no healthcheck row)
✅ #10 Agent-verification        (commands.agent_verify, declaration-only; per-edit gate in fix-loop)
~  #11 Collaboration (CONTRIBUTE) ← DE-SCOPED
```

### Discovery loop (per repo, during setup)
`repo.healthcheck` → harness/customer fills gap in repo → `catalog.service.update` →
`repo.healthcheck` green. Repeat until `summary.blocking_failures == 0`.

---

## 1. Point #1 — Understand-Anything · *file + CI existence*

**Healthcheck gate** (status `present`): `.gitattributes` contains the canonical
diff-suppression lines **AND** `.github/workflows/understand-anything.yml` exists **AND**
`.understand-anything/knowledge-graph.json` is present in the tree. If
`understand_anything.enabled` is unset/false, the row is `not_declared` (blocking).

**Remediation:**
1. Set `understand_anything: { enabled: true }` via `catalog.service.update`.
2. Add the canonical `.gitattributes` lines (commit the artifact; suppress the diff):
   ```gitattributes
   .understand-anything/knowledge-graph.json binary -diff linguist-generated
   .understand-anything/**             linguist-generated
   ```
   Leave `.understand-anything/config.json` and any plugin manifests committed as plain text.
3. Ship the GitHub Action with `on: pull_request` (so it runs against the onboarding branch
   before merge). Use the validated default in `templates/understand-anything-action.yml`
   (OpenCode harness + Zen provider; company API key from a GitHub Actions secret).
4. Open the PR that adds the workflow + artifact + `.gitattributes`. The Action runs on the PR.
5. **After the Action completes green and the PR is merged**, the harness flips the trigger
   to production: `on: { push: {}, workflow_dispatch: }` with a job-level conditional
   `if: github.ref == format('refs/heads/{0}', github.event.repository.default_branch)`
   (no hardcoding `main`).
6. Verify with `ws ai run repo.understand.verify --input {service_id, repo_path, pr_number}`
   (asserts artifact parses + Action green + PR merged). Requires `gh` / `GITHUB_TOKEN`.

> **Assumption to verify:** OpenCode Zen is reachable from CI with an API key. The
> customer said this pair is validated; confirm Zen's CI auth path when onboarding.

---

## 2. Point #2 — README.md · *structural-only*

**Gate** (`present`): README exists at repo root, non-empty, has ≥1 `#` heading.
Path resolved from `docs:[{type:readme}]` if declared, else `README.md`. Structural only
— no section or substantiveness requirements.

**Remediation:** *"No README found. Create `README.md` at the repo root containing: project
name, one-paragraph purpose, install/test/run commands (pull from `commands.*` once
present), and a link to `CONTRIBUTING.md` if it exists."*

(`README.md` is a repo file, not a catalog declaration. The harness authors it directly;
no `catalog.service.update` needed unless declaring `docs:[{type:readme, path:README.md}]`.)

---

## 3. Point #3 — AGENT.md / CLAUDE.md · *informational only, never blocks*

**Gate:** declaration of `docs:[{type:agent, path}]`. **Informational only — no validation,
no stub-check, never blocking.** The harness may note whether the declared file exists on
disk (the healthcheck evidence reports it), but absence is not a failure.

**Remediation:** *"Optional — no validation. If your repo has an AGENT.md/CLAUDE.md, declare
it via `docs:[{type:agent, path:...}]` in the catalog."* (Company `AGENTS.md` is ws-managed
via `ws init` + `provider.config.sync_instructions`.)

Record via `catalog.service.update` (top-level `docs` replaces the whole array).

---

## 4. Point #4 — Install deps · *declaration-only*

**Gate** (`declared`): `commands.install` present.

**Remediation:** *"No `commands.install` declared. Identify the repo's package manager from
its lockfile (`package.json`→`npm install`, `Cargo.toml`→`cargo build`, `go.mod`→`go mod
download`, `pyproject.toml`→`pip install -e .` or `uv sync`, `Gemfile`→`bundle install`,
etc.), add the script, and record via `catalog.service.update`."*

(`commands` is a map → `catalog.service.update` does a per-key merge; adding
`commands.install` won't touch `commands.test`.)

---

## 5. Point #5 — Run locally e2e · *declaration ×3 (blocking)*

**Gate** (`present`): `commands.dev`, `commands.run`, `commands.verify_run` all declared.
Partial if only some are present. The runtime probe is exercised by `repo.run`'s serve
mode and `repo.verify`.

**Remediation:** *"No `commands.dev` / `commands.run` / `commands.verify_run` declared.
`dev` = command that starts the service for local development (may not exit, e.g.
`npm run dev`). `run` = production start. `verify_run` = a one-liner that confirms the
service came up (e.g. `curl -fsS localhost:8080/healthz`) — **you may point this at your
own script (`./scripts/healthcheck.sh`), which you own**. Record via
`catalog.service.update`."*

---

## 6. Point #6 — Unit tests · *declaration-only*

**Gate** (`declared`): `commands.test` present.

**Remediation:** *"No `commands.test` declared. Identify the test runner (`package.json`→
`npm test`, `Cargo.toml`→`cargo test`, `go.mod`→`go test ./...`, `pytest`, `vitest`/`jest`),
add it, record via `catalog.service.update`."*

---

## 7. Point #7 — Integration tests · *declaration-only, **optional*** (never blocks)

**Gate:** `commands.test_integration` declared (`declared`) or absent (`not_declared`).
**Both are non-blocking.** Plain absence = `missing`/`not_declared`, *not* `n/a`. There is
no explicit `skip` field for #7.

**Remediation:** *"No `commands.test_integration` declared. If your service has integration
tests pointing at a running instance, add the script (popular: `npm run test:integration`,
`pytest tests/integration`, `go test -tags=integration`), ensure it expects the service from
#5 already running on the `dev` port, and record via `catalog.service.update`. If the repo
has no meaningful integration tests (pure library, CLI tool without service), leave
undeclared — #7 is optional."*

> **Caveat:** because there's no deliberate-opt-out field, the healthcheck cannot
> distinguish *forgotten* integration tests from *deliberately-absent* for a library repo —
> the harness must judge. If this becomes a pain point, a follow-up adding
> `commands.test_integration: null` (deliberate-opt-out) semantics is the upgrade path.

---

## 8. Point #8 — Deploy + envs + when · *declaration OR skip*

**Gate** (`declared`): `deploy` is a plain command string **OR** `{skip: true, reason}`
(explicit opt-out for libraries / CLIs). Envs / when / triggers are **not** a catalog field —
they live in `workflows/deploy.md` (company-level, ws-managed). The healthcheck reports
`skip: true` as `declared` (satisfied).

**Remediation:** *"No `commands.deploy` declared. Add the deploy command as a plain string.
Envs/when/triggers are documented at company level in `workflows/deploy.md`. If this repo
has no deploy (library/cli), set `deploy: {skip: true, reason: ...}`."*

Record via `catalog.service.update` (top-level `deploy` replaces whole value). **`deploy`
is excluded from `repo.run`, `repo.verify`, and `repo.fix_loop.prompt` — locked dangerous.**

---

## 9 — Best practices · *DE-SCOPED*
No catalog field, no healthcheck row. Do not add one.

## 10. Point #10 — Agent-verification · *declaration-only*

**Gate** (`declared`): `commands.agent_verify` present. This is the per-edit verification
gate the harness runs after implementing a change (see `workflows/repo-verify.md`).

**Remediation:** *"No `commands.agent_verify` declared. Author a script that verifies a
single change (re-run unit tests + a targeted check of the changed surface), add it to the
repo (e.g. `./scripts/verify-change.sh`), and record via `catalog.service.update`."*

## 11 — Collaboration (CONTRIBUTING) · *DE-SCOPED*
No catalog field, no healthcheck row. Do not add one.

---

## Healthcheck gate-type summary (locked)

| # | Gate type | Pass condition |
|---|---|---|
| 1 | file+CI existence | `.gitattributes` lines + workflow file + artifact in tree |
| 2 | structural | README exists, non-empty, ≥1 `#` heading |
| 3 | declaration (informational only) | `docs:[{type:agent}]` declared/not-declared (never blocks) |
| 4 | declaration | `commands.install` declared |
| 5 | declaration (×3) | `commands.dev` + `commands.run` + `commands.verify_run` all declared |
| 6 | declaration | `commands.test` declared |
| 7 | declaration (optional) | `commands.test_integration` declared/missing (never blocks) |
| 8 | declaration | `deploy` declared OR `skip:true` |
| 10 | declaration | `commands.agent_verify` declared |

Gate types are locked: declaration-only (no probe, no drift heuristic, no execution) for
every point except #1 / #2, which are structural/file-existence only. **`ws` never fixes,
never scaffolds, never runs commands during a healthcheck.**
