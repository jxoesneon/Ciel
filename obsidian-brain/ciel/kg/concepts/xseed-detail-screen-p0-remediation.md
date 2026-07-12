---
title: X-Seed Detail Screen — Design Council P0 Remediation Pattern
type: concept
tags: [concept, design-council, x-seed, flutter, remediation]
created: 2026-07-12
status: active
---

# X-Seed Detail Screen — Design Council P0 Remediation Pattern

## Definition

A checklist-like pattern for fixing high-severity P0 usability findings from a Design Council review of a Flutter screen. The goal is to wire hidden controls, remove misleading actions, add semantics, and explain disabled states.

## The Pattern

1. **Wire hidden controls.**
   - If a model/controller exists but is not rendered, connect it to the UI.
   - Convert the parent widget to a `ConsumerWidget`/`ConsumerStatefulWidget`, watch the provider, apply filters, and conditionally render when useful.

2. **Remove or fix misleading actions.**
   - If a bottom action operates on a single item in a list, remove it or make the selection explicit.
   - Prefer per-item actions in sheets/menus.

3. **Add semantic labels.**
   - Wrap chips and buttons in `Tooltip`.
   - Replace emoji-only indicators with text or icon + text.
   - Add `MaterialTapTargetSize.padded` to chip-like controls.

4. **Explain disabled states.**
   - Add tooltips or helper text for every disabled control stating the prerequisite.
   - Use a reusable `_DisabledTooltip` helper if the same pattern repeats.

5. **Localize new strings and regenerate l10n.**
   - Update `app_en.arb` / `app_es.arb`, then run `flutter gen-l10n`.

## Why It Matters

A single hidden feature can degrade Clarity, Efficiency, and Actionability simultaneously. Fixing it yields a multi-lens improvement and unblocks the Design Council gate.

## Related

- [[ciel/diary/2026-07-12-detail-screen-p0-implementation]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review]]
- [[ciel/projects/X-Seed/X-Seed]]
