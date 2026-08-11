# Workflow: Issue to Implementation

This workflow details setting up the multi-repo development environment for a
given feature or epic, using the slug-first feature-workspace machinery.

## Orchestration loop (agent-driven, `ws` is the deterministic oracle)

1. **Read the prompt/PRD** and resolve context:
   ```bash
   ws ai run context.resolve --input '{"query": "<feature description>"}'
   ```
2. **Create the workspace** — no ticket required; the folder slug comes from the
   title, and a `description` seeds `ws tasks` search:
   ```bash
   ws ai run workspace.create --input '{
     "title": "Realtime Chat",
     "description": "Realtime chat agent targetting the web client",
     "ticket": "ACME-123",        # optional — omit while unticketed
     "services": ["intelligence", "notification"],
     "base_branch": "main",
     "create_branches": true
   }'
   ```
   This does the following:
   - Fetches latest updates to `.cache/repos`.
   - Creates separate Git worktrees in `workspaces/<slug-or-TICKET-slug>/repos/<service>`.
   - Checks out a fresh feature branch named `<ticket>` (if assigned) else `<slug>`.
   - Generates `workspace.yaml`, `locks.yaml`, `<folder>.code-workspace`, and a
     minimal resume `AGENTS.md`.
3. **Split into parallel sub-tasks** (one worktree per task × repo on branch
   `<slug>-<task>`):
   ```bash
   ws ai run workspace.add_task --input '{
     "q": "ACME-123",
     "key": "ACME-357",
     "slug": "task-auth",
     "services": ["intelligence"]
   }'
   ```
4. **Resume work** — the agent and human both use the tasks CLI; `open` prints a
   path to `cd` into:
   ```bash
   ws tasks                 # what are my open tasks? (one-liners)
   ws tasks show <token>    # expand a row
   ws tasks open <token>    # print the worktree path; you cd
   ws tasks <token>         # shorthand for `tasks open`
   ```
5. **Attach a ticket later** (rename-on-attach, safe: identity is not the path):
   ```bash
   ws attach realtime-chat --ticket ACME-123   # workspaces/realtime-chat → workspaces/ACME-123-realtime-chat
   ```

`ws` stays a deterministic oracle: the agent reads the PRD, drives these
commands, and keeps the picked task in context. Tokens are resolved by `id`
(slug), `ticket`, folder name/prefix, list index, or a word against the
workspace description.
