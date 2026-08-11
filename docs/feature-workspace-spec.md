# Feature Workspaces — Spec Draft (v0.3)

> Status: **brainstorm — not built.** Design doc for the "start a feature / pick it
> back up" experience on top of the existing epic workspace machinery. Build later.
> Date: 2026-08-10 · Applies to: `ws` CLI in this repo.
>
> **Changelog:**
> - v0.3: identity is slug-first, `ticket` optional (rename-on-attach). Editor-opening
>   demoted; primary surface becomes **`ws tasks` list + select** (the "what are my open
>   tasks / which one do I work in" CLI). Resume context is one minimal `AGENTS.md`.
> - v0.2: task worktrees grouped per-task (`tasks/<task>/<repo>`); deps/env linking and
>   cleanup removed from scope; added `ws open` selection (removed again in v0.3).

## 1. Goal

When starting a feature:

1. **Name the workspace after the feature** — `realtime-chat`, or `EPIC-123-realtime-chat`
   once a ticket exists. No ticket? The name is made on the fly from the prompt; rename
   later when the epic/ticket number arrives. Renames are safe (identity is not the path).
2. **Resume story**: a CLI that answers "**what are my open tasks?**" and lets you pick one,
   then `cd` into its folder. The agent keeps the picked task in context. Each workspace stores
   a short description from the prompt, so the list is semantically searchable and any single
   row can be expanded into full detail.
3. **One git worktree per service**, and per sub-task (`tasks/<task>/<repo>`), so work stays
   parallel and isolated.

Explicitly **out of scope**: dependency/artifact linking, env linking, workspace
cleanup/close, and editor-opening workflows — we work in the folder with the AI agent.

## 2. Grounding — what exists today (verified in code)

- `crates/ws-workspace/src/lib.rs`:
  - `get_workspace_dir(root, epic_key) = root/workspaces/<epic_key>` — **the single choke point**
    for the folder name. Every load/save/status paths through it.
  - `create_epic_workspace` clones **bare mirrors** to `.cache/repos/<owner>/<name>.git`, then
    `git worktree add -b <epic> <base>` into `workspaces/<epic_key>/repos/<service_id>`. Writes
    `workspace.yaml` (`id`, `services`, `base_branch`, `create_branches`, `editor`), `locks.yaml`
    (per-service `baseline_commit`), `<epic>.code-workspace`.
  - `add_service_to_epic_workspace` grows a workspace; `workspace.status` reports per-repo branch/status.
- **Identity today is `epic_key` (required, and it IS the dir name + branch).** This is the
  thing that must loosen for the no-ticket flow.
- Issue provider here is **dex/beads**: issues have a `name` (summary) fetchable via
  `provider.issue.get_issue` → seed for the folder slug (when a ticket exists).
- `ws status` (main.rs ≈1294) lists active workspaces by reading `workspaces/` — the seed for the tasks list.
- Agent context centralized at ws root (KB under `catalog/knowledge/wiki`, catalogs in
  `catalog/*`); `context.resolve` maps a feature query → products/services/teams.

## 3. Proposed layout

```
workspaces/
├── realtime-chat/                   # no ticket yet: <slug(title)> from the prompt
│   └── ...
└── EPIC-123-realtime-chat/          # ticket attached: <TICKET>-<slug>
    ├── workspace.yaml               # id: realtime-chat, ticket: EPIC-123, title, services[], base_branch
    ├── locks.yaml                   # per-service baseline commits (unchanged)
    ├── AGENTS.md                    # NEW — one minimal resume file (agent, auto-loaded on cd)
    ├── repos/                       # main worktrees, one per service, branch <ticket|slug>
    │   ├── api/
    │   └── web/
    └── tasks/                       # task worktrees, grouped per-task (slice 3)
        └── task-a/                  # branch <id>/task-a
            ├── api/
            └── web/
```

## 4. Design decisions

### 4.1 Naming & identity (Slice 1)

- **Folder name** = `kebab(title)` when no ticket; `TICKET-kebab(title)` once assigned.
  - `<slug>` is generated from the feature prompt/title (`realtime-chat`).
  - Ticket prefix (`EPIC-123-`) is added for uniqueness + grep-ability once assigned.
  - Renaming on attach (unguarded `mv`) is safe: nothing locates the workspace by folder name.
- **Model** (`Workspace`):
  - `id: String` — **the slug** (stable identity; folder name is derived, not identity).
  - `ticket: Option<String>` — the issue key, if any (was `epic_key`, now optional).
  - `title: String`, `description: String` (free-text from the prompt/PRD; powers search),
    `services: []`, `base_branch`, `create_branches`, `editor` (unchanged).
  - Optional `tasks: [{ key, slug, repos: [...] }]` added in slice 3.
- **Lookup by content** (`resolve_workspace_dir(root, q)`): scan `workspaces/*/workspace.yaml`,
  match `q` against `id` (slug) OR `ticket`. Path-prefix `{ticket}-`/`{slug}-` as fallback.
  Reject ambiguous matches. Q7 decision: scan, no manifest.
- **Branch naming**: `<ticket>` if assigned, else `<slug>` (stable, PR-linkable; not title-derived).

### 4.2 Rename-on-attach (Slice 1)

- New command path `ws attach <q> --ticket EPIC-123` (or fold into `workspace.create`
  update): sets `workspace.ticket`, rewrites folder `workspaces/<slug>` → `workspaces/EPIC-123-<slug>`,
  moves the worktrees with `git worktree move`, rewrites the code-workspace filename if kept,
  updates `locks.yaml`/`AGENTS.md`. Since lookups are by content, nothing else changes.
- While not attached, `workspace.status`/tasks show the workspace as **unticketed**.

### 4.3 `ws tasks` — list, inspect & select (Slice 2, the primary surface)

The "what do I have open" CLI. No editor involvement; the answer is a path to `cd` into.
Three verbs: list (one-liners), show (expand a row), open (print path + `cd`).

```bash
ws tasks                    # list open workspaces (and their tasks, once present)
ws tasks show <token>       # expand one workspace/task: full detail, no cd needed
ws tasks open <token>       # print path of workspace/task matching <token>; you cd
ws tasks <token>            # shorthand for `tasks open` — prints `workspaces/<dir>`; you cd
```

- **List row**: `slug | ticket | description(truncated) | repos | branch | task` +
  last-modified, with a dirty indicator per repo (cheap `git status --porcelain`).
  Description travels to the right so the line stays scannable; grep-able output.
- **`<token>` matching is semantic, not just identity**: resolves by `id` (slug), `ticket`,
  path prefix, index — **or a substring/word match against `description`**. So
  `ws tasks realtime` finds the unticketed, oddly-named workspace whose description says
  "realtime chat agent". Ambiguous → list the candidates with indices, ask for one.
- **`ws tasks show <token>`** expands a single row:
  - slug, title, ticket (or "unticketed"), full `description`
  - created + last-modified, resolved absolute path
  - base_branch, and per service: `repos/<service>` path, branch, dirty/unpushed counts, baseline commit
  - tasks (slice 3): per-task `key`/`slug`, its repos, branches, status
- **Q9 answer (from user):** selecting a task is just that — pick it, tell the agent to keep
  it in context. No editor, no guesswork: `open` targets the main `repos/<repo>` unless the
  token matches a task (`tasks/<task>/<repo>`).

### 4.4 Task-level worktrees (Slice 3)

- Motivation: one feature, several parallel sub-tasks → one worktree per (task, repo).
- **Layout (decided): per-task grouping** — `tasks/<task-slug>/<repo>` (e.g.
  `tasks/task-a/api`, `tasks/task-a/web`). Groups a task's repos, matching "a task may
  expand into sub-tasks."
- **Git constraint (free with this layout):** the only forbidden shape is a worktree nested
  inside another worktree of the same repo. `repos/api` (main) + `tasks/task-a/api` are
  sibling paths → valid. Keep one worktree per (task, repo).
- New `workspace.add_task` (or the create flow) makes worktrees for the task's repos on
  branch `<id>/<task-slug>`, records baseline in `locks.yaml`. `workspace.status`/`ws tasks`
  report per-task branch/status.

### 4.5 Orchestration ("I start a feature, it infers…")

- *Inference* is the agent's job, driving `ws` for mechanics: read PRD/prompt →
  `context.resolve` → pick `services` → `workspace.create` (now auto-titles **and captures a
  `description`** from the prompt, no ticket required) → `workspace.add_task` per sub-task →
  `ws tasks` to resume.
  `ws` stays a deterministic oracle; no monolithic `ai start` command.
  `workflows/issue-to-implementation.md` updated to codify the loop.

## 5. Build order (each slice independently useful)

1. **Naming + identity** — `Workspace.id`→slug, `ticket` optional, `resolve_workspace_dir`
   scan, `workspace.create` title-from-prompt (no ticket required), `workspace.yaml` fields,
   `workspace.status` shows unticketed. Touches the choke point only.
2. **`ws tasks` list/show/select** — `ws tasks` (list with description), `ws tasks show <token>`
   (expand detail), `ws tasks open <token>` (semantic matching incl. description),
   uncommitted-works indicator, resolved-path printing for `cd`.
3. **Rename-on-attach** — `ws attach <q> --ticket EPIC-123` (move dir + worktrees, rewire fields).
4. **Task worktrees** — `Workspace.tasks`, `workspace.add_task`, `tasks/<task>/<repo>` layout,
   status/tasks integration.

Editor-opening (`ws open`, `editor.open`, `.code-workspace`) is left as-is — not part of the
primary flow, and not removed (existing behavior unchanged).

## 6. Non-goals (explicit)

- Dependency/artifact sharing (`node_modules`/`target`/`.venv` linking, shared stores).
- `.env` copy/link handling.
- Workspace close/cleanup (`ws close`, archive, purge).
- Editor-opening workflows as the resume path (kept only for backward compatibility).
- Multi-machine / shared caches; UI/editor plugin work.



