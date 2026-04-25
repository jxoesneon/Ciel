---
name: document-and-video-intelligence
version: 1.0.0
format: skill/1.0
description: Document conversion/OCR via Nutrient and technical video explainers via Manim.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(process|ocr|convert|animate).*(pdf|docx|nutrient|manim|explainer)"
    confidence: 0.9
  - pattern: "technical explainer"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Doc & Video Intelligence (Nutrient & Manim)

This skill formalizes document processing and technical visualization. it prioritizes data integrity and visual clarity.

## Document Processing (Nutrient)
1. **Conversion**: Use for high-fidelity PDF/DOCX/HTML conversion. Mandate `output: { "type": "docx" }` for tabular recovery.
2. **OCR**: Use on all scanned PDFs. specify `language: "eng"` or `fra` to improve extraction accuracy.
3. **Redaction (Hard Gate)**: Mandate pattern-based redaction (`social-security-number`, `email-address`) before any PII-bearing doc is shared or archived.

## Technical Explainers (Manim)
- **Visual Thesis**: Define one sentence that the animation MUST prove.
- **Scene Discipline**: 3-6 scenes max. Use "Progressive Reveal" to prevent screen clutter.
- **Network Graphs**: Distinguish low-signal clutter from high-signal "Bridge" nodes using color coding.
- **Render Loop**: Smoke test at `ql` (low quality) first. Render final at `qh` (high quality) ONLY after timing approval.

## Integrated Workflow
1. Extract table from PDF (Nutrient).
2. Transform table into metric progression (Python).
3. Animate metric progression (Manim).

## Anti-Patterns
- **Silent Redaction Failure**: Assuming a doc is safe because it was "converted" without an explicit redaction step.
- **The 10-Minute Render**: Running high-quality Manim renders on untested code.
- **Visual Noise**: Adding motion to Manim objects just to "keep the screen busy."
