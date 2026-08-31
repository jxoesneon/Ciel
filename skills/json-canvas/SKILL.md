---
name: json-canvas
version: 1.0.0
format: skill/1.0
description: CIEL's framework for JSON Canvas (.canvas) files. Build visual canvases, mind maps, and flowcharts with nodes, edges, and groups per the JSON Canvas Spec 1.0.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:knowledge"]
side_effects: ["fs"]
triggers:
  - pattern: "(\\.canvas|json canvas).*(node|edge|group|create|edit)"
    confidence: 0.9
  - pattern: "json canvas"
    confidence: 1.0
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: JSON Canvas

Create and edit JSON Canvas files (`.canvas`) following the [JSON Canvas Spec 1.0](https://jsoncanvas.org/spec/1.0/). Build visual canvases, mind maps, flowcharts, and project boards with typed nodes, connecting edges, and grouping containers.

## File Structure

- Two top-level arrays: `{"nodes": [], "edges": []}` (both optional)
- `nodes` — placed objects; array order = z-index (first = bottom, last = top)
- `edges` — connections referencing node IDs

## Workflows

- **Create canvas**: Base structure → generate unique 16-char hex IDs → add nodes (required: `id`, `type`, `x`, `y`, `width`, `height`) → add edges → validate
- **Add node**: Parse existing → generate non-colliding ID → position with 50-100px spacing → append → optionally add edges → validate
- **Connect nodes**: Identify source/target IDs → unique edge ID → set `fromNode`/`toNode` → optional `fromSide`/`toSide`/`label` → append → validate
- **Edit**: Parse JSON → locate by `id` → modify attributes → write back → re-validate

## Node Types

- **text**: required `text` (Markdown-supported plain text)
- **file**: required `file` (vault path); optional `subpath` (heading/block, starts with `#`)
- **link**: required `url` (external URL)
- **group**: optional `label`, `background`, `backgroundStyle` (`cover`/`ratio`/`repeat`); visually contains child nodes positioned inside its bounds

## Generic Node Attributes

- Required: `id` (16-char hex), `type`, `x`, `y`, `width`, `height`
- Optional: `color` — preset `"1"`-`"6"` (red/orange/yellow/green/cyan/purple) or hex (`"#FF0000"`)

## Edge Attributes

- Required: `id`, `fromNode`, `toNode`
- Optional: `fromSide`/`toSide` (`top`/`right`/`bottom`/`left`), `fromEnd`/`toEnd` (`none`/`arrow`; default `toEnd=arrow`), `color`, `label`

## Layout & IDs

- Coordinates can be negative; `x` increases right, `y` increases down; position is top-left corner
- Space nodes 50-100px apart; 20-50px padding inside groups; align to grid (multiples of 10-20)
- Suggested sizes: small text 200-300×80-150; medium 300-450×150-300; file/link preview 250-500×100-400
- IDs: 16-char lowercase hex (64-bit random), unique across BOTH nodes and edges

## Validation Checklist

1. All `id` values unique across nodes and edges
2. Every `fromNode`/`toNode` references an existing node ID
3. Required type-specific fields present (`text`/`file`/`url`)
4. `type` is `text`/`file`/`link`/`group`; sides are `top`/`right`/`bottom`/`left`; ends are `none`/`arrow`
5. Colors are `"1"`-`"6"` or valid hex; JSON is valid and parseable

## Anti-Patterns

- **Literal `\\n` in text**: Obsidian renders `\\n` as backslash-n, not a newline; use `\n` in JSON strings.
- **Dangling edge refs**: `fromNode`/`toNode` pointing at non-existent IDs renders broken edges; always validate after edits.
- **ID collisions**: Reusing an ID across nodes/edges silently overwrites or breaks references; generate fresh 16-char hex each time.
- **Overlapping nodes**: Placing nodes without 50-100px spacing produces an unreadable canvas; respect layout grid.
