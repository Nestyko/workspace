# CLI-Centric Restructure — Decision Record

> Status: **In progress — paused mid-grill.** This document records decisions reached so far
> and the codebase findings that grounded them. Open questions are listed at the end.
> Date: 2026-07-13 · Branch: `feat/cli-centric`

---

## 1. Objective

Release the workspace as **just the CLI**. Today the repo ships a flat top-level layout of
data folders (`catalog/`, `config/`, `schemas/`, `templates/`, `workflows/`, `agents/`,
`docs/`, `skills/`) alongside the Rust workspace. We are collapsing that into:

- The **binary** as the single, self-contained release artifact.
- Canonical **asset sources** embedded into the binary at build time.
- A thin **`docs/`** layer: the folder contract + command reference.
- The existing **`skills/`** release path, untouched.

`cargo install --path crates/ws-cli` must produce a binary that can, on its own, scaffold a
**complete and valid** workspace — no manual copy of extra folders out of git.

---

## 2. Codebase findings (grounding evidence)

These are facts discovered by reading the code, not opinions. They drive every decision below.

### 2.1 The binary already ships "just the CLI" — no assets are compiled in today

- There is **zero** `include_str!` / `include_dir!` / `include_bytes!` usage anywhere in `crates/`.
- `cargo install --path crates/ws-cli` (per `README.md`) installs only the `ws` binary.
- All data folders are read **at runtime from the workspace root** (`std::env::current_dir()`),
  never from the source tree.

### 2.2 But `ws init` does NOT scaffold a complete workspace — latent breakage

`crates/ws-cli/src/main.rs` `handle_init` (≈ lines 390–500) writes only:

- `catalog/{services,products,teams,knowledge}/` dirs (via `ws_catalog::ensure_catalog_dirs`)
- a single `templates/service.yaml` (embedded string literal)
- one sample `ProductCatalog` ("cosell") + one sample `TeamCatalog` ("platform")
- `.ws-config.local.yaml`

It does **not** scaffold `workflows/`, `config/providers/`, `schemas/`, `docs/`,
`agents/`, `catalog/knowledge/SCHEMA.md`, or the knowledge wiki skeleton.

### 2.3 Runtime code assumes those missing folders exist — the latent bug

- `ws-providers/src/lib.rs:362` — reads `config/providers/*.md` at runtime to inject provider
  instructions into `AGENTS.md`.
- `ws-providers/src/lib.rs:460` — does `fs::read_dir(root.join("workflows"))` at runtime to
  build the workflow link list in AGENTS.md.
- `ws-providers/src/lib.rs:484–489` — counts `catalog/{services,products,teams}` for the
  AGENTS.md snapshot.
- `ws-repo/src/fix_loop.rs` + `healthcheck.rs` — emit prompts referencing
  `workflows/repo-init.md` and `workflows/repo-verify.md` as if present.

**Consequence:** a fresh `ws init` workspace is *broken* for `ws ai run provider.config.sync_instructions`
and for `repo.fix_loop.prompt`, because `workflows/` and `config/providers/` are absent.
Embedding closes this real, latent bug.

### 2.4 The committed schemas/docs are `schemars` output, not source

- `schemas/{product,service,team,workspace}.schema.json` are draft-07 JSON Schema,
  matching the `schemars::gen::SchemaSettings::draft07()` generator already used at runtime
  in `crates/ws-core/src/command.rs:58–65` for AI command I/O schemas.
- Every catalog entity struct (`ServiceCatalog`, `ProductCatalog`, `TeamCatalog` in
  `ws-core/src/models.rs`) and every workspace struct (`ws-workspace/src/lib.rs`)
  already derives `JsonSchema`.
- `docs/command-api.md` is the documented output of `ws ai docs generate`
  (`crates/ws-cli/src/main.rs:202,887`).
- **No `ws schema generate` command exists today.** Only `ws ai schema command <id> input|output`
  exists — it covers AI command I/O, not catalog-entity validation schemas.

### 2.5 Build baseline is green

`cargo build -p ws-cli` finishes with warnings only (unused `_ctx`, dead code in
`ws-provider-jira`). Clean baseline to restructure from.

---

## 3. Decisions reached

### Decision 1 — Fork (A): Embed-and-scaffold

**The binary becomes the single source of truth.** `ws init` regenerates a complete, valid
workspace from the binary alone.

Accepted over:
- (B) document-only (won't fix the latent `workflows/`/`config/providers/` breakage; leaves
  every `cargo install` consumer with a broken workspace).
- (C) hybrid (splits the source of truth; reintroduces drift risk).

**Why:** closes the real latent bug in §2.3; guarantees the release artifact reproduces a
correct workspace; eliminates drift between committed copies and runtime expectations.

### Decision 2 — Per-tier asset disposition

The top-level folders are **not all the same kind**. We split them into four tiers:

| Tier | Folders | Disposition |
|---|---|---|
| **1. Runtime-read** | `workflows/*.md`, `config/providers/*.md` | **Embed** into the binary. These are read at runtime (§2.3) but not scaffolded by `ws init` — the latent bug. Embedding fixes it. |
| **2. Generated artifacts** | `schemas/*.json`, `docs/command-api.md` | **Make the binary generate them; delete the committed copies.** Add a new `ws schema generate` subcommand (catalog entity schemas via the existing `schemars` derives). Keep `ws ai docs generate` as the source for `command-api.md`. `catalog.validate` validates against embedded/generated schema bytes — same source of truth as the Rust structs. |
| **3. Scaffolding seeds** | `templates/*.md`+`.yaml`, `catalog/knowledge/SCHEMA.md` + wiki skeleton, `config/default.yaml`, `agents/*.md` | **Embed as starters.** `ws init` writes them once; user owns them thereafter (do not overwrite on re-init — see Open Question Q4). |
| **4. Separate release / out of scope** | `skills/`, `.agents/`, `skills-lock.json`, `tmp/` | **Leave as-is.** `skills/` is released independently via `bunx skills add .` (`README.md:144`). `tmp/` is gitignored — local delete only. |

