# Product Knowledge Base — Schema & Operations Contract

> This file is the **schema** layer of an [LLM-maintained wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).
> It is the contract between the human and the agent: it defines how the wiki
> is structured, what the conventions are, and the workflows the agent must
> follow when *ingesting* sources, *answering* questions, or *linting* the wiki.
> Co-evolve this file with the product team as you learn what works.

This knowledge base covers the **company product side only**. It is compiled and
maintained by the AI agent. The `catalog/teams/` and `catalog/services/`
directories are governed by their own schemas and are **out of scope** here — do
not restructure them or duplicate their content into this wiki.

---

## The three layers

```
catalog/knowledge/
├── SCHEMA.md        ← THIS FILE (the contract). Human + agent co-evolve it.
├── raw/             ← RAW SOURCES (immutable, gitignored, local-only)
│   └── assets/      ← downloaded images / attachments
└── wiki/            ← THE WIKI (agent-generated, version-controlled markdown)
    ├── index.md     ← content catalog (read this first)
    ├── log.md       ← chronological append-only operation log
    ├── topics/      ← concept pages
    ├── entities/     ← entity pages (products, features, personas, customers)
    ├── sources/     ← one summary page per ingested source
    └── synthesis/   ← cross-cutting comparisons, analyses, decisions
```

### 1. Raw sources — `raw/`
- Immutable source of truth: clipped articles, exported PDFs, Confluence
  exports, meeting transcripts, interview notes, spec dumps.
- The agent **reads** from here but **never modifies** them.
- **Place source files here.** The human curates what goes in; the agent does
  not decide what gets sourced.
- This directory is **gitignored** — sources are large, often sensitive, and
  already exist elsewhere. They are not version-controlled by this repo.
- Use `raw/assets/` for downloaded images referenced by a clipped article.
  When ingesting an article that references images, read the text first, then
  view referenced images separately for additional context.

### 2. The wiki — `wiki/`
- A directory of LLM-generated markdown. The agent **owns this layer
  entirely**: creates pages, updates existing ones on new evidence, maintains
  cross-references, flags contradictions, and keeps everything consistent.
- Humans **read** the wiki; the agent **writes** all of it.
- Every fact in the wiki must trace back to a source in `raw/` or a configured
  Document source. Unconfirmed claims are marked `[unconfirmed]`.

### 3. The schema — `SCHEMA.md` (this file)
- What makes the agent a disciplined wiki maintainer rather than a chatbot.
- Update it whenever conventions evolve; the agent must follow it rigorously
  in every session.

---

## Page conventions

### File naming
- Lowercase-kebab-case, `.md` extension.
- One concept/entity/source per file. Names are stable identifiers — rename
  only via a `lint` pass and update all `[[wikilinks]]`.
- Place each page in its category folder (`topics/`, `entities/`,
  `sources/`, `synthesis/`).

### Frontmatter (YAML)
Every wiki page begins with YAML frontmatter:
```yaml
---
title: Human-Readable Title
type: topic | entity | source | synthesis
tags: [revenue, pricing]           # optional, lowercased
created: 2026-06-23               # ISO date of first creation
updated: 2026-06-23               # ISO date of last meaningful edit
sources: [src-notion-pricing-2026] # source page slugs that back this page
status: draft | active | stale | superseded
---
```

### Linking
- Use Obsidian-style `[[wikilinks]]` to slugs (filename without `.md`) for
  cross-references. Prefer links over restating content.
- Every entity/topic mentioned more than once across the wiki deserves its own
  page. Link the mentions instead of expanding inline.

### Citations & provenance
- Every non-trivial claim cites its source inline, e.g.
  `Conversion is ~3.2% [[src-conversion-audit-2026]]`.
- Source pages (in `sources/`) record the original location: file in `raw/`,
  Confluence space+page URL, or date of capture.

### Contradictions
- When a new source contradicts an existing claim, **do not silently
  overwrite**. Add a `## Contradictions` section to the relevant page noting
  both claims, their sources, and dates. Mark the older claim `status: stale`
  or `superseded` only once the conflict is resolved with the human.

