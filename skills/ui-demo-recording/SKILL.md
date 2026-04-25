---
name: ui-demo-recording
version: 1.0.0
format: skill/1.0
description: A protocol for recording polished UI demo videos using Playwright with injected cursors and subtitles.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(record|create).*(demo|walkthrough|tutorial).*(video)"

    confidence: 0.9

  - pattern: "show me how it works visually"

    confidence: 0.85

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: UI Demo Recording (Playwright)

This skill formalizes the process for creating professional, storytelling demo videos of web applications. It uses Playwright for deterministic recording and overlays for visual polish.

## The 3-Phase Process

### Phase 1: Discover (Introspection)

Before scripting, the Orchestrator MUST dump the interactive elements of the target page:

- **Identify**: Are fields `input`, `textarea`, or `contenteditable`?
- **Verify**: Are select options values or text-based?
- **Map**: Locate primary CTA buttons (e.g., "Submit Request").

### Phase 2: Rehearse (Verification)

Run the script without recording to verify every selector.

- **Rule**: If a selector fails in rehearsal, the recording MUST be aborted until fixed.
- **Diagnostic**: Dump all visible elements on failure to aid in selector discovery.

### Phase 3: Record (Storytelling)

- **Overlay**: Inject an SVG cursor overlay and a subtitle bar.
- **Movement**: Move the mouse to targets over 10 steps (no teleporting).
- **Typing**: Use `pressSequentially` with a 25-40ms delay per character.
- **Pacing**: Add 2-4s pauses after major actions (Login, Navigate, Submit).

## Output Standard

- **Resolution**: Fixed at 1280x720 (720p).
- **Format**: WebM.
- **Naming**: `docs/demos/YYYY-MM-DD-<feature>.webm`.

## Anti-Patterns

- **Teleporting Cursors**: Clicking elements instantly without showing the mouse path.
- **Blind Recording**: Attempting to record without the rehearsal phase.
- **Opaque Errors**: Swallowing selector failures in `try/catch` blocks.
