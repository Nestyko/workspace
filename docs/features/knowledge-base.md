# Feature Spec — Knowledge Base (`ws kb`)

> Spec authored with the [to-spec](https://github.com/mattpocock/skills/tree/main/skills/engineering/to-spec) pattern.
> Grounded in `docs/cli-centric-restructure-plan.md` (the CLI-centric restructure ADR) and the
> [LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).
> Status: **ready for triage** (file via `bd create` when scheduling).

---

## Problem Statement

Today, `ws init` scaffolds `catalog/{services,products,teams,knowledge}/` but does **not** scaffold
the knowledge-base tree that the harness assumes exists at runtime. The committed
`catalog/knowledge/` (SCHEMA.md, README, `wiki/` skeleton, `raw/` skeleton) lives in the source tree
but is never written by the binary. A consumer who runs `cargo install --path crates/ws-cli` and
`ws init` gets a workspace with **no knowledge base**, even though `ws-providers` already injects
instructions into `AGENTS.md` telling the agent to "read `catalog/knowledge/SCHEMA.md` before
maintaining the wiki." That is a latent bug, not a feature gap.

Beyond the latent bug, the existing knowledge base is scoped to the **company product side only**.
The harness surfaces engineering concepts during implementation work (epics, features, PRs) that
have no home — there is no contract for capturing a concept discovered mid-build, with provenance
pointing at the commits/files/issues that produced it. Product knowledge and engineering knowledge
currently live in separate mental models with no shared compounding artifact.

The user wants a single, agent-maintained knowledge base that (a) is scaffolded by the CLI out of
the box, (b) holds **both** product and engineering knowledge in one wiki, and (c) grows
organically as concepts surface during work — with the harness proposing new concepts to the human
and filing them only on consent, with references tracing where each concept came from.

## Solution

Ship a `ws kb` subcommand namespace backed by embedded assets (Tier-3 scaffolding seeds per the
restructure ADR). The CLI owns **structural consistency**; the agent (following `SCHEMA.md`) owns
**content and operations**. Two CLI commands only:

- **`ws kb init`** — scaffolds the complete knowledge-base tree from embedded canonical assets.
  Skip-by-default on existing files (user content is never destroyed); a `--reset <asset>` flag
  explicitly refreshes a single asset from the embedded copy.
- **`ws kb lint`** — deterministic health check. MVP scope is deliberately minimal: **orphans**
  (wiki pages with no inbound `[[wikilinks]]`) and **broken `[[wikilinks]]`** (links whose target
  slug resolves to no page). Nothing else for MVP.

Everything else — **ingest, query, update, concept capture, the yes/no gate, cross-referencing,
log/index maintenance** — is an **agent operation defined in `SCHEMA.md`**, performed by editing
markdown directly. There is no CLI command for those. The harness detects new concepts per SCHEMA
instructions (not via a `ws kb suggest` hook), gathers a definition, lists its sources, asks the
human for consent, and on approval performs the SCHEMA-defined Ingest workflow by writing markdown.

The knowledge base is a **single wiki** holding both product and engineering streams. The two
streams are distinguished in frontmatter (a `stream` field), not by folder partition — the agent
filters on stream at save/query time, and `[[wikilinks]]` across streams form an implicit graph
(visible in any markdown graph viewer such as Obsidian).

Provenance is a **hybrid model** with two source types:

- **Blob sources** — immutable materialized files in `raw/` (clipped articles, Confluence exports,
  PDFs, transcripts, conversation snapshots). These get a source page in `wiki/sources/` and are
  cited as `[[src-<slug>]]`.
- **Pointer sources** — addresses to things that live elsewhere and are not copied in
  (`commit:<sha>`, `file:<path>#Lstart-Lend`, `issue:<id>`, `pr:<num>`, `web:<url>`). These cite
  **inline as typed tokens**, rendered as clickable markdown links, with **no source page** by
  default. When a cluster of pointers needs narrative bundling, the agent writes an **opt-in,
  concept-keyed** source page in `wiki/sources/` (version-controlled, tagged `external` when the
  page is a cluster of external pointers) and cites it as `[[src-<concept>-evidence]]`.

Conversational evidence is preserved as a **snapshot blob** in `raw/snapshots/` (verbatim
definition + paraphrased narrative, consent-gated), gets a `src-conv-<date>-<topic>` source page,
and is cited as `[[src-conv-...]]`; the bare `conversation:<date>` pointer appears **only on that
source page** as a provenance marker — never as a bare inline token on a topic page.

## User Stories

1. As a **workspace owner**, I want `ws init` to produce a complete knowledge-base tree out of the box, so that I don't have to manually copy `catalog/knowledge/` out of the source repo.
2. As a **workspace owner**, I want the knowledge base scaffolded alongside everything else by `ws init`, so that a fresh workspace is not structurally broken for the harness's knowledge-base instructions.
3. As a **workspace owner**, I want `ws kb init` available as a standalone command, so that I can set up (or repair) the knowledge base independently of a full `ws init`.
4. As a **workspace owner**, I want `ws kb init` to skip files that already exist, so that re-running it never destroys my accumulated wiki content.
5. As a **workspace owner**, I want `ws kb init --reset <asset>` to refresh one embedded asset from the binary (e.g. pull a SCHEMA improvement shipped in a new binary version), so that I can adopt upstream improvements without nuking my wiki.
6. As a **workspace owner**, I want `ws kb init --reset <asset>` to only refresh the named asset, so that an upgrade to one seed never touches my other content.
7. As an **agent (following `SCHEMA.md`)**, I want a documented Ingest operation, so that I know exactly how to register a new source, write its summary, update topic/entity pages, update `index.md`, and append `log.md`.
8. As an **agent**, I want the Ingest workflow to be performed by editing markdown directly (no CLI call), so that the creative judgment of which pages to touch stays with me and the CLI only enforces structure.
9. As an **agent**, I want a documented concept-capture operation, so that when I notice a concept mentioned more than once without a page (or a concept the wiki lacks while work references it), I propose it to the human.
10. As an **agent**, I want the concept-capture proposal to include my draft definition and a list of sources where I got it, so that the human can review provenance before consenting.
11. As an **agent**, I want the human to be able to paste additional sources into the proposal before ingest, so that evidence can be enriched at the gate.
12. As an **agent**, I want the yes/no gate to fire only on new-page creation and on contradictions to existing claims, so that routine refinement edits (adding a citation, extending a summary, fixing a link) stay silent and the wiki does not become interrupt-heavy.
13. As an **agent**, I want routine refinement edits to be logged in `log.md` without a gate, so that the timeline of the wiki's evolution stays complete without nagging the human.
14. As a **human**, I want a consent prompt specifically when the evidence behind a proposed concept is a conversation (not an external artifact), so that I am warned before ephemeral chat context is persisted as a source.
15. As a **human**, I want the verbatim definition preserved when a concept is captured, regardless of whether I typed it or the agent generated it, so that the load-bearing claim is never paraphrased away.
16. As a **human**, I want the surrounding narrative (how the concept surfaced, the back-and-forth) paraphrased rather than verbatim, so that the source note is a deliberate artifact and not a chat-log dump.
17. As a **human**, I want the consent gate's positive action to be the agent performing the SCHEMA-defined Ingest (writing markdown), so that approving a concept makes it official without a separate CLI step.
18. As a **product researcher**, I want to drop a Confluence export or clipped article into `raw/` and have the agent ingest it into the product stream, so that product knowledge compounds in the wiki.
19. As an **engineer**, I want engineering concepts discovered mid-implementation (e.g. a third product-listing type referenced in code but absent from the wiki) to be capturable with provenance pointing at commits, files, issues, and PRs, so that engineering knowledge compounds alongside product knowledge.
20. As an **engineer**, I want pointer sources cited inline as clickable links (e.g. a commit hash that opens the commit on GitHub), so that I can jump straight to the evidence behind a claim.
21. As an **engineer**, I want claim-level provenance preserved (each sentence cites the specific commit/file that backs it), so that I never have to guess which evidence in a bundle supports a given claim.
22. As an **engineer**, I want the agent to optionally write a concept-keyed source page when a cluster of pointers needs narrative explanation, so that the *relationship* between commits X/Y/Z is recorded rather than lost.
23. As a **wiki maintainer**, I want the single wiki to hold both product and engineering streams, so that cross-stream links (an engineering concept touching a product feature) are first-class `[[wikilinks]]`.
24. As a **wiki maintainer**, I want streams distinguished by a `stream` frontmatter field (not by folder), so that the wiki stays flat and the agent can categorize/filter without physical partition.
25. As a **wiki maintainer**, I want the implicit graph of `[[wikilinks]]` navigable in a markdown graph viewer (e.g. Obsidian), so that I can see what connects to what without a dedicated index store.
26. As a **wiki maintainer**, I want `ws kb lint` to report orphan pages (no inbound `[[wikilinks]]`), so that I can find pages that have drifted out of the graph.
27. As a **wiki maintainer**, I want `ws kb lint` to report broken `[[wikilinks]]` (target slug resolves to no page), so that dangling links surface before they mislead a reader.
28. As a **wiki maintainer**, I want `ws kb lint` to be fast and deterministic, so that I can run it routinely without thinking about it.
29. As a **wiki maintainer**, I want `ws kb lint` to emit a parseable, machine-friendly report, so that the agent (or CI) can act on findings.
30. As a **workspace owner**, I want the SCHEMA and template documents embedded in the binary, so that `ws kb init` reproduces a canonical knowledge base with no external dependencies.
31. As a **workspace owner**, I want the scaffolded `SCHEMA.md` to document the hybrid provenance model and the pointer citation vocabulary, so that the agent has a precise contract to follow for engineering concepts.
32. As a **workspace owner**, I want the scaffolded `_template.md` to include the `stream` frontmatter field, so that every new page starts with the streams convention.
33. As a **teammate cloning the repo**, I want the knowledge-base wiki to be version-controlled (committed), so that citations resolve on my machine too.
34. As a **teammate cloning the repo**, I want `raw/` to remain gitignored (immutable blobs, local-only), so that large/sensitive sources don't pollute git history.
35. As a **teammate cloning the repo**, I want pointer-cluster source pages (in `wiki/sources/`) to be version-controlled text, so that external-link evidence is shared with the team and not stranded on one machine.
36. As an **agent**, I want a documented Query operation (read `index.md`, follow `[[wikilinks]]`, synthesize with citations, file valuable answers back as `synthesis/` pages), so that exploration compounds in the knowledge base.
37. As an **agent**, I want a documented Update operation (surface an unconfirmed fact, ask the human for a source, only integrate once a source exists), so that unconfirmed claims never enter the wiki as if sourced.
38. As an **agent**, I want a documented Lint operation (contradictions, stale claims, missing cross-references, concepts mentioned-but-pageless), so that the wiki stays healthy over time — noting that the *mechanical* subset of lint is owned by `ws kb lint` and the *judgment* subset is mine.
39. As an **agent**, I want source-page identifiers to be stable and semantic (e.g. `src-pricing-evidence`), so that pages don't fragment by ingest date.
40. As a **future maintainer**, I want the feature MD to record that `ws kb graph` and a real index store are deferred to post-MVP, so that the evolution path is intentional rather than forgotten.

## Implementation Decisions

### Architectural decisions (from the grilling session)

1. **Single knowledge base, both streams.** One wiki holds product and engineering knowledge. Streams are a frontmatter concern (`stream: product | engineering`), not a folder partition. The agent distinguishes streams at save/query time; cross-stream `[[wikilinks]]` form an implicit graph.
2. **CLI governs structural consistency; SCHEMA governs content + operations.** The CLI's job is mechanical (scaffold + lint). The agent's job, defined entirely in `SCHEMA.md`, is everything else (ingest, query, update, concept capture, cross-referencing, log/index maintenance).
3. **CLI surface = two commands only.** `ws kb init` and `ws kb lint`. There is deliberately **no** `ws kb ingest`, `ws kb query`, `ws kb update`, `ws kb list`, `ws kb status`, or `ws kb suggest`. Ingest is a SCHEMA-defined agent workflow performed by editing markdown; the yes/no gate invokes that workflow on consent.
4. **Hybrid provenance.** Two source types: blob (materialized in `raw/`, gets a source page) and pointer (an address to something living elsewhere, cited inline as a typed token, no source page by default).
5. **Pointer vocabulary (MVP).** `commit:<sha>`, `file:<path>#Lstart-Lend`, `issue:<id>`, `pr:<num>`, `web:<url>`, and `conversation:<date>` (the last appears **only** on conversation-snapshot source pages as a provenance marker, never as a bare inline token on a topic page).
6. **Typed-token stored form vs. clickable render form.** The stored citation is the parseable typed token `[commit:abc123]`. In prose the agent renders it as a clickable markdown link, e.g. `` [`abc123`](https://github.com/org/repo/commit/abc123) ``. `ws kb lint` does not parse or validate typed tokens in MVP.
7. **Opt-in concept-keyed source pages for pointer clusters.** When a bundle of pointers needs narrative explanation, the agent writes `wiki/sources/src-<concept>-evidence.md` — version-controlled, stable semantic id (NOT date-keyed), tagged `external` where the page is a cluster of external pointers. Topic pages cite it as `[[src-<concept>-evidence]]` when the whole bundle is the evidence, or cite individual pointers inline when one backs a specific claim. There is **no** `raw/sources/external/` folder (that was rejected: it collides with `raw/`'s gitignore, loses claim-level provenance, and date-keys sources).
8. **Conversation-snapshot resolution.** Conversational evidence → snapshot blob in `raw/snapshots/<date>-<topic>.md` (verbatim definition + paraphrased narrative, consent-gated) → source page `wiki/sources/src-conv-<date>-<topic>.md` → cited as `[[src-conv-...]]`. The bare `conversation:<date>` pointer lives only on that source page.
9. **Consent gate scope.** The yes/no gate fires on (a) new-page creation and (b) contradictions to existing claims (the existing SCHEMA's Contradictions-section flow). Routine refinement edits to existing pages are ungated and logged in `log.md`.
10. **Concept detection is agent-judged, not CLI-driven.** No `ws kb suggest` command. The harness, following SCHEMA instructions, notices a concept mentioned-without-a-page (or referenced-in-work-but-absent), gathers a definition, lists sources, and asks the human. The quality of this behavior is a function of how well `SCHEMA.md` is written — a doc-quality dependency, not a code dependency.
11. **`ws kb init` is Tier-3 scaffolding.** Per the restructure ADR, knowledge-base seeds are user-owned after first write. Re-init is skip-by-default; `--reset <asset>` refreshes a single embedded asset explicitly.
12. **Idempotency semantics.** On a workspace where `catalog/knowledge/wiki/index.md` already exists, `ws kb init` skips existing files, creates missing directories and missing seed files only. `ws kb init --reset <asset>` rewrites the named embedded asset from the binary (e.g. `--reset SCHEMA.md`).
13. **Graph is implicit for MVP.** The graph is the emergent `[[wikilink]]` structure; no index store, no `ws kb graph` command in MVP. `ws kb graph` is the documented first evolution when `index.md`-linear reads stop scaling (a few hundred pages). `ws kb lint`'s orphan detection is the MVP's only graph-traversal code and seeds that future command.
14. **`ws kb lint` MVP scope = orphans + broken `[[wikilinks]]` only.** No frontmatter validation, no pointer-staleness revalidation, no contradiction detection. These evolve later. Pointer staleness (a `file:` path that drifted) has no mechanical backstop in MVP — recorded as a known limitation.

### Modules / interfaces

- **New `ws-kb` crate** (module-level seam) exposing two pure operations:
  - `scaffold(root: &Path, reset: Option<AssetId>) -> Result<ScaffoldReport>` — writes the embedded KB tree, skip-by-default, optional single-asset refresh. Returns which assets were written/skipped/refreshed.
  - `lint(root: &Path) -> Result<Vec<LintFinding>>` — scans `wiki/**/*.md`, parses `[[wikilinks]]`, returns orphans + broken-link findings.
- **`ws-cli`** wires `ws-kb` under a `ws kb` subcommand (`ws kb init [--reset <asset>]`, `ws kb lint`). The CLI is a thin wrapper over the two library functions.
- **Embedded assets** live under the restructure ADR's canonical location (`crates/ws-cli/assets/catalog-knowledge/`) and are embedded via the restructure ADR's chosen mechanism (the `include_dir` crate, pending confirmation in the parent ADR's Q3). The embedded tree is the existing on-disk tree:
  ```
  catalog/knowledge/
  ├── SCHEMA.md          (extended: streams section + hybrid provenance + pointer vocabulary)
  ├── README.md
  ├── raw/  (README.md, .gitignore, assets/.gitkeep)
  └── wiki/ (index.md, log.md, _template.md, topics/, entities/, sources/, synthesis/)  (dirs .gitkeep'd)
  ```

### Schema changes (to the scaffolded `SCHEMA.md` and `_template.md`)

- Add a **`stream` frontmatter field** (`product | engineering`) to `_template.md` and document it in SCHEMA. No new folders.
- Add a **"Both streams live here"** section to SCHEMA documenting that product and engineering knowledge coexist, distinguished by `stream`, cross-linked via `[[wikilinks]]`.
- Add a **"Source types"** section to SCHEMA enumerating blob vs. pointer, the pointer vocabulary, the verbatim-definition/paraphrased-narrative rule, and the consent gate for conversational evidence.
- Add a **"Concept capture"** operation to SCHEMA describing the agent-judged gap detection, draft-definition + sources list, optional human enrichment, and the yes/no gate invoking Ingest on consent.
- Preserve the existing Ingest/Query/Update/Lint operations, noting that Ingest/Query/Update are agent-performed markdown edits and that the mechanical subset of Lint is delegated to `ws kb lint`.

### API contracts

- `ws kb init` exit 0 with a human-readable scaffold report (written/skipped/refreshed per asset); exit non-zero on filesystem error.
- `ws kb init --reset <asset>`: unknown asset name → error listing valid asset names; known asset → refresh + report.
- `ws kb lint`: exit 0 if no findings, exit non-zero (or 0 with warnings — to be confirmed at implementation) if findings exist. Output is a parseable report (findings with page path + finding type + severity + message), one per line, suitable for agent/CI consumption.
- `[[wikilink]]` resolution rule: a link target resolves if the slug (filename without `.md`) exists anywhere under `wiki/` ignoring the category folder — i.e. `[[revenue]]` resolves to `wiki/topics/revenue.md` or `wiki/entities/revenue.md`. Broken = no page with that slug exists. Orphan = a wiki page whose slug is never the target of any `[[wikilink]]` across the wiki.

### Specific interactions

- `ws init` (the umbrella command from the restructure ADR) invokes `ws kb init` as one of its module steps, so a full init produces a complete KB tree.
- `ws kb init` can be run standalone to set up or repair the KB without a full `ws init`.
- The harness's existing `AGENTS.md` injection (which tells the agent to "read `catalog/knowledge/SCHEMA.md`") now resolves to a real file in every fresh workspace, closing the latent bug noted in the restructure ADR §2.3.

## Testing Decisions

### What makes a good test here

Only test **external behavior at the library-function seam**, not implementation details. The two
operations are pure functions over a filesystem root, so fixtures in a `tempfile::TempDir` are the
natural highest seam. The CLI wrapper is too thin to warrant its own tests beyond a smoke test.
**One seam ideal:** the `ws-kb` library functions exercised via tempdir fixtures.

### Proposed seams (to be confirmed)

1. **`ws-kb::scaffold` over a TempDir** — assert the full tree is written; assert embedded file
   contents match the canonical assets byte-for-byte; assert re-running on a populated tree
   **skips** existing files (content unchanged); assert `--reset <asset>` rewrites exactly that
   asset and leaves others untouched; assert `--reset <unknown-asset>` errors with the list of
   valid asset names.
2. **`ws-kb::lint` over TempDir fixture wikis** — assert orphans reported (a page with no inbound
   links); assert broken links reported (a `[[wikilink]]` to a nonexistent slug); assert clean wiki
   reports no findings; assert slug resolution is folder-agnostic (links resolve across
   `topics/`/`entities/`/`sources/`/`synthesis/`).

### Which modules will be tested

- `ws-kb` (new crate): `scaffold` and `lint` as above. This crate establishes the first real test
  pattern in the workspace.
- `ws-cli`: smoke test only (the `ws kb` subcommand dispatches to the library without extra logic).

### Prior art

There is essentially **no existing Rust test prior art** in this workspace — only one
`#[cfg(test)]` module exists (in the GitHub-gh provider crate), and there are no `tests/`
directories. `handle_init` is currently untested. This feature therefore establishes the test
pattern (tempdir-based unit tests at the library seam). The nearest conceptual precedent is
`ws-catalog`'s `ensure_catalog_dirs` + `add_knowledge`/`get_knowledge` functions, which operate on
the filesystem root the same way `ws-kb` will — they are the idiomatic reference for "pure
functions over a workspace root" in this codebase, though they are themselves untested.

## Out of Scope

- **`ws kb graph` command** — deferred to post-MVP as the first scaling evolution, triggered when
  `index.md`-linear agent reads stop scaling (a few hundred pages). Reuses `lint`'s traversal.
- **Index store / hybrid search / embeddings / FTS5** — deferred indefinitely (the gist and its
  commenters reserve these for "past a few hundred pages"). MVP has no search index.
- **`ws kb ingest` / `query` / `update` / `list` / `status` / `suggest` commands** — explicitly
  rejected. These are SCHEMA-defined agent operations performed by editing markdown, not CLI
  commands.
- **Pointer-staleness revalidation in `ws kb lint`** — deferred. `file:` paths that drift and
  `commit:` SHAs that vanish are not mechanically checked in MVP. (Recorded as a known limitation.)
- **Frontmatter validation in `ws kb lint`** — deferred. The `stream` field and other frontmatter
  are not schema-validated mechanically in MVP.
- **Contradiction detection / stale-claim detection / missing-cross-reference detection** — the
  *judgment* subset of lint stays a SCHEMA-defined agent operation, not a CLI command, in MVP and
  likely permanently.
- **A `concepts/` folder or any folder partition for streams** — rejected. Streams live in
  frontmatter.
- **A dedicated external-sources folder under `raw/`** — rejected. External-link evidence is
  version-controlled text in `wiki/sources/`, not gitignored blobs.
- **Implementation of the feature** — this spec is specification only, per the user's instruction.
  Code is not written in this pass.

## Further Notes

### Known limitations accepted during design

1. **Pointer staleness has no mechanical backstop in MVP.** A `file:src/foo.rs#L40-58` source whose
   target moves or shifts rots silently until `ws kb lint` grows pointer-revalidation. This is an
   accepted MVP tradeoff for a deliberately minimal lint. Revisit when engineering concepts with
   file-range pointers become common.
2. **Agent query navigation is linear over `index.md`.** With no `ws kb graph` command, the agent
   reads `index.md` (which grows linearly with the wiki) and guesses which pages to drill into.
   Fine at tens of pages, painful at hundreds. The scaling cliff triggers `ws kb graph` (the
   documented first evolution).
3. **Content quality depends entirely on `SCHEMA.md` quality.** The CLI enforces structure only.
   The discipline of ingest, concept capture, cross-referencing, and filing-answers-back is governed
   by how well `SCHEMA.md` instructs the agent. This is the honest division: SCHEMA governs content
   quality; CLI governs structural consistency. A weak SCHEMA cannot be rescued by a stronger CLI.

### Dependencies on the parent restructure ADR

This feature assumes the restructure ADR's **Decision 1 (embed-and-scaffold)**, **Decision 2
(Tier-3 disposition for knowledge-base seeds)**, and **Decision 3 (`crates/ws-cli/assets/` as the
canonical asset location)**. The embedding mechanism (the ADR's open Q3, recommending `include_dir`)
is a parent dependency: until that lands, the embedded-asset path is provisional. The scaffolded
`SCHEMA.md` content additions (streams, source types, pointer vocabulary, concept-capture operation)
are part of this feature's deliverable and ship as embedded asset bytes.

### Relationship to the LLM Wiki pattern

This feature is a concretization of Karpathy's LLM Wiki pattern, with two extensions the pattern
leaves open: (1) a **merged product+engineering stream** in a single wiki (the pattern is
domain-agnostic; this feature commits to a two-stream single-wiki instantiation), and (2) a
**hybrid provenance model** adding pointer sources to the pattern's blob-only "raw sources" layer.
The `ws kb lint` command corresponds to the mechanical subset of the pattern's "Lint" operation;
the judgment subset stays with the agent per the pattern's "the LLM does the grunt work" principle.

### Evolution roadmap (recorded, not built)

1. `ws kb graph <subquery>` — reverse links, neighbors, shortest path, orphan listing. Reuses
   `lint`'s traversal. Triggered when `index.md`-linear reads stop scaling.
2. Pointer-staleness revalidation in `ws kb lint` — re-validate `commit:` SHAs exist, `file:`
   paths still resolve, `issue:`/`pr:` IDs are retrievable. Triggered when engineering
   pointer-sourced concepts become common.
3. Frontmatter schema validation in `ws kb lint` — enforce the `stream` field and reserved
   frontmatter. Triggered when frontmatter drift causes agent confusion.
4. Index store (FTS5/embeddings) — triggered only when the wiki reaches "past a few hundred pages"
  _scale, per the pattern and its community commenters.
