# Workflow: Cross-Repository Review

This workflow details pushing modifications across multiple repositories and opening Pull Requests.

## Steps

1. **Verify Workspace status:**
   Check modified files and current commits across the workspace worktrees:
   ```bash
   ws ai run workspace.status --input '{"epic_key": "COSELL-123"}'
   ```
2. **Create Pull Requests:**
   Trigger PR creation for the changed services:
   ```bash
   ws ai run pr.create --input '{
     "workspace_id": "COSELL-123",
     "services": ["notification", "intelligence"],
     "title": "[COSELL-123] Support Slack partner integration",
     "body": "Implement templates and logic for Slack agents.",
     "draft": false
   }'
   ```
   This will:
   - Push local branch commits to the origin remotes.
   - Run `gh pr create` in each worktree.
   - Comment on the Jira epic link with the PR URLs.
3. **Resolve reviews:**
   Once PR reviews are complete, merge them and clean up the worktrees by deleting the workspace directory.
