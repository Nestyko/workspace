---
name: ws-init
description: >-
  Bootstrap a fresh ws multi-repo AI workspace in an empty folder using only non-interactive
  ws commands. Use when the user says 'initialize ws', 'set up ws workspace', 'bootstrap ai
  workspace', 'create a new ws workspace', or when working in an empty folder intended to
  become a ws workspace. Replaces the TTY-only interactive `ws init` flow for headless
  agents - the human answers setup questions in chat while the agent applies config, seeds
  the catalog, validates, and regenerates AGENTS.md. Scaffolds config + catalog only; does
  not run healthchecks or fix loops (use ws-self-heal for that).
disable-model-invocation: true
---

# Skill — `ws-init` (Workspace Bootstrap)

> Scaffold a brand-new `ws` AI workspace in an empty folder, end-to-end, using only
> non-interactive `ws` commands. The human answers the setup questions in chat; the agent
> runs the deterministic `ws` commands.

## Description / Triggers

Initialize a `ws` multi-repo workspace. Activate when the user says: "initialize ws",
"set up ws workspace", "bootstrap ai workspace", "create a new ws workspace", or when
working in an empty folder intended to become a `ws` workspace.

## Why this skill exists

`ws init` is interactive-only (TTY prompts) and therefore **cannot be driven by an agent
in a headless harness** (Codex, Claude Code, Cursor, pi). This skill replaces that TTY
flow with: (a) the agent asking the human the setup questions in chat, and (b) the agent
applying them through fully non-interactive `ws` commands. The split is deliberate:

- **Deterministic** (the `ws` commands) — never improvised, always the exact JSON/flag form.
- **Non-deterministic** (the human's answers + repo selection) — customized per workspace.

**Scope boundary:** this skill *only* scaffolds config + catalog. It does **not** run
healthchecks or fix loops — that is the separate `ws-self-heal` skill. Init = scaffold.

---

## Prerequisites

- `ws` binary installed and on PATH. Verify: `ws --help`. If missing, install from the
  `ws` source repo: `cargo install --path crates/ws-cli` (ensure `~/.cargo/bin` is on PATH).
- `gh` CLI authenticated (the code provider is GitHub-via-`gh`). Verify: `gh auth status`.

---

## Procedure

### Step 0 — Pre-flight
Run from the intended workspace root (an empty folder, or accept the user's chosen path).
Confirm `ws` is installed. Then confirm the installed binary actually has the quality
commands (a stale binary predates them):
```bash
ws ai manifest | grep -c '"id"'        # expect 39; if lower, reinstall from source
```

### Step 1 — Ask the human the setup questions (chat, non-deterministic)
Ask, one batch or iteratively, and let the user customize each:

1. **Issue provider** — only `jira` is supported today. Confirm or skip (issue tracking is
   optional for pure code-quality use).
2. **Code provider** — `github-gh`. Confirm.
3. **Default editor** — one of `cursor`, `vscode`, `zed`, `vim`.
4. **GitHub org/user** (`code-owner`) — the default owner for `ws add repo` / discovery.
5. **Jira base URL** (e.g. `https://example.atlassian.net`) — only if issue provider used.
6. **Jira default project key** — only if issue provider used.
7. **Confluence base URL + space key** — only if the doc/knowledge provider is used
   (optional; the harness works without it).

Record the answers. Do not proceed until the user confirms.

### Step 2 — Apply config deterministically (`ws config set`)
All eight keys are non-interactive and write to `.ws/config.yaml`:
```bash
ws config set code-provider github-gh
ws config set code-owner      <owner>
ws config set editor          <cursor|vscode|zed|vim>
ws config set issue-provider  jira          # only if used
ws config set jira-url        <url>         # only if used
ws config set jira-project    <key>         # only if used
ws config set confluence-url  <url>         # only if used
ws config set confluence-space <key>        # only if used
```

### Step 3 — Validate provider auth (warn, don't hard-block)
```bash
ws ai run provider.code.check_auth   --input '{}'
ws ai run provider.issue.check_auth  --input '{}'   # may be unconfigured
ws ai run provider.doc.check_auth    --input '{}'   # may be unconfigured
```
GitHub auth is required (the harness depends on `gh`). Jira/Confluence auth failures are
warned and surfaced to the human; they do not abort init (code quality works without them).

### Step 4 — Seed the catalog (non-interactive)
`ws add` is fully non-interactive (no TTY) and creates the `catalog/<kind>/` dirs on demand:
```bash
ws add team    <TeamName>          # → catalog/teams/<id>.yaml   (id = lowercased name)
ws add product <ProductName>       # → catalog/products/<id>.yaml
ws add repo    <owner>/<repo>      # → catalog/services/<repo>.yaml
```
Ask the human which teams/products/repos to seed, then add each.

#### Discovery (harness-friendly replacement for `ws discover`)
`ws discover` uses a TTY `MultiSelect` the agent cannot drive. Use the JSON listing
instead, present the page to the human in chat, and add their picks explicitly:
```bash
ws ai run provider.code.list_recent_repos --input '{"limit": 50, "page": 1}'
# → array of {full_name, name, owner, ssh_url, default_branch, description, ...}
# present to human → on pick:
ws add repo <owner>/<name>
# increment "page" to fetch the next batch when the human wants more
```

### Step 5 — Validate the catalog
```bash
ws ai run catalog.validate --input '{}'
# expect: {"success": true, "message": "All catalog files are valid."}
```
If invalid, fix the offending `catalog/**/*.yaml` and re-run until green.

### Step 6 — Regenerate the ws-managed AGENTS.md
This writes/refreshes the workspace's behavioral contract for agents (the file that tells
every agent in this workspace the rules + which `ws ai run` commands exist):
```bash
ws ai run provider.config.sync_instructions --input '{}'
```
The result is `AGENTS.md` at the workspace root. Custom integration blocks go between the
dedicated `BEGIN ... / END` markers only — never edit the ws-managed section by hand.

### Step 7 — Hand off
Print a status summary and tell the human the workspace is scaffolded. Point them at the
next step: run the **`ws-self-heal`** skill (single-repo, human-guided, or all-repos
report) to converge each service's healthcheck to green.

```bash
ws status        # human-facing workspace overview
```

---

## Deterministic vs. human — quick reference

| Step | Who | How |
|---|---|---|
| Setup answers (provider, editor, owner, jira/confluence creds) | Human | chat |
| Config apply | `ws config set` (8 keys) | deterministic |
| Auth check | `provider.*.check_auth` | deterministic |
| Catalog seeding | `ws add team/product/repo` | deterministic command; human picks names |
| Repo discovery | `provider.code.list_recent_repos` | deterministic listing; human picks repos |
| Catalog validation | `catalog.validate` | deterministic |
| AGENTS.md | `provider.config.sync_instructions` | deterministic |

## What this skill does NOT do
- No `repo.healthcheck`, no `repo.run`, no fix-loop → use `ws-self-heal`.
- No Understand-Anything / CI setup → human-in-the-loop, deferred to `ws-self-heal` notes.
