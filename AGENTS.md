# AI Workspace Rules for Autonomous Coding Agents

Welcome Agent! This document defines your behavioral boundaries, rules, and guidelines when working in this multi-repo workspace. **This file is ws-managed** — regenerate it with `ws ai run provider.config.sync_instructions --input '{}'`. Do not hand-edit the ws-managed sections; append custom integration blocks between dedicated BEGIN/END markers instead.

## Core Rules

1. **Use the JSON API:** Always prefer running commands via the `ws ai run <command_id> --input <file>` interface rather than executing manual git or file operations, unless specifically instructed. This ensures workspace lockfiles (`locks.yaml`) and workspace configs (`workspace.yaml`) remain in sync.
2. **Grow the Catalog Incrementally:** Do not edit global catalog configurations. If you introduce or work with a new service or repository, create a separate YAML file for it under `catalog/services/<repo-name>.yaml`.
3. **Keep Workspaces Disposable:** Local epic workspaces created under `workspaces/` are temporary, generated environments. Do not store permanent configuration, logs, or uncommitted work outside of git repositories or the `.ws` config directory.
4. **Preserve Baseline Commits:** Always reference `baseline_commit` inside `locks.yaml` when analyzing changes or creating pull requests.
5. **Always Validate:** Before committing new catalogs, run `ws ai run catalog.validate --input '{}'` to ensure parsing schemas are fully respected.

## Map

- **Knowledge base** (Karpathy LLM-wiki): `catalog/knowledge/`. Read `catalog/knowledge/SCHEMA.md` before maintaining; `catalog/knowledge/raw/` = human-dropped sources (never edit), `catalog/knowledge/wiki/` = agent-owned pages. Use it as the primary context for product work; have the user confirm and source any unverified fact before you inject it.
- **Catalog** (one YAML file per entity): `catalog/services/<repo>.yaml` = the git repos, `catalog/products/<id>.yaml` = products, `catalog/teams/<id>.yaml` = teams.
- **Tasks**: epic workspaces live in `workspaces/<slug>/`. Start a task when a feature/epic needs implementation: `ws ai run workspace.create` (epic) -> `ws ai run workspace.add_task` (slice) -> work inside the generated worktree. List open tasks with `ws tasks`; resume with `ws tasks show|open <token>`.
- **Runbooks**: step-by-step processes in `workflows/*.md` (repo-init, repo-verify, deploy, issue-to-implementation).