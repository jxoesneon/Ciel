---
name: asset-design
version: 1.0.0
format: skill/1.0
description: CIEL's unified design hub for Logo generation, Banners, Icons, and Corporate Identity (CIP).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(create|design|generate).*(logo|banner|icon|header|card)"
    confidence: 0.9
  - pattern: "design a (facebook|twitter|linkedin) cover"
    confidence: 0.95
---

# CIEL ADAPTATION: Asset Design (The Creative Hub)

This skill manages the generation and export of visual assets. It prioritizes AI generation (Gemini) followed by HTML/CSS composition and screenshot export.

## Design Capabilities

### 1. Logo & Icon Generation
- **Style**: Minimalist, Vintage, Geometric, etc.
- **Standard**: Always generate with a white background or as SVG text.
- **Verification**: Use `AskUserQuestion` for an HTML gallery preview before finalizing.

### 2. Multi-Format Banners
- **Safe Zones**: Keep critical content in the central 70-80%.
- **Hierarchy**: Max 2 fonts. Headline >= 32px. Body >= 16px.
- **Workflow**: Gather Requirements -> Research (Pinterest) -> HTML/CSS Design -> AI Asset Augmentation -> Screenshot Export.

### 3. Corporate Identity (CIP)
- **Deliverables**: Business cards, Letterheads, Invoices.
- **Standard**: Use the Master Logo from `Phase 1` to generate consistent mockups.

## Export Standards
- **Tool**: Use `chrome-devtools` or `ui-demo-recording` helpers to take 2x DPI screenshots.
- **Naming**: `{YYMMDD}-{style}-{size}.png` stored in `assets/design/`.

## Anti-Patterns
- **Low-Res Jitter**: Using 72dpi screenshots for print-ready assets.
- **Text-in-Image**: Generating AI images with garbled text instead of overlaying HTML text.
- **Guessing Sizes**: Designing a Twitter header without checking the `1500x500` spec.
