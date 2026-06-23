# Workflow: Product Knowledge Base

How to grow, query, and maintain the LLM-maintained product wiki under
`catalog/knowledge/`. This implements the
[LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

> Read [`catalog/knowledge/SCHEMA.md`](../catalog/knowledge/SCHEMA.md) first — it
> is the authoritative contract for structure, conventions, and operations.

## Scope

This workflow covers the **company product side only**. `catalog/teams/` and
`catalog/services/` have their own purposes and are out of scope — do not
restructure them or mirror their content into this wiki.

## Layout

```
catalog/knowledge/
├── SCHEMA.md          ← the contract (structure, conventions, operations)
├── raw/               ← RAW SOURCES, gitignored, drop files here
│   └── assets/        ← downloaded images / attachments
└── wiki/
    ├── index.md       ← content catalog (read first)
    ├── log.md         ← chronological append-only log
    ├── topics/        ← concept pages
    ├── entities/      ← entity pages
    ├── sources/       ← one summary per ingested source
    └── synthesis/     ← cross-cutting analyses & comparisons
```

## Ingest a source

1. **Place the source.** Drop the file into `catalog/knowledge/raw/` (images in
   `raw/assets/`). For external Document sources, point to the configured feed —
   by default **Confluence** (see `config/providers/confluence.md` and the
   product's `knowledge_sources` in `catalog/products/<product>.yaml`).
2. **Read & discuss.** The agent reads the source fully, shares key takeaways,
   and confirms emphasis/open questions with the human.
3. **File a source page** in `wiki/sources/` (slug `src-<short-id>`).
4. **Update the wiki.** Create/update `topics/` and `entities/` pages touched by
   this source; flag contradictions in a `## Contradictions` section (never
   silently overwrite an existing claim).
5. **Update `wiki/index.md`** with every new/changed page.
6. **Append to `wiki/log.md`**: `## [YYYY-MM-DD] ingest | <Source Title>`.

Prefer one source at a time with the human involved. Batch-ingest is allowed
with lighter supervision — note it in the log.

## Query (brainstorm / answer)

1. **Start from the wiki.** Read `wiki/index.md` to locate relevant pages.
2. **Read** the relevant pages, following `[[wikilinks]]`.
3. **Synthesize** an answer with inline citations to source pages.
4. **File back** valuable analyses as `synthesis/` pages; link from `index.md`
   and log them so exploration compounds.

When developing a new idea, the wiki is the **primary context layer**: cite what
is known, surface gaps to investigate.

## Update (new facts surface)

1. If an unconfirmed fact or concept emerges during brainstorming, **pause and
   ask the human** to confirm it and provide/update a source (drop a file in
   `raw/` or point to a Confluence page).
2. Only after a source exists, ingest it and integrate the fact with a citation.
3. Never inject an unconfirmed fact into the wiki as if it were sourced.

## Lint (health-check)

Run periodically or on request. Report (and log) any:

- Contradictions between pages.
- Stale claims superseded by newer sources.
- Orphan pages with no inbound `[[wikilinks]]`.
- Important concepts mentioned but lacking their own page.
- Missing cross-references.
- Data gaps worth a web search or Confluence fetch.

Append to `wiki/log.md`: `## [YYYY-MM-DD] lint | summary`.
