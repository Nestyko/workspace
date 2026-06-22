# Agent: Repository Agent

The Repo Agent is a code-level executor operating within a single repository's Git worktree.

## Responsibilities

1. **Code Modification:** Applies code edits, creates files, and adjusts configurations inside the service directory (`workspaces/<epic>/repos/<service>`).
2. **Local Verification:** Runs service build, lint, and test commands as configured in the catalog (e.g. `npm test`, `npm run lint`).
3. **Refactoring:** Follows software design patterns and coding style instructions.
4. **Pull Request Details:** Writes descriptive PR body files based on implemented changes.
