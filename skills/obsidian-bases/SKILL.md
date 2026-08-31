---
name: obsidian-bases
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Obsidian Bases (.base files). Build database-like views of notes with filters, formulas, summaries, and table/cards/list/map layouts.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:knowledge"]
side_effects: ["fs"]
triggers:
  - pattern: "(\\.base|bases).*(view|filter|formula|summary|table|cards)"
    confidence: 0.9
  - pattern: "obsidian bases"
    confidence: 1.0
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: ["obsidian"] }
---

# CIEL ADAPTATION: Obsidian Bases

Create and edit Obsidian Bases (`.base` files) — YAML-defined database views over vault notes. Scope notes via filters, compute values with formulas, aggregate with summaries, and render as table, cards, list, or map views.

## Workflow

1. **Create** a `.base` file with valid YAML
2. **Scope**: Add `filters` (global and/or per-view) to select notes by tag, folder, property, or date
3. **Formulas** (optional): Define computed properties in `formulas`
4. **Views**: Add one or more views with `order` listing displayed properties
5. **Validate**: Confirm valid YAML, all referenced properties/formulas exist, quoting is correct
6. **Test**: Open in Obsidian; a YAML error usually means a quoting issue

## Schema

- `filters` — global filter (string or `and`/`or`/`not` object); applies to ALL views
- `formulas` — `name: 'expression'` computed properties
- `properties` — `displayName` overrides for note/file/formula properties
- `summaries` — custom summary formulas: `name: 'values.mean().round(3)'`
- `views` — array of view configs (`type`, `name`, `limit`, `groupBy`, `filters`, `order`, `summaries`)

## Filter Syntax

- Single: `filters: 'status == "done"'`
- `and` / `or` / `not` — recursive objects of filter strings
- Operators: `==`, `!=`, `>`, `<`, `>=`, `<=`, `&&`, `||`, `!`
- File helpers: `file.hasTag("x")`, `file.hasLink("x")`, `file.inFolder("x")`

## Properties

- **Note**: frontmatter values (`author` or `note.author`)
- **File**: `file.name`, `file.path`, `file.mtime`, `file.tags`, `file.links`, `file.backlinks`, etc.
- **Formula**: `formula.my_formula`
- `this`: base file in main content, embedding file when embedded, active file in sidebar

## Formulas

- Arithmetic: `total: "price * quantity"`; Conditional: `status_icon: 'if(done, "✅", "⏳")'`
- Date math: `days_old: '(now() - file.ctime).days'` — subtracting dates returns a **Duration**, NOT a number; access `.days`/`.hours` first
- Guard nulls: `'if(due_date, (date(due_date) - today()).days, "")'`
- Key functions: `date()`, `now()`, `today()`, `if()`, `duration()`, `file()`, `link()`

## Views & Summaries

- Types: `table`, `cards`, `list`, `map` (map needs lat/lng + Maps plugin)
- `order` lists properties to display; `groupBy` groups by a property
- Default summaries: `Average`, `Sum`, `Min`, `Max`, `Median`, `Range`, `Stddev`, `Earliest`, `Latest`, `Checked`, `Unchecked`, `Empty`, `Filled`, `Unique`
- Embed in Markdown: `![[MyBase.base]]` or `![[MyBase.base#View Name]]`

## YAML Quoting

- Single-quote formulas containing double quotes: `'if(done, "Yes", "No")'`
- Double-quote simple strings with special chars: `"Status: Active"`
- Unquoted strings break on `:`, `{`, `}`, `[`, `]`, `,`, `&`, `*`, `#`, `?`, `|`, `-`, `<`, `>`, `=`, `!`, `%`, `@`, `` ` ``

## Anti-Patterns

- **Duration without field access**: `(now() - file.ctime).round(0)` errors — Duration is not a number; use `.days.round(0)`.
- **Unguarded date math**: `(date(due_date) - today()).days` crashes when `due_date` is empty; wrap in `if()`.
- **Undefined formula refs**: Listing `formula.total` in `order` without defining `total` in `formulas` fails silently.
- **Double quotes inside double quotes**: `"if(done, "Yes", "No")"` is invalid YAML; wrap in single quotes.
