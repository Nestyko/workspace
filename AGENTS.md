# AI Workspace Rules for Autonomous Coding Agents

Welcome Agent! This document defines your behavioral boundaries, rules, and guidelines when working in this multi-repo workspace.

## Core Rules

1. **Use the JSON API:** Always prefer running commands via the `ws ai run <command_id> --input <file>` interface rather than executing manual git or file operations, unless specifically instructed. This ensures workspace lockfiles (`locks.yaml`) and workspace configs (`workspace.yaml`) remain in sync.
2. **Grow the Catalog Incrementally:** Do not edit global catalog configurations. If you introduce or work with a new service or repository, create a separate YAML file for it under `catalog/services/<repo-name>.yaml`.
3. **Keep Workspaces Disposable:** Local epic workspaces created under `workspaces/` are temporary, generated environments. Do not store permanent configuration, logs, or uncommitted work outside of git repositories or the `.ws` config directory.
4. **Preserve Baseline Commits:** Always reference `baseline_commit` inside `locks.yaml` when analyzing changes or creating pull requests.
5. **Always Validate:** Before committing new catalogs, run `ws ai run catalog.validate --input '{}'` to ensure parsing schemas are fully respected.

## Workflow Rules

- Refer to the workflows documented under `workflows/` for step-by-step processes:
  - [workflows/idea-to-prd.md](file:///Users/nestyko/Documents/playground/ws/workflows/idea-to-prd.md)
  - [workflows/prd-to-issues.md](file:///Users/nestyko/Documents/playground/ws/workflows/prd-to-issues.md)
  - [workflows/issue-to-implementation.md](file:///Users/nestyko/Documents/playground/ws/workflows/issue-to-implementation.md)
  - [workflows/cross-repo-review.md](file:///Users/nestyko/Documents/playground/ws/workflows/cross-repo-review.md)
  - [workflows/knowledge-base.md](file:///Users/nestyko/Documents/playground/ws/workflows/knowledge-base.md)

## Product Knowledge Base

The company product side maintains an LLM-maintained wiki under `catalog/knowledge/`, following the [LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f). It is a persistent, compounding artifact: source documents are compiled once into interlinked markdown and kept current over time.

- **Read [`catalog/knowledge/SCHEMA.md`](file:///Users/nestyko/Documents/playground/ws/catalog/knowledge/SCHEMA.md) before maintaining the wiki** — it is the authoritative contract for structure, conventions, and the ingest/query/update/lint operations.
- **Raw sources** go in `catalog/knowledge/raw/` (the human drops files here; it is gitignored and never modified by the agent). External Document sources are also supported — by default **Confluence** (see `config/providers/confluence.md` and each product's `knowledge_sources`).
- **The wiki** (`catalog/knowledge/wiki/`) is agent-owned, version-controlled markdown (`index.md` + `log.md` + topic/entity/source/synthesis pages).
- When brainstorming an idea, use the wiki as the primary context layer. If a new, unconfirmed fact surfaces, **ask the user to confirm it and provide a source before integrating it** — never inject unconfirmed facts as if they were sourced.
- This covers the **product side only**. Do not restructure `catalog/teams/` or `catalog/services/` or mirror their content into this wiki.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:7510c1e2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
