# Workflow — Deploy Policy (company-level)

> **Scope:** This document defines the company policy for **environments, timing, and
> triggers** for service deployment. It is the prose counterpart to the per-service
> catalog field `deploy:` (see `schemas/service.schema.json`).
>
> **Owner of this file:** `ws` (the tool). `provider.config.sync_instructions` regenerates
> the root `AGENTS.md` and links this document so every harness and human operating in
> the workspace reads the same policy.

---

## 1. What lives in the catalog vs. here

| Concern | Where it lives | Shape |
|---|---|---|
| The deploy command (what to run) | `catalog/services/<repo>.yaml` → `deploy:` | A plain command string, OR `{skip: true, reason: ...}` for repos with no deploy target (libraries / CLIs). |
| Environments, promotion order, *when* and *what triggers* a deploy | **This file** (`workflows/deploy.md`) | Prose policy, harness-readable. |

`ws` is deliberately **not** a deploy engine. It records the command and points the
harness here for the rules. The harness (Claude Code, etc.) is responsible for executing
deploys according to this policy; when in doubt, the harness **asks the customer
(human)** — envs and approval gates are a human decision.

---

## 2. Environments

Every service that deploys MUST define its environment ladder here, in promotion order.
The default corporate ladder is:

1. **`local`** — developer machine (the `dev` command; never "deployed", only run).
2. **`staging`** — integration environment; promoted automatically on merge to the
   default branch (see §3). Reachable for smoke tests; secrets sourced from the
   environment's secret store, never committed.
3. **`production`** — the live environment. Promoted **only** after staging smoke is
   green **and** a human approves.

> Service teams may override the ladder (e.g. add a `qa` tier, or collapse to a single
> `production` for an internal tool) by appending a **Per-service envs** section below.
> Until a service has such a section, the default ladder applies.

### Per-service envs
<!-- Append one subsection per service that diverges from the default ladder. -->
<!-- Example:
### `svc-repo`
- staging: auto on merge to `main`; deploy command `npm run deploy:staging`.
- production: manual gate; deploy command `npm run deploy:prod`.
-->

---

## 3. When and what triggers a deploy

| Environment | Trigger | Gating |
|---|---|---|
| `staging` | Push/merge to the repo's **default branch**. | `repo.verify` green on the merged commit (install → dev → test → test_integration → agent_verify; see `workflows/repo-verify.md`). |
| `production` | **Manual** (human) or a tagged release (`v*`). | staging deploy succeeded + Understand-Anything artifact (`#1`) refreshed on the merged commit + human approval. |

Rules (locked):

- **No hardcoding `main`.** "Default branch" means `github.event.repository.default_branch`
  (dynamic). This matters for repos on `master` / `develop` / `trunk`.
- **`deploy` is excluded from `repo.verify` and `repo.fix_loop.prompt`.** Running deploy
  inside a verification/discovery loop is dangerous and is refused by `repo.run`.
- **Secrets never live in the catalog.** The `deploy:` command string references CI /
  runtime secrets by name (e.g. `${{ secrets.DEPLOY_TOKEN }}`); the values live in the
  provider's secret store.
- **Skipped deploys are explicit.** A repo with no deploy target (library / CLI) declares
  `deploy: {skip: true, reason: ...}`. The healthcheck reports this as `declared`
  (satisfied), not `missing`.

---

## 4. How the harness uses this file

1. When `repo.healthcheck` reports `#8 deploy` as `not_declared`, the harness follows the
   `#8` remediation template in `workflows/repo-init.md`, then **reads this document** to
   decide which environment ladder applies to the service.
2. Before any production deploy, the harness **asks the human** to approve (envs/approval
   gates are a human decision per the `ws`/harness/customer split). `ws` never approves.
3. The harness records the chosen `deploy:` command via `catalog.service.update` and, if
   the service diverges from the default ladder, appends a **Per-service envs** subsection
   here (a catalog write the harness performs via a normal PR).

---

## 5. Change log

- 2026-06-27 — Initial policy. Default ladder (local/staging/production); staging auto on
  default-branch merge, production manual or tagged; deploy excluded from verify/fix-loop.