### Decision 3 — Single asset location: `crates/ws-cli/assets/`

All canonical asset sources move under `crates/ws-cli/assets/`. This is the only location
from which `$CARGO_MANIFEST_DIR` resolves cleanly at compile time without a `build.rs` shim,
because `ws-cli` is the sole consumer. The repo's top level collapses to:

```
crates/            # Rust workspace (unchanged)
docs/              # folder contract + command reference (regenerated command-api.md lives here at runtime, not committed)
skills/            # separate bunx release (untouched)
README.md
LICENSE
Cargo.toml
```

---

## 4. Open questions (not yet decided)

These were raised during the grill but the user paused before confirming.

### Q2 (carryover, recommended) — Tier 2 deletion

**Pending confirmation** that `schemas/*.json` and `docs/command-api.md` should be deleted
from git and generated by the binary instead. This is the most opinionative call. If any
committed `.json`/`.md` must remain as hand-curated reference, surface it now.

### Q3 — Embedding mechanism ✅ RESOLVED

**Decision:** adopt the **`include_dir` crate** (`include_dir!("$CARGO_MANIFEST_DIR/assets/...")`),
whose `Dir::entries()` iterator maps 1:1 onto a recursive `fs::write` scaffold loop — the same
embedded tree powers both runtime reads (Tier 1) and `ws init` scaffolding (Tier 3). Rejected:
`rust-embed` (heavier; no MIME/compression needs) and manual `include_str!` per file (high drift
risk — the exact problem we're killing).

**Tracking:** filed as a standalone adoption ticket (the `include_dir` dependency +
`crates/ws-cli/assets/` establishment); the `ws kb` epic's crate-skeleton ticket is blocked by it.

### Q4 — Re-init semantics ✅ RESOLVED

**Decision:** skip-by-default — `ws init` skips any Tier-3 starter that already exists
(preserving user edits), creating only missing dirs/seed files. A `ws init --reset <asset>`
(or `ws kb init --reset <asset>` for the KB subcommand) provides explicit single-asset refresh.
Only Tier 1 (runtime-read `workflows/`, `config/providers/`) is refreshable; Tier 3 (templates,
knowledge wiki, agents) is user-owned after first write.

### Q5 — Folder-contract documentation scope ✅ RESOLVED (deferred)

**Decision:** `docs/` will contain `docs/folder-contract.md` — purpose + required files +
schema ref per folder (`catalog/`, `config/`, `workflows/`, `agents/`, `templates/`). No
committed `command-api.md` (regenerated on demand via `ws ai docs generate`). **Deferred** —
not in the `ws kb` feature scope; pull forward when Tier 1/2 scaffolding work needs the contract
documented.

### Q6 — `ws schema generate` surface ✅ RESOLVED

**Decision:**
- `ws schema generate [service|product|team|workspace] [--out schemas/]`
- default: write all four into `<root>/schemas/`
- `catalog.validate` reads the generated bytes at validation time rather than reading
  committed files.

Not in the `ws kb` feature scope — Tier 2 work.

---

## 5. Resulting target layout (after restructure)

```
ws/
├── Cargo.toml              # workspace
├── crates/
│   ├── ws-cli/
│   │   └── assets/         # CANONICAL asset sources (embedded at build)
│   │       ├── workflows/
│   │       ├── config/providers/
│   │       ├── templates/
│   │       ├── catalog-knowledge/   # SCHEMA.md + wiki skeleton
│   │       ├── config-default.yaml
│   │       └── agents/
│   └── (other crates unchanged)
├── docs/
│   ├── folder-contract.md
│   └── commands.md
├── skills/                 # separate release (untouched)
├── README.md
└── LICENSE
```

Top-level folders removed (content moved to `crates/ws-cli/assets/` or, for Tier 2, deleted
in favor of generation): `catalog/`, `config/`, `schemas/`, `templates/`, `workflows/`,
`agents/`, `tmp/`, `.agents/`, `skills-lock.json`.

`docs/command-api.md` is deleted from git and regenerated by `ws ai docs generate`.

---

## 6. Status

- ✅ Decision 1, 2, 3 recorded.
- ✅ Q3 resolved (`include_dir`) **and shipped** — `include_dir@0.7` added to the workspace, `crates/ws-cli/assets/` established as the canonical embedded-asset location, macro wired (`ws_cli::assets::ASSETS`), smoke test green. Standalone adoption ticket landed.
- ✅ Q4 resolved (skip-by-default + `--reset <asset>`; Tier 1 refreshable, Tier 3 user-owned).
- ✅ Q5 resolved (`docs/folder-contract.md`), deferred out of `ws kb` scope.
- ✅ Q6 resolved (`ws schema generate [service|product|team|workspace] [--out schemas/]`), Tier 2.
- ⏸️ Q2 (Tier-2 deletion of committed schemas) still pending confirmation.
- ⏸️ Q2 only remaining. Code changes begun: Q3 mechanism shipped, `ws kb` epic underway (tickets filed in dex).
