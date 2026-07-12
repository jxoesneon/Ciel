---
title: kepano/obsidian-skills
type: concept
aliases: [obsidian skills]
tags: [concept, obsidian, ciel, skills]
created: 2026-07-08
status: active
---

# kepano/obsidian-skills

A collection of agent skills for Obsidian that teach AI agents to use Obsidian CLI and open formats: Markdown, Bases, JSON Canvas, and Defuddle.

Source: https://github.com/kepano/obsidian-skills

## Skills

- **obsidian-markdown** — Obsidian Flavored Markdown (wikilinks, embeds, callouts, properties, tags, comments, highlights, math, Mermaid, footnotes).
- **obsidian-cli** — Interact with a running Obsidian instance via the `obsidian` CLI (read, create, search, manage notes, plugin/theme dev commands).
- **obsidian-bases** — Create and edit `.base` files with database-like views, filters, formulas, and summaries.
- **json-canvas** — Create and edit `.canvas` files with nodes, edges, groups, and connections per the JSON Canvas Spec 1.0.
- **defuddle** — Extract clean markdown from web pages, reducing token usage by removing navigation and ads.

## Local installation

Cloned to `.ciel/obsidian-skills/` for reference.

`defuddle` is installed in `scripts/obsidian/` for web-to-markdown extraction.

## Integration with Ciel

- Use `obsidian-markdown` conventions when writing notes in the vault.
- Prefer `defuddle` over raw WebFetch when saving web pages to `raw/`.
- Use `obsidian-cli` commands when the user asks to interact with the live Obsidian app.
- Use `json-canvas` when creating or editing visual canvases in the vault.
- Use `obsidian-bases` when the user wants database-like views over notes.

## Related

- [[_CLAUDE.md]]
- [[ciel/kg/concepts/council-subagent-invocation]]
- [[index]]
