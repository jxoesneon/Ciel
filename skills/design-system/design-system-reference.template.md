# Design System Reference — [System / Product name]

The living reference for building **[product]** on this design system in code. The skill holds the *method*; this file holds what's been discovered about *this* system. Keep it lean — capture intent, gotchas, and decision criteria; let the code hold live values. Update as you learn. (Can live here or as a section of `CLAUDE.md`.)

## Sources & access

- **Design system source:** [package name + version  ·  or  ·  local components path]
- **Token source:** [Tailwind config / CSS vars / tokens file (DTCG) / theme object]
- **Theming:** [how themes switch — `[data-theme]` / `dark:` variant / ThemeProvider]
- **Docs / references:** [Storybook, docs site, examples — link + 1-line each]
- **Writable area:** [the dir(s)/files safe to modify]
- *(All writes still require user confirmation per SKILL.md.)*

## Foundations

> Fill in only what exists. Note the tier (primitive / semantic / component) and reach-for-first names.

### Color

- Semantic roles to reach for first: [e.g. `bg-surface`, `text-primary`, `border-default`, `accent`]
- Raw palette (avoid for new work): [e.g. `blue-500`]
- Notes / gotchas: [...]

### Spacing · sizing · radius · border

- Scale: [...]   Radius: [...]   Control heights / icon sizes: [...]
- Naming quirks: [...]

### Typography

- Type scale / tokens, or text components: [...]
- Families / weights: [...]
- How to apply: [text component / token / utility class]

### Elevation / shadow

- [names + when used]

### Breakpoints / layout

- [breakpoints; layout & grid primitives]

### Icons

- Source: [package] · naming: [convention] · sizes: [...]
- How they slot into components: [...]

### Token binding pattern

[the working snippet for this system — import path + how tokens are referenced (utility class / CSS var / theme key)]

## Components

> One entry per **family**. Selection axes + key props + non-introspectable notes only. Introspect for live prop details.

### [Family name]

- **Selection axes:** [which component to import + the discrete variants]
- **Props (API):** [key props + 1-line on what each does]
- **Import · version:** [`import { X } from '<pkg>'` · package version] (re-introspect if the version changed)
- **Notes:** [intent, when to use which variant, gotchas]

## Framework & surfaces

- **App shell / layout:** [component or pattern + how to use it]
- **Where new features go:** [the surface(s) — content slot, route/page template, modal — and how]
- **Recipes:** [e.g. "new page → AppLayout + PageHeader + content slot"]

## Confirmed patterns

- [rules verified in this codebase — these override generic instincts]

## Gotchas

- [things that look like bugs but aren't, or that cost time once — e.g. required providers, RSC boundaries, peer-dep quirks]
