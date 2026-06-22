# Jira Provider Instructions (Customer-Specific Config)

This configuration describes how the AI agent should create and map issues in Jira for this project.

## Issue Configurations

### Project Mapping
- Default project: `PLATFORM`

### Issue Types
- **Epic**: Use for features that span multiple PRs or services.
- **Story**: Use for user-facing changes. Ensure description uses the following format:
  ```text
  As a [User]
  I want to [Action]
  So that [Benefit]
  ```
- **Task**: Use for refactoring, CI/CD, or pure backend work.

### Custom Fields
- Security Label: Ensure the label `sec-review-pending` is added when the issue description touches authentication or session tokens.
