---
name: obsidian-markdown
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Obsidian Flavored Markdown. Author and edit wikilinks, embeds, callouts, properties, tags, and math/diagram syntax that extends CommonMark and GFM.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:knowledge"]
side_effects: []
triggers:
  - pattern: "(obsidian|wikilink|callout|frontmatter|embed).*(markdown|note|syntax)"
    confidence: 0.9
  - pattern: "obsidian flavored markdown"
    confidence: 1.0
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Obsidian Flavored Markdown

Create and edit valid Obsidian Flavored Markdown, which extends CommonMark and GFM with wikilinks, embeds, callouts, properties, comments, and math/diagram syntax. Standard Markdown (headings, bold, lists, tables, code blocks) is assumed knowledge; this skill covers only Obsidian-specific extensions.

## Authoring Workflow

1. **Frontmatter**: Add properties (title, tags, aliases) at the top of the file
2. **Content**: Use standard Markdown for structure plus Obsidian-specific syntax below
3. **Links**: `[[wikilinks]]` for in-vault notes (auto-tracks renames); `[text](url)` for external URLs only
4. **Embeds**: `![[embed]]` for inline content from other notes, images, or PDFs
5. **Callouts**: `> [!type]` for highlighted information
6. **Verify**: Confirm the note renders correctly in Obsidian's reading view

## Internal Links (Wikilinks)

- `[[Note Name]]` — link to note
- `[[Note Name|Display Text]]` — custom display text
- `[[Note Name#Heading]]` — link to heading
- `[[Note Name#^block-id]]` — link to block
- `[[#Heading in same note]]` — same-note heading link
- Define block IDs by appending `^block-id` to a paragraph; for lists/quotes place the ID on a separate line after the block

## Embeds

- `![[Note Name]]` — embed full note
- `![[Note Name#Heading]]` — embed section
- `![[image.png]]` / `![[image.png|300]]` — image, optional width
- `![[document.pdf#page=3]]` — PDF page

## Callouts

- `> [!note]` — basic callout
- `> [!warning] Custom Title` — callout with custom title
- `> [!faq]- Collapsed by default` — foldable (`-` collapsed, `+` expanded)
- Types: `note`, `tip`, `warning`, `info`, `example`, `quote`, `bug`, `danger`, `success`, `failure`, `question`, `abstract`, `todo`

## Properties (Frontmatter)

- `tags` — searchable labels (also settable inline via `#tag` / `#nested/tag`)
- `aliases` — alternative note names for link suggestions
- `cssclasses` — CSS classes for styling
- Tag rules: letters, numbers (not first char), underscores, hyphens, forward slashes

## Other Syntax

- **Comments**: `%%hidden text%%` (inline) or block `%% ... %%`
- **Highlights**: `==highlighted text==`
- **Math**: inline `$e^{i\pi}+1=0$`; block `$$ ... $$`
- **Mermaid**: ```` ```mermaid ```` blocks; link nodes via `class NodeName internal-link;`
- **Footnotes**: `[^1]` with `[^1]: content`; inline `^[content]`

## Anti-Patterns

- **External links for vault notes**: Using `[text](path.md)` instead of `[[wikilinks]]`, losing Obsidian's rename tracking.
- **Unescaped block IDs**: Placing `^block-id` mid-list instead of on a separate trailing line, breaking block references.
- **Markdown links for embeds**: Writing `[text](image.png)` instead of `![[image.png]]`, which links instead of embedding.
- **Over-nesting callouts**: Deeply nested callouts that break reading-view rendering; keep callouts flat.
