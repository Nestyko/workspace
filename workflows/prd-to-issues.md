# Workflow: PRD to Issues

This workflow describes how a completed PRD is translated into actionable tickets (Epics, Tasks, Stories) in the issue provider (Jira).

## Steps

1. **Jira Authentication:**
   Verify connection credentials with Jira:
   ```bash
   ws ai run provider.issue.check_auth --input '{}'
   ```
2. **Create Epic:**
   Create a parent Epic for the feature implementation:
   ```bash
   ws ai run provider.issue.create_epic --input '{
     "project": "COSELL",
     "name": "Slack Integration",
     "summary": "Support Slack notifications for partner collaboration",
     "description": "Epic tracking all work related to Slack agent integration"
   }'
   ```
3. **Generate User Stories & Tasks:**
   - Create individual user stories or developer tasks associated with the Epic key.
   - For backend/frontend tasks, link them using the link command:
     ```bash
     ws ai run provider.issue.link --input '{
       "inward_key": "COSELL-124",
       "outward_key": "COSELL-125",
       "link_type": "Blocks"
     }'
     ```
