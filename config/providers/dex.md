# Dex Provider Instructions (Customer-Specific Config)

This configuration describes how the AI agent should create and map issues using
[`dex`](https://dex.rip), the local, file-backed issue tracker.

## Why Dex

Dex is a **fully local** task tracker: no account, no domain, no API tokens, no
network. Tasks live as a JSONL file under `.dex/tasks.jsonl` in the git repo (or
`~/.dex` as a fallback). This makes it ideal for solo developers or teams that
want issue tracking to stay local to the development machine.

## Issue Configurations

### Project Mapping
- Dex has no notion of a "project". The `project_key` reported on issues defaults
  to `dex`. It exists only so the rest of the workspace tooling keeps a consistent
  shape across providers (Jira, Linear, …).

### Issue Types
Dex tasks are hierarchical (`parent_id` links a task to its parent):

- **Epic**: a top-level dex task (no `parent_id`). Mapped from `create_epic`.
- **Task / Story**: a dex task with a parent. Mapped from `create_issue` when an
  `epic_key` is supplied, otherwise a standalone task.

### Status Mapping
| Workspace status |`dex` action                                  |
|------------------|----------------------------------------------|
| `To Do`          | (default — task exists but not started)      |
| `In Progress`    | `dex start <id>`                             |
| `Done` / `Completed` / `Closed` / `Resolved` | `dex complete <id> --result "..." --no-commit` |

### Comments
Dex has no first-class comments. Comments are appended to the task description under
a `--- Comment ---` separator so the context is preserved with the task body.

### Linking Issues
`link_issues` maps to a dex **blocking dependency**: the outward task blocks the
inward task (`dex edit <inward> --add-blocker <outward>`).

## Environment
The `dex` CLI must be on `PATH`. Install it from https://dex.rip.

Override task storage with the `DEX_STORAGE_PATH` environment variable if you want
tasks stored outside the default git-root location.
