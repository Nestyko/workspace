# `ws` Agent Skills

Two harness-agnostic skills that turn the `ws` CLI into an end-to-end AI workspace quality
system. `ws` is the deterministic oracle (measures, executes primitives, validates
writes); these skills are the harness layer that operates it.

## Skills

| Skill | File | Purpose | Modes |
|---|---|---|---|
| **ws-init** | [`ws-init/SKILL.md`](ws-init/SKILL.md) | Bootstrap a fresh `ws` workspace in an empty folder (replaces the interactive `ws init` TTY flow for headless agents) | single |
| **ws-self-heal** | [`ws-self-heal/SKILL.md`](ws-self-heal/SKILL.md) | Run the repo quality loop | **A** all-repos autonomous report-only · **B** single-repo human-guided fix |

### Why both are needed
`ws init` is TTY-interactive and the fix-loop is, by contract, an external harness concern
(`ws` emits the spec and stops). These skills close that gap: the init skill drives setup
through non-interactive `ws` commands while the human customizes in chat; the self-heal
skill runs the healthcheck → fix → record → re-check loop the `ws` primitives define.

## Installing into a harness

Each `SKILL.md` is a self-contained prompt compliant with the
[Agent Skills](https://agentskills.io/) standard (YAML frontmatter: `name`,
`description`, `disable-model-invocation`). Install them with the
[`skills`](https://www.npmjs.com/package/skills) CLI in a single command from the repo
root — it auto-detects installed agents and symlinks each skill into the right directory:

```bash
bunx skills add .
```

Add `-l`/`--list` to preview without installing, `-g` for a global (user-level) install,
`-a <agent>` to target a specific agent, `--skill '*'` for all skills, or `-y` to skip
prompts (e.g. `bunx skills add . --skill '*' -a claude-code -y`).

> [!NOTE]
> The install command is **`add`**, not `install`. Both skills carry
> `disable-model-invocation: true`, so they are **never auto-loaded on init** and never
> appear in the system prompt — they don't fill the context window. Invoke them manually
> only when needed, via your harness's skill command (e.g. in pi: `/skill:ws-init`,
> `/skill:ws-self-heal`).

### Manual / no-CLI fallback
Each `SKILL.md` is plain markdown with no harness-specific runtime. If you can't use the
`skills` CLI, copy or symlink the two directories into your agent's skills dir yourself,
e.g. for pi:
```bash
mkdir -p ~/.pi/agent/skills
ln -s "$(pwd)/skills/ws-init"      ~/.pi/agent/skills/ws-init
ln -s "$(pwd)/skills/ws-self-heal" ~/.pi/agent/skills/ws-self-heal
```
(pi loads any `SKILL.md` under `~/.pi/agent/skills/<name>/` or `.pi/skills/<name>/`.)

### Prerequisites (all harnesses)
- `ws` on PATH with the quality commands (expect 39 in `ws ai manifest`). If fewer,
  reinstall from this repo: `cargo install --path crates/ws-cli`.
- `gh` CLI authenticated (GitHub code provider).
- For self-heal: working git checkouts of the services (`ws` never clones).

## Schema drift note (carry over)
Two source-level inconsistencies remain and are flagged inside each skill so the agent
doesn't lose turns to trial-and-error:
- `ws ai manifest` advertises `ws ai schema command <id> input` but the parser wants
  `ws ai schema <id> input` (drop the word `command`).
- The service identifier field differs: `repo.*` commands use `service_id`;
  `catalog.service.get`/`update` use `id`.
