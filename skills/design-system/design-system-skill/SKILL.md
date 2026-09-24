---
name: design-system-skill
version: 1.0.0
description: Use when building UI in a frontend codebase on top of an existing design system or component library. Discovers tokens and components, reuses them correctly, and binds to semantic tokens instead of hardcoding. Works with ANY system (MUI, Chakra, shadcn, Radix, internal @org/ui, local components, or Figma).
triggers: [build UI, design system, component library, frontend, ui implementation, tailwind, shadcn, Figma reference]
tags: [frontend, design-system, ui, scope:local, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: MIT
source: { tier: 1, origin: acquisition }
dependencies: { skills: [filesystem/SKILL.md] }
---

# Building on a Design System (frontend code)

A way to build UI in a frontend codebase *on top of an existing design system*. You discover the system, record what you find in a per-project reference, reuse its components, and bind to its tokens — accreting knowledge as you go. **The skill holds the method; the per-project reference holds the system.**

## Rules

1. **Read-only by default.** Treat the codebase as read-only until the user points you at where to write.
2. **Confirm before writing.** Name what you're about to change and get a clear go-ahead. Don't infer permission from intent.
3. **Verify the system before building; never invent it.** Confirm the resolved design-system source with the user, and surface what you *can* and *can't* reproduce from it, **before** generating UI. If the system is ambiguous or incomplete, or a token / component / icon can't be sourced from it, **ask** — do not silently substitute an invented token, a look-alike component, or a hand-drawn icon and present it as on-system. Fabricated fidelity is worse than a flagged gap.

## How to operate

1. **Find the design system — and confirm the source before building.** The system's source may be **(a) the codebase** — dependencies (a UI package like MUI / Chakra / shadcn / Radix, or an internal `@org/ui`), a local components directory, and the token source (Tailwind config, CSS custom properties, a tokens file/DTCG, a theme object) — or **(b) a Figma reference** the user provides (a library/file URL), when the real system lives in Figma rather than in code (e.g. a native app with no web component library). Detect (a) by inspecting the codebase; take path (b) whenever a Figma reference is passed (see "Figma-sourced design systems" below). **If neither is present, surface what's installed or available and offer those as options, or ask which to adopt — don't invent one.** Then **confirm the resolved source — and what you can/can't reproduce from it — with the user before building** (Rule 3). Also confirm *what* you're building and *where* it's allowed to write.
2. **Discover foundations** by introspection (see the Foundations checklist below) — read the types, the theme/token source, and any Storybook/docs — and record them by copying `design-system-reference.template.md` into the project (or a section of `CLAUDE.md`).
3. **Discover components on use.** The first time you reach for a component, introspect its **props** — its TypeScript types, Storybook args, or source. Record the selection axes + key props + any non-introspectable gotcha.
4. **Build on the framework.** Reuse the app shell / layouts / page or route templates the system or app already defines, rather than recomposing chrome from primitives. If undocumented, inspect or ask where new UI belongs.
5. **Capture as you go.** Add confirmed patterns and gotchas to the reference so the next session doesn't re-learn them.

Don't front-load all discovery — probe what the current task needs.

## Figma-sourced design systems (when a Figma reference is provided)

When the system lives in Figma (common for native apps with no importable web component library), extract it from the Figma reference and render against the **real** extracted assets — never fabricate. Discover once; record in the reference; then confirm with the user (Rule 3) before building.

- **Tokens** — pull variables/styles (`get_variable_defs`, plus the Plugin API for unpublished or mode-scoped ones) → export as CSS custom properties. Bind every layout value to these.
- **Type** — text styles → a type ramp (family, sizes, weights, line-heights). Use the real font family.
- **Icons & assets — use the REAL ones; never hand-draw.** Find the icon set in the source (an icon library / component set) and **export the needed icons as SVG** from Figma (`download_assets`, or the asset download URLs returned by `get_design_context`); reference those files. Generating look-alike icons in code is a fabrication — if a needed icon can't be exported, flag it and ask, don't invent it.
- **Components** — capture each component's **appearance** faithfully from the source (`get_design_context` / screenshots of the real component and its states): control heights, radii, padding, chrome, selected/hover/disabled states. Record a per-component appearance entry.
  - **Honest limitation:** a Figma/native system's real components usually **cannot be imported into code**. You approximate their appearance from the extracted reference — and that approximation must be **labelled as an approximation**, never presented as the real component. If pixel-true component fidelity is required, say so and route the work to a Figma render instead of claiming the HTML uses real components.

## Working principles

- **Reuse before creating.** Look for an existing component in the library before building one. The library is large — assume it exists.
- **Configure, don't fork.** Use a component's real **props/variants**. Copy-pasting a library component and editing its internals severs it from the system and drifts. An override you'd repeat 3+ times = a missing variant — flag it, don't keep re-applying it.
- **Bind to *semantic* tokens; never hardcode.** Semantic/alias token first → component token if one fits → primitive only as a last resort (and flag it — it usually means a missing semantic token). Hardcoded hex/px *and* primitive bindings both break theming.
- **Introspect before using an unfamiliar component.** Its props handle the whole variant space — don't reach for a different component when a prop would do.
- **Cite sources; distinguish observation from rule.** "Seen in [path]" ≠ "the system always does X." Confirm before codifying a pattern.

## Foundations: what to discover

Probe each; record only what exists.

| Foundation | Where it lives in code |
| --- | --- |
| Color, spacing, sizing, radius, border, opacity, z-index — any single value + theming | CSS custom properties / Tailwind theme / tokens file (DTCG) / theme object |
| Typography (family / size / weight / line-height) | Type-scale tokens / text components / typography utilities |
| Elevation / shadow | Shadow / elevation tokens or utilities |
| Layout grids / breakpoints | Breakpoint tokens / layout & grid primitives |
| Components | The component library (installed package or local) |

*Universal core* (almost always present): color, typography, spacing, sizing, corner radius, border/stroke, elevation/shadow, iconography, focus & state. Confirm which exist; don't assume.

## Token tiers & theming

Three tiers; theming lives in the middle one:

1. **Primitive / global** — raw values (palette steps, base scale). Usually *not* consumed directly.
2. **Semantic / alias** — roles (`bg-accent`, `text-primary`, `space-md`). **Consume these.** Theme switches (light / dark / brand / density) repoint them, so semantic-bound UI themes automatically.
3. **Component** — per-component values; present in some systems, not all.

Theming is applied via a code switch: a `[data-theme]` attribute, a `dark:`-style utility variant, or a `ThemeProvider`/context at the root. Systems vary — map what you find onto this model.

## Components

Two layers to every component:

- **Selection axes** — which component to import and which discrete variants exist (size, role, emphasis, icon-only…). You pick by importing the right one.
- **API** — its **props** (variant / boolean / value / slot). Read them from the component's TypeScript types, Storybook, or source. The same prop model applies across a family.

Introspect the props before using a component, and match each value to the prop's **type** — don't guess prop names or values.

## Framework & surfaces

Above components, a system or app usually defines higher-order patterns — an app shell, layouts, and **the surfaces where new UI belongs** (a content slot, a route/page template, a modal). Reuse these as the foundation for feature work rather than recomposing chrome from primitives. Many libraries publish authored guidance — read the README / docs / Storybook before building.

## Code mechanics & gotchas

- **Verify by building, not by reading.** A change isn't done until it typechecks/builds and renders. Run the typechecker and the build first — the compiler catches misused props and bad imports instantly, so use that hard signal. Then render it: run the app or Storybook and look. If no browser/preview is available, fall back to a dev-server smoke test (confirm it serves and the modules transform without error) and say so.
- **Import from the system, not hand-rolled.** Use the library's component (`import { Button } from '<pkg>'`) via its real export path, not a raw `<button>` or a re-implementation. Check the package's exports before assuming a path.
- **Reference tokens, not literals.** Apply the semantic token (CSS var / theme key / utility class), never a raw hex/px. Hardcoded literals don't theme.
- **Respect the framework's component model.** Honor client/server boundaries (e.g. React Server Components), required providers/context (a `ThemeProvider` the components expect — used outside it, they render unstyled or throw), and the styling mechanism (CSS-in-JS / utility classes / CSS modules).
- **Pin to the installed version.** The package **version** (package.json / lockfile) is the source of truth — verify the prop API against the *installed* version, not docs for a different one. Watch peer-dependency mismatches.
- **Don't restyle past the API.** Configure via props/variants and the theme. Forcing appearance with `!important` or deep selectors into a component's internals is the code equivalent of detaching — it drifts and breaks on upgrade.

## The per-project reference

Copy `design-system-reference.template.md` into the project (or keep it as a section of `CLAUDE.md`) and keep it live — discovered foundations, component entries, framework surfaces, confirmed patterns, and gotchas accumulate there. Keep it lean: capture what introspection *can't* give back (intent, gotchas, decision criteria) and let the code itself hold live values.

**Stamp cached components for staleness.** Record each cached component family with the package **version** it was read from. Next session, if the installed version changed, re-introspect those; otherwise trust the cache. (The compiler is a backstop: a renamed or removed token/prop fails the build rather than passing silently, so a stale entry surfaces on its own.)
