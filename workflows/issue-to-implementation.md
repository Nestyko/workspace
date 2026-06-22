# Workflow: Issue to Implementation

This workflow details setting up the multi-repo development environment for a given Epic.

## Steps

1. **Verify Epic Info:**
   Retrieve the epic description and status:
   ```bash
   ws ai run provider.issue.get_issue --input '{"key": "COSELL-123"}'
   ```
2. **Create Workspace:**
   Create the worktrees for the services that need changes:
   ```bash
   ws ai run workspace.create --input '{
     "epic_key": "COSELL-123",
     "services": ["intelligence", "notification"],
     "base_branch": "main",
     "create_branches": true
   }'
   ```
   This does the following:
   - Fetches latest updates to `.cache/repos`.
   - Creates separate Git worktrees in `workspaces/COSELL-123/repos/intelligence` and `workspaces/COSELL-123/repos/notification`.
   - Checks out a fresh feature branch named `COSELL-123`.
   - Generates `workspace.yaml`, `locks.yaml`, and the editor workspace file.
3. **Open in Editor:**
   Open the workspaces in Cursor, VS Code, Zed, or Vim:
   ```bash
   ws ai run editor.open --input '{
     "epic_key": "COSELL-123",
     "editor": "vscode"
   }'
   ```
4. **Implement Changes:**
   Perform coding task inside the worktree folders.
