# DOCKET 2026-10-08 — Industrial Dashboard Beauty Redesign

**Case**: `/Users/mey/martensite/.ciel/council/CASE_DASHBOARD_BEAUTY.md`
**Artifact**: Converged spec for `examples/industrial_dashboard` visual redesign (all elements preserved)
**Method**: Design Council (sequential) → Chairman synthesis v1 → General Council (sequential) → final synthesis v2. Stage-2 anonymized cross-review per ANONYMIZATION.md.

## Vote record

| Design Council | S1 | S2 | | General Council | S1 | S2 |
|---|---|---|---|---|---|---|
| Clarity | 6 | 7 | | Coherence | 7 | 7 |
| Inclusion | 6 | 5 | | Capability | 8 | 8 |
| Efficiency | 7 | 7 | | Safety | 6 | 7 |
| Aesthetics | 8 | 8 | | Efficiency | 7 | 7 |
| Actionability | 7 | 7 | | Evolution | 7 | 8 |

- Design Council: **PASS** — 4/5 ≥ 6, Inclusion > 3, no veto.
- General Council: **PASS** — weighted 0.20·7 + 0.20·8 + 0.25·7 + 0.15·7 + 0.20·8 = **7.40** ≥ 6.5; pivotal lens **Safety** (binding constraint: badge-tick independence, additive weight API, element-preservation gate).

## Key contested resolutions (Chairman rulings)

1. **REGISTRY grouping mechanism**: flatten into additional zone *pages* via `pages()` — NOT nested Tabs. Verified in source: `Tabs::event` returns Ignored for non-nav keys and never forwards ANY keys to panel children; TabPanels cannot hold FocusManager focus. Flattening is keyboard-switchable, auto-suspends ticking, deep-linkable today. Page count capped ~7.
2. **Annunciation is non-negotiable (C2-lite)**: any page/segment concealing live/alarm channels carries count+worst-severity badge — painted, in accessible name, AccessKit live-region on change. **Badge data source lives in the always-mounted chrome/alarm model, NOT the suspended page tick** (Safety's stale-badge catch — the worst-case regression this spec could ship). Default-selected page = one with unacked alarms. Per-Segment inner badges deferred to v1.5.
3. **Weight plumbing scope — NOW-OR-NEVER** (Evolution won over Efficiency's deferral): the dashboard paints every panel through its *forked* `TextPainter` → weight landing only in `shape_text`→`Text` widget leaves the whole dashboard weightless. Must thread through ambient `TextShaper::paint_shaped_text`/`paint_label` (~150 call sites) as **additive API only** + `TextShapeCache` key gains weight bits (else regular text hits semibold-shaped entries) + `GlyphRun` carries weight metadata (else `paint_audit` stays weight-blind — wcag.rs:76) + reconcile the forked TextPainter.
4. **Extraction — split by divergence cost**: NOW = forked TextPainter merge, ambient weight path, `TokenKey`/`ThemeDictionary` both-themes for inset/overlay/series (needs `SeriesN` keys or `ThemeToken::Colors` + `ThemeDiff` handling or theme switches pop), deterministic font fixture for goldens (system-font goldens poison baseline corpus). DEFERRED (tracked, framework-bound) = sweep→PNG into shared `martensite-test` (correct attribution — `martensite-render-test` is test-only), shared chrome widget, per-widget palette supersession (pie_chart PALETTE et al.).
5. **"Hero" renamed → `primary`** — collides with the removed HeroHeader precedent in DASHBOARD_LAYOUT_GRAMMAR.md. Rank = primary-task surface (monitoring counts — NOT verb-restricted). Differentiation via type tier + level badge + tonal shift. **No accent keyline** (collides with accent focus ring). Title names the task, never restates the zone tab.
6. **Chrome-mount idiom resolved**: `panel_chrome`/`panel_border` exist only on dock-level panels. In-page surfaces get: `quiet` = `group_label` + bare band + well/fill token (no chrome band); `standard`/`primary` chrome tiers scoped to dock panels + a defined in-page mount idiom to be codified in the grammar doc.
7. **Type-scale token home**: new semantic `FontSize*` TokenKey variants (non_exhaustive, additive) for both themes + published role→tier mapping in the grammar doc. Legacy Small/Medium/Large trio untouched.
8. **New widget APIs acknowledged** (additive-only): `Button` filled/primary variant (currently hardcoded FACE/EDGE); severity color setter on `Badge` (currently hardcoded ERROR red); badge slot on `Tabs::tab`/Segmented option labels.
9. **"Consolidate" clarified**: no selector surface is deleted — the ~6 `selected_asset` writers distribute across pages; V7 inventory gate enforces preservation.
10. **PNG encoding**: pin `png` 0.18.1 exactly (already locked + cargo-vet exempt via vendored vello) — not tiny-skia's disabled `png-format` feature.
11. **V4 CVD**: ship the ~30-line LMS-matrix deutan/protan post-processor inside the dump-frames tool — a sighted "PNG eyeball" cannot verify CVD; an unimplemented gate contradicts the standards-cited lint convention.
12. **MVP scope**: Phase A + B1 + B2(ambient) + B6 + B7 + C1 + C2-lite + C5 ≈ 80% of visual payoff. B8 (per-zone tint) last + droppable. A2's scale-factor parity stays a gate item (scale mismatch halves reported font sizes — masks violations).
13. **Tabs::event correction**: mechanism is "returns Ignored for non-nav keys, never forwards keys to panels" — the D4 fix targets focus routing into TabPanel; filed as tracked framework follow-up.
14. **Deferred to follow-up tickets**: `Tabs::event`/`TabPanel` focus-routing defect (affects every nested-navigation app); `page_request` promoted as a general "navigation intent" pattern; sweep→PNG shared-crate extraction; shared chrome-tier widget; per-widget palette unification.

## Final converged spec — see SPEC_DASHBOARD_BEAUTY_V2.md in project .ciel/council/
