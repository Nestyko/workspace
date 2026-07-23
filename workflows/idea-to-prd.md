# Workflow: Idea to PRD

This workflow details the process of translating a high-level product feature request or idea into a structured Product Requirement Document (PRD).

## Steps

1. **Query Context Resolution:**
   Run the AI context resolution command to find matched products, services, and teams:
   ```bash
   ws ai run context.resolve --input '{"query": "Build a Slack agent for Acme"}'
   ```
2. **Review Matched Catalogs:**
   - Inspect the returned product catalog details (e.g. `catalog/products/acme.yaml`).
   - Inspect the recommended services (e.g. `catalog/services/notification.yaml`).
   - Identify knowledge spaces (e.g. Confluence spaces or Jira projects).
3. **Draft the PRD:**
   - Use the PRD markdown template located at `templates/prd.md`.
   - Document the user stories, architectural flows, and required cross-repository changes.
4. **Identify Out-of-Scope Elements:**
   - Clearly delineate which microservices or features will *not* be modified.