### Synthesis & filing answers back
- **Good answers get filed back into the wiki as new pages.** A comparison you
  were asked for, an analysis, a discovered connection — these belong in
  `synthesis/` so exploration compounds in the knowledge base instead of dying
  in chat history.

---

## Indexing and logging

### `wiki/index.md` — content catalog
- The catalog of everything in the wiki, organized by category.
- Each entry: link, one-line summary, optional metadata (date, source count).
- The agent **updates it on every ingest**. When answering a query, read
  `index.md` first to find relevant pages, then drill in.
- Works well up to a few hundred pages; revisits search tooling only if scale
  demands it.

### `wiki/log.md` — chronological log
- Append-only record of what happened and when.
- Each entry starts with a parseable prefix:
  `## [YYYY-MM-DD] ingest | <Source Title>` (also `query`, `lint`, `update`).
- The log gives a timeline of the wiki's evolution and helps recall recent work.
  `grep "^## \[" wiki/log.md | tail -5` lists the last five operations.

---

## Operations

### Ingest — adding a source
Trigger: the human drops a file into `raw/` (or points to a Confluence page) and
asks to ingest it. Flow:
1. **Read** the source fully.
2. **Discuss** key takeaways with the human; agree on what to emphasize and any
   open questions.
3. **Write a source summary page** in `wiki/sources/` (slug `src-<short-id>`).
4. **Update** relevant `topics/` and `entities/` pages — create new ones for
   concepts mentioned more than once. A single source may touch 10–15 pages.
5. **Flag contradictions** with existing claims (see above).
6. **Update `wiki/index.md`** with every new/changed page.
7. **Append an entry to `wiki/log.md`**.

Prefer ingesting one source at a time and staying involved. Batch-ingest is
allowed for many sources with lighter supervision — note it in the log.

### Query — answering against the wiki
Trigger: the human asks a product/domain question. Flow:
1. **Read `wiki/index.md`** to locate relevant pages.
2. **Read** the relevant pages, following `[[wikilinks]]` as needed.
3. **Synthesize** an answer with inline citations to source pages.
4. **File the answer back** if it is a valuable comparison/analysis — write a
   `synthesis/` page, link it from `index.md`, log it.

When an idea/feature is being developed, the wiki is the **primary context
layer**: start every brainstorm from the wiki, cite what is known, and surface
gaps to investigate.

### Update — surfacing new facts
During brainstorming/research the agent or human may stumble on a new,
unconfirmed fact or concept not yet backed by a source. Flow:
1. **Pause and ask the user** to confirm the fact and provide/update a source
   (drop a file in `raw/` or point to a Confluence page).
2. Only after a source exists, **ingest** it (see Ingest) and integrate the fact
   into the wiki with a citation.
Never silently inject an unconfirmed fact into the wiki as if it were sourced.

### Lint — health-check
Run periodically (or when the human asks). Look for and report:
- Contradictions between pages.
- Stale claims superseded by newer sources.
- Orphan pages with no inbound `[[wikilinks]]`.
- Important concepts mentioned but lacking their own page.
- Missing cross-references.
- Data gaps worth a web search or a Confluence fetch.
Log the lint pass and its findings in `wiki/log.md`.

---

## Document sources (external)

Raw files in `raw/` are the default. The wiki can also be fed from configured
**Document sources**. By default we support **Confluence**.

- Confluence spaces and page-format conventions are defined in
  [`../../config/providers/confluence.md`](../../config/providers/confluence.md).
- Each product catalog (`catalog/products/<product>.yaml`) lists its
  `knowledge_sources` (e.g. Confluence space, Jira project). These are the
  authoritative upstream feeds for that product's wiki content.
- When ingesting from Confluence, capture the space, page title, URL, and fetch
  date into the source summary page in `wiki/sources/`.

To add a new Document source type, extend the provider config under
`config/providers/` and document its space/folder mappings there. This wiki
remains a single compiled layer fed by any number of such sources.
