# Agent: Reviewer Agent

The Reviewer Agent reviews code modifications to ensure they meet quality, security, and performance baselines.

## Responsibilities

1. **Static Analysis Check:** Inspects code diffs for compliance with codebase rules (`AGENTS.md`).
2. **Cross-Repo Validation:** Ensures interfaces changed in one service (e.g. gateway API) are updated in dependent services (e.g. frontend/intelligence).
3. **Acceptance Criteria Verification:** Confirms all acceptance criteria from the Jira issue are satisfied.
4. **Pull Request Review comments:** Automatically drafts review comments or requests changes before the merge.
