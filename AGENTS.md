# AI Workspace Rules for Autonomous Coding Agents

Welcome Agent! This document defines your behavioral boundaries, rules, and guidelines when working in this multi-repo workspace.

## Core Rules

1. **Use the JSON API:** Always prefer running commands via the `ws ai run <command_id> --input <file>` interface rather than executing manual git or file operations, unless specifically instructed. This ensures workspace lockfiles (`locks.yaml`) and workspace configs (`workspace.yaml`) remain in sync.
2. **Grow the Catalog Incrementally:** Do not edit global catalog configurations. If you introduce or work with a new service or repository, create a separate YAML file for it under `catalog/services/<repo-name>.yaml`.
3. **Keep Workspaces Disposable:** Local epic workspaces created under `workspaces/` are temporary, generated environments. Do not store permanent configuration, logs, or uncommitted work outside of git repositories or the `.ws` config directory.
4. **Preserve Baseline Commits:** Always reference `baseline_commit` inside `locks.yaml` when analyzing changes or creating pull requests.
5. **Always Validate:** Before committing new catalogs, run `ws ai run catalog.validate --input '{}'` to ensure parsing schemas are fully respected.

## Workflow Rules

- Refer to the workflows documented under `workflows/` for step-by-step processes:
  - [workflows/idea-to-prd.md](file:///Users/nestyko/Documents/playground/ws/workflows/idea-to-prd.md)
  - [workflows/prd-to-issues.md](file:///Users/nestyko/Documents/playground/ws/workflows/prd-to-issues.md)
  - [workflows/issue-to-implementation.md](file:///Users/nestyko/Documents/playground/ws/workflows/issue-to-implementation.md)
  - [workflows/cross-repo-review.md](file:///Users/nestyko/Documents/playground/ws/workflows/cross-repo-review.md)
