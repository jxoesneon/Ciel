---
name: taste
version: 1.0.0
description: Anti-slop frontend engineering directive and design taste system. Enforces brief inference, dynamic design dials, absolute prohibition of AI visual tells, and 100% full-output completion.
domain: design
triggers:
  - "design taste"
  - "anti-slop"
  - "frontend taste"
  - "ui taste"
  - "redesign ui"
  - "clean frontend"
  - "design dials"
tags:
  - "frontend"
  - "design"
  - "ui"
  - "ux"
  - "anti-slop"
  - "accessibility"
runtime_compatibility:
  - "gemini_cli"
  - "antigravity"
  - "tauri"
  - "web"
source:
  origin: "https://github.com/Leonxlnx/taste-skill"
  author: "Leon Lin (@lexnlin)"
  license: "MIT"
---

# Taste: Anti-Slop Frontend Engineering Directive

A rigorous frontend taste engine and design governance protocol for interfaces, landing pages, and redesigns.
Every rule below is contextual. None of it fires blindly. First read the brief, set the dials, and execute without generic AI clichés.

---

## 0. BRIEF INFERENCE (Read the Room First)

Before generating code or selecting components, **infer what the user actually wants**. AI design output degrades when models immediately jump to a generic aesthetic rather than understanding context.

### 0.A Signals to Read
1. **Page Kind** - Marketing/landing, product screen, portfolio, editorial/docs, surgical redesign.
2. **Vibe Cues** - Minimalist, high-density, calm, editorial, stark/brutalist, luxury, devtool-clean.
3. **Target Audience** - Technical buyers, design-conscious consumers, enterprise evaluators.
4. **Existing Brand Assets** - Colors, logos, fonts, existing component libraries.
5. **Quiet Constraints** - Accessibility-first (WCAG AA), public sector, high trust. **These constraints override aesthetic preferences.**

### 0.B Mandatory 1-Line Design Read
Before any code generation, output a single line:
> **"Reading this as: `<page kind>` for `<audience>`, with a `<vibe>` language, leaning toward `<design system or aesthetic family>`."**

### 0.C Anti-Default Discipline
Actively resist standard LLM defaults:
- Do NOT use purple gradient meshes or neon cyan button glows.
- Do NOT default to three identical feature cards with thin Lucide icons.
- Do NOT use generic glassmorphism on every container.
- Do NOT default to Inter + Slate-900 or Fraunces editorial serifs without justification.

---

## 1. THE THREE DIALS (Dynamic Calibration)

After the design read, establish three calibrated dials:

* **`DESIGN_VARIANCE: 1..10`** - `1` = Strict symmetry & grid, `10` = Artsy asymmetry & editorial chaos.
* **`MOTION_INTENSITY: 1..10`** - `1` = Completely static, `10` = Cinematic physics & kinetic springs.
* **`VISUAL_DENSITY: 1..10`** - `1` = Spacious art gallery, `10` = Information-dense workbench.

### Dial Presets
| Use Case | VARIANCE | MOTION | DENSITY |
|---|---|---|---|
| Modern Web SaaS | 7 | 6 | 4 |
| Developer Tools / Documentation | 5 | 3 | 7 |
| Creative / Studio Portfolio | 9 | 8 | 3 |
| Editorial / Publication | 6 | 4 | 4 |
| Surgical Redesign (Preserve) | Match existing | Match + 1 | Match existing |
| Surgical Redesign (Overhaul) | +2 | +2 | Match existing |

---

## 2. ABSOLUTE NEGATIVE CONSTRAINTS (Banned AI Tells)

The following patterns are hard failures and must never appear in generated output:

### 2.1 Complete Em-Dash Ban
**The em-dash (`—`) and en-dash (`–`) are banned.**
- Do not use em-dashes in headlines, copy, button labels, quotes, or eyebrows.
- Use periods, commas, colons, or parentheses for sentence structure.
- Use simple hyphens (`-`) exclusively for compound words and numeric ranges (`2024-2026`).

### 2.2 Color Discipline
- **Single Accent Color:** Pick one accent color for the entire page. Saturation must be `< 80%`.
- **No Purple AI Glow:** Ban neon cyan/purple glows and automatic radial mesh gradients.
- **Gray Harmony:** Never mix warm and cool grays within the same composition. Pick Zinc, Slate, or Stone and lock it.

### 2.3 Typography & Copywriting
- **Serif Discipline:** Serif is banned as the default display font. Do NOT use `Fraunces` or `Instrument Serif` as automatic defaults. Use sans-serif displays (`Geist`, `Cabinet Grotesk`, `Satoshi`, `Outfit`) unless the brief explicitly demands a heritage or literary publication.
- **Descender Clearance:** When using italic display type, ensure descender clearance (`leading-[1.1]` minimum with reserve padding) so letters like `g`, `y`, `p`, `j` are never clipped.
- **No Fake Precision:** Never fabricate fake statistics (`92% faster`, `4.1x efficiency`, `48k users`) unless provided by real brief data or explicitly marked with `<!-- mock -->`.
- **No Pretentious Labels:** Ban phrases like "Quietly trusted by", "From the field", "Currently on the bench", or "Stage 1: Install". Use plain verbs: "Install", "Configure", "Ship".

### 2.4 Layout & Component Traps
- **No Div-Based Fake Previews:** Do not build fake terminal windows or fake dashboard widgets out of styled divs in the hero section. Use real screenshots, real interactive components, or clean typography.
- **No Floating Corner Paragraphs:** Do not place orphaned explainer text in the top-right corner of section headers.
- **Zero Decorative Status Dots:** Coloured status dots are permitted only for real semantic system state, never as list bullets or visual filler.

---

## 3. FULL OUTPUT ENFORCEMENT (Zero-Slop Directive)

A partial output is a broken output.
1. **Banned Shortcuts:** `// ...`, `// rest of code`, `// implement here`, `/* TODO */`, `// add remaining items`.
2. **Deliverable Completeness:** If 5 components or 3 files are requested, write all 5 components and all 3 files in full. Never leave implementations as an exercise for the user.
3. **Handling Token Boundaries:** If an output approaches context limits, complete the current section cleanly and conclude with:
   `[PAUSED — X of Y complete. Send "continue" to resume from: <section_name>]`

---

## 4. ACCESSIBILITY & PERFORMANCE GUARDRAILS

- **WCAG AA Compliance:** Ensure high contrast ratios across all text and icon glyphs.
- **Touch Target Minimums:** Interactive mobile targets must be at least `44x44px` with adequate touch spacing.
- **Motion Isolation:** All physics and continuous scroll listeners must reside in leaf client components using Motion's `useMotionValue` / `useTransform`. Never bind continuous user input to React `useState`.
- **Reduced Motion:** Always wrap kinetic spring animations and parallax effects with `prefers-reduced-motion` fallbacks.
- **Reduced Transparency:** Provide solid opaque fallbacks for glassmorphism panels when `prefers-reduced-transparency` is active.

---

## 5. SURGICAL REDESIGN PROTOCOL

When improving an existing codebase, follow the **Scan → Diagnose → Fix** loop:
1. **Scan:** Read existing styling mechanisms (Tailwind, CSS modules, design tokens).
2. **Diagnose:** Identify generic fonts, oversaturated accents, orphaned text lines, and AI fingerprints.
3. **Fix:** Apply targeted upgrades within the existing architecture. Do not rewrite working foundations from scratch. Use `text-wrap: balance` on headlines and tighten letter-spacing.
