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

Each `SKILL.md` is a self-contained prompt — load it however your harness ingests skills:

### pi
```bash
mkdir -p ~/.pi/agent-personal/skills
ln -s "$(pwd)/skills/ws-init"      ~/.pi/agent-personal/skills/ws-init
ln -s "$(pwd)/skills/ws-self-heal" ~/.pi/agent-personal/skills/ws-self-heal
```
(pi loads any `SKILL.md` under `~/.pi/agent-personal/skills/<name>/`.)

### Claude Code
Append the skill body to `.claude/CLAUDE.md`, or place the file under a skills dir your
project loads. For a per-project install, drop the two `SKILL.md` files into
`.claude/skills/` (or reference them from the project `CLAUDE.md`).

### Codex / Cursor / other
Paste the `SKILL.md` body into your agent's instructions/config file, or import it as a
custom instruction set. The files are plain markdown with no harness-specific runtime.

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
