# AI Workspace CLI ![Alpha Version](https://img.shields.io/badge/version-alpha-red)

An open-source Rust CLI control plane for managing AI-friendly multi-repository workspaces.

> [!WARNING]
> This project is currently in **alpha** state. It is under active development, and features may change or break. Use at your own risk.

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

## Installation

### 🤖 AI Agent Copy-Paste Installation
If you are using an agentic coding assistant (like Claude Code, Cursor, Copilot, or Antigravity), you can copy and paste the following prompt to let the agent compile, install, and configure the tool for you:

> [!TIP]
> **Prompt for AI Agent:**
> ```text
> Please clone the repository from https://github.com/Nestyko/workspace.git, build and install the `ws-cli` package so that the `ws` binary is globally available. Ensure Cargo's bin directory is in my PATH, and verify that running `ws --help` outputs the help page successfully.
> ```

### 🛠️ Manual Installation
To build and install the `ws` binary manually:

1. **Ensure Rust is installed:**
   Ensure you have Rust and Cargo installed. If not, follow the instructions at [rustup.rs](https://rustup.rs/).

2. **Install the binary:**
   From the workspace root directory, compile and install the CLI:
   ```bash
   cargo install --path crates/ws-cli
   ```
   This compiles the `ws-cli` package and installs the executable under the name `ws` into your local Cargo binary directory (typically `~/.cargo/bin`).

3. **Configure your PATH:**
   Ensure that `~/.cargo/bin` is in your shell's search path. If it isn't, add the following to your profile file (e.g., `~/.zshrc`, `~/.bashrc`, or `~/.profile`):
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

4. **Verify installation:**
   Run the help command to make sure it is installed and executable:
   ```bash
   ws --help
   ```

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

---

## Agent Skills

This repo ships two harness-agnostic [Agent Skills](https://agentskills.io/) that turn the
`ws` CLI into an end-to-end AI workspace quality system:

| Skill | Purpose |
|---|---|
| **ws-init** | Bootstrap a fresh `ws` workspace in an empty folder using only non-interactive `ws` commands (replaces the TTY-only `ws init` for headless agents). |
| **ws-self-heal** | Run the repo quality loop — Mode A: all-repos autonomous report-only; Mode B: single-repo human-guided fix. |

Both skills live under [`skills/`](skills/). They are **manual-run only**
(`disable-model-invocation: true`): they are never loaded on init and never fill the
context window — invoke them on demand (e.g. `/skill:ws-init`).

Install both into any supported agent with a single command, from the repo root:

```bash
bunx skills add .
```

> The install command is **`add`**, not `install`. See [`skills/README.md`](skills/README.md)
> for harness-specific notes and a manual fallback.
