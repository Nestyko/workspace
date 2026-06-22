# AI Workspace CLI

An open-source Rust CLI control plane for managing AI-friendly multi-repository workspaces.

## Objective

This tool provides a control plane to manage per-epic workspaces across multiple repositories. Rather than maintaining a monorepo, `ws` offers:
- A single workspace root directory.
- A decentralized catalog of products, teams, services, and knowledge sources (stored in YAML, one file per entity).
- Lazy repository discovery and loading.
- Per-epic developer workspaces built on Git worktrees.
- Pluggable issue and code provider abstractions (Jira and GitHub via `gh` CLI provided as first-class MVPs).
- Native editor workspace configurations for Cursor, VS Code, Zed, and Vim.
- A human-facing interactive command line.
- A larger, typed AI-facing JSON Command API with JSON schema validation and manifest listing.

---

## Human CLI Commands

```bash
# Initialize workspace, config, provider credentials, and default editor
ws init

# View config or set parameters
ws config
ws config set editor vscode

# Interactive repository discovery from code provider (GitHub)
ws discover
ws discover --limit 10

# Add catalog records manually
ws add repo example-org/notification
ws add product cosell
ws add team platform

# Open epic workspace or specific service in default editor
ws open COSELL-123
ws open COSELL-123 --service notification

# Show overall workspace and repository status
ws status
ws status COSELL-123

# Push branches and create pull requests for modified repositories
ws pr create COSELL-123 --all
```

---

## AI Command API

AI Agents interact with the workspace via a typed JSON Command interface:

```bash
# Get manifest of all available AI command IDs
ws ai manifest

# Generate command API schema documentation
ws ai docs generate

# Output JSON schema for command input/output
ws ai schema command workspace.create input
ws ai schema command workspace.create output

# Execute an AI command using a JSON input file
ws ai run workspace.create --input ./workspace-create.json
```

All 21 AI commands are documented in [command-api.md](file:///Users/nestyko/Documents/playground/ws/docs/command-api.md).

---

## Architecture & Extensibility

The CLI core (`ws-core`) defines decoupled traits for:
- `CodeProvider` (MVP: `GitHubGhProvider` via `gh` CLI)
- `IssueProvider` (MVP: `JiraProvider` supporting real HTTP and a local mock fallback)
- `EditorAdapter` (Adapters for Cursor, VS Code, Zed, and Vim)

This keeps the core engine decoupled from specific APIs and tool details.
