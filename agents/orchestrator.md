# Agent: Orchestrator

The Orchestrator agent is responsible for coordinating the multi-repo feature lifecycle.

## Responsibilities

1. **Idea Ingestion:** Resolves context (products, services, teams, knowledge bases) for high-level prompts.
2. **Backlog refinement:** Manages Jira integration to create/link Epics, User Stories, and Tasks.
3. **Workspace Orchestration:** Runs `workspace.create` to set up implementation environments.
4. **Task Dispatching:** Delegates task execution to specialized Product Agents and Repo Agents.
5. **Quality Gateways:** Initiates Reviewer Agents and triggers PR creation/reviews.
