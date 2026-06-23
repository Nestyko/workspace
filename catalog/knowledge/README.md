# Product Knowledge Base

An **LLM-maintained wiki** for the company product side, following the
[LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

This directory is the compiled knowledge layer that sits between you (the human,
curating sources and asking questions) and the raw upstream material. The AI
agent **owns and maintains** the wiki; you own sourcing, direction, and good
questions.

## Layout

```
catalog/knowledge/
├── README.md        ← you are here
├── SCHEMA.md        ← the contract: structure, conventions, ingest/query/lint workflows
├── raw/             ← RAW SOURCES — drop source files here (gitignored, local-only)
│   └── assets/      ← downloaded images / attachments
└── wiki/            ← THE WIKI — agent-generated markdown, version-controlled
    ├── index.md     ← content catalog (read this first)
    ├── log.md       ← chronological operation log
    ├── topics/      ← concept pages
    ├── entities/    ← entity pages
    ├── sources/     ← one summary page per ingested source
    └── synthesis/   ← cross-cutting analyses & comparisons
```

## How to use it

### Add a source
1. Drop the file (clipped article, PDF, export, transcript) into `raw/`.
   Put any referenced images in `raw/assets/`.
2. Tell the agent: *"ingest `<filename>`"*.
3. The agent reads it, discusses takeaways, writes a source summary, updates
   relevant topic/entity pages, updates `index.md`, and logs it.

### Ask a question
Just ask. The agent reads `index.md`, follows links into the wiki, and answers
with citations. Valuable analyses get filed back as `synthesis/` pages so your
exploration compounds.

### Brainstorm an idea
Start from the wiki. The agent uses it as the primary context layer — citing
what is known and surfacing gaps to investigate. When a new, unconfirmed fact
surfaces, the agent will **ask you to confirm it and provide a source** before
integrating it into the wiki.

### External document sources
`raw/` is the default. The wiki can also be fed from configured Document sources
— by default **Confluence** (see `config/providers/confluence.md` and each
product's `knowledge_sources` in `catalog/products/<product>.yaml`).

## Scope

This knowledge base covers the **company product side only**. The
`catalog/teams/` and `catalog/services/` directories are governed by their own
schemas and are **not** part of this wiki — do not restructure them or mirror
their content here.

## The contract

Everything above is governed by [`SCHEMA.md`](SCHEMA.md). Read it before
maintaining the wiki.
