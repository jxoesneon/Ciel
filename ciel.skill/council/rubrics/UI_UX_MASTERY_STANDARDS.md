# UI/UX MASTERY STANDARDS — Design Council Master Specifications (2026)

Synthesized by a joint session of the **Ciel System Council of Five** and the **Design Council of Five**. Revised with deep research (27 web searches across WCAG 2.2/3.0, Core Web Vitals field data, agentic UX pattern libraries, Tailwind v4 token architecture, shadcn/ui Base UI, mobile ergonomics research, undo/cancel pattern frameworks, performance perception science, spring-physics micro-interactions, OKLCH dark mode, bento grid intent hierarchy, liquid glass composite materials, AI slop definition/detection, distributional convergence, C2PA content provenance, EU AI Act Article 50, the struggle premium, dead internet theory, indie web counter-movement, and prompt engineering craft).

---

## The 18 UI/UX Engineering & Design Standards

### 1. Agentic UX — Transparency, Control & Recovery (2026 Pattern Library)

The agent interface is a **control surface**, not a conversation. Chat-first UX is an anti-pattern for agents — agents are asynchronous, long-running, and multi-step. Six patterns map to three lifecycle phases:

### Pre-Action (Establishing Intent)

- **Intent Preview / Plan Summary**: Before any significant action, the agent reflects the task back in its own words as an alignment checkpoint. Multi-step operations outline key phases. Miscommunication caught here costs nothing; caught after 12 automated actions costs trust.
- **Autonomy Dial**: Users set agent boundaries — what it can do alone, what needs approval, what is forbidden. Scope declaration names what the agent will and will not do.

### In-Action (Providing Context)

- **Explainable Rationale**: Show the "why" behind agent decisions while working — not every micro-decision, but at critical junctures (irreversible, high-stakes, ambiguous).
- **Confidence Signal**: Surface certainty levels using human-readable categories (not raw "87% confident"), so users calibrate oversight appropriately.
- **Live Tool-Call Trace**: Real-time progress indicators when external tools or APIs run. Taskboard pattern (goals, tasks, owners, status, SLA) replaces chat as primary workspace.

### Post-Action (Safety & Recovery)

- **Action Receipts**: Every agent action produces a receipt — what changed, where, with what permissions, a diff or confirmation. Not a vague summary.
- **Action Audit Trail**: Timestamped, structured log grouped by task, with reversibility status (green/amber/red), selective undo, and before/after diff views.
- **Escalation Pathway**: High-ambiguity moments route to human review with the conflicting context surfaced.

**Human-in-the-Loop Interception**: High-risk or destructive actions MUST require explicit user approval. EU AI Act mandates transparency for high-risk AI systems — this is compliance, not just UX.

### 2. Core Web Vitals (CWV) — Field Data Thresholds

CWV are **field data** measured at the 75th percentile of real Chrome users over a 28-day rolling window (CrUX). A perfect Lighthouse score means nothing if 25%+ of real visitors on mid-range phones get slow results.

- **LCP (Largest Contentful Paint)**: $\le 2.5\text{s}$ (good) / $2.5$–$4.0\text{s}$ (needs improvement) / $> 4.0\text{s}$ (poor).
- **INP (Interaction to Next Paint)**: $\le 200\text{ms}$ (good) / $200$–$500\text{ms}$ (needs improvement) / $> 500\text{ms}$ (poor). INP replaced FID in March 2024 — it measures ALL interactions (not just first), reporting the worst or ~98th percentile. ~40% of mobile origins still fail INP.
- **CLS (Cumulative Layout Shift)**: $\le 0.1$ (good) / $0.1$–$0.25$ (needs improvement) / $> 0.25$ (poor). Reserve aspect-ratio boxes for async elements; size images and ads before load.

**INP Optimization**: Break long tasks ($> 50\text{ms}$) using `scheduler.yield()` with `setTimeout` fallback. Defer non-visual work (analytics, telemetry) via `requestIdleCallback` or Web Workers. Audit third-party scripts with facades, dynamic imports, or Worker isolation.

### 3. WCAG 2.2 Level AA/AAA Accessibility (Inclusion Lens Veto)

WCAG 2.2 (published 5 October 2023) adds 9 success criteria with deliberate focus on cognitive disabilities and mobile/touch interactions. EU EN 301 549 incorporates 2.2 in 2026.

- **Focus Not Obscured (Minimum) (2.4.11, AA)**: Keyboard-focused components MUST NOT be entirely hidden by sticky headers, cookie banners, or floating action buttons.
- **Focus Not Obscured (Enhanced) (2.4.12, AAA)**: No part of the focused component is hidden by author-created content.
- **Focus Appearance (2.4.13, AAA)**: Focus indicator $\ge$ area of 2 CSS pixel thick perimeter; contrast ratio $\ge 3:1$ between focused and unfocused states.
- **Target Size (Minimum) (2.5.8, AA)**: Pointer input targets $\ge 24 \times 24$ CSS pixels, with spacing exceptions for adequately separated smaller targets.
- **Dragging Movements (2.5.7, AA)**: Any drag-and-drop functionality MUST also be achievable with single-pointer clicks (sortable lists need up/down buttons; sliders need text inputs; kanban boards need "move to" menus).
- **Consistent Help (3.2.6, AA)**: Help mechanisms (chat, contact, FAQ) appear in the same relative position across all pages — users with cognitive disabilities rely on spatial memory.
- **Redundant Entry (3.3.7, AA)**: Prohibit re-entering information already provided in the same process. Multi-step checkouts must auto-populate or offer selection of previously entered data.
- **Non-Cognitive Auth (3.3.10, AAA)**: Authentication must offer alternatives to complex memory or visual puzzles.

**WCAG 3.0 Forward Readiness**: WCAG 3.0 Working Draft (March 2026, Silver Task Force) shifts from true/false success criteria to outcome-based measurement addressing cognitive/learning disabilities, low vision, and emerging technologies (AR/VR/XR, voice assistants). Design systems SHOULD begin adopting outcome-based accessibility testing in parallel with WCAG 2.2 conformance.

### 4. Bento Grid — Intent-Based Spatial Hierarchy

67% of the top 100 SaaS products on ProductHunt use bento grids. 35% longer dwell times vs uniform grids. CSS Grid (not Flexbox) is the only proper foundation — it offers true two-dimensional control over rows and columns.

- **Intent-Based Tier Hierarchy**:
  - **Hero Tile (Tier 1)**: Occupies most real estate ($2 \times 2$ or $3 \times 2$ span); contains primary CTA or most critical data.
  - **Utility Tiles (Tier 2)**: Medium ($2 \times 1$ or $1 \times 2$); secondary metrics, navigation shortcuts.
  - **Micro-Data Tiles (Tier 3)**: Small ($1 \times 1$); status indicators, social links, minor meta-information.
- **Strict Compartmentalization**: Each content piece lives in its own clearly defined box. Enforced visual boundaries create cognitive chunking — users scan 10+ pieces of information instantly without overload. Tasks completed 23% faster vs linear layouts.
- **Uniform Spacing**: Gutters between boxes MUST be identical throughout — consistency creates rhythm and intentionality.
- **Responsive Breakpoints**: Desktop ($\ge 1024\text{px}$): 4–6 columns, full complexity. Tablet ($768$–$1023\text{px}$): 2–3 columns, reduced spans. Mobile ($< 768\text{px}$): 1–2 columns, linear stacking.
- **Gestalt Principles**: Law of proximity groups related metrics in adjacent tiles. Law of similarity applies consistent styling within data categories.

### 5. Liquid Glass — Three-Layer Composite Material

Liquid Glass $\ne$ Glassmorphism. Apple's WWDC 2025 reset refracts and responds to light; glassmorphism is just a frosted card with blur. Typing `backdrop-filter: blur(10px)` and calling it glassmorphism is a 2026 quality failure.

- **Three-Layer Composite**: (1) Bottom layer handles `backdrop-filter` in isolation (GPU compositor thread, `transform: translateZ(0)`, `will-change: transform`). (2) Content layer sits above, never filtered directly (prevents composite jitter and sub-pixel antialiasing damage). (3) Sheen pseudo-element with diagonal gradient (`mix-blend-mode: screen`) for gloss.
- **Thickness Cues**: Thin border edge breaking the boundary, controlled highlight band implying light direction, subtle inner stroke hinting at cross-section. Inconsistency between components makes the UI look like stitched screenshots.
- **Chromatic Aberration**: Ultra-thin, high-chroma linear gradients on the 1px border simulate spectral splitting — gives the material physical weight.
- **IOR (Index of Refraction)**: Standard glass $\sim 1.5$, water $\sim 1.33$, diamond $\sim 2.4$. Real refraction (Chromium only) via SVG `feTurbulence` + `feDisplacementMap` — background warps at edges, not just blurs.
- **Accessibility Constraint**: Text on glass depends entirely on what sits behind it — a card that passes WCAG contrast on one screen fails on the next. Use glass as accent surfaces, not full layouts. Always verify contrast against the worst-case background.
- **Safari 18+**: `backdrop-filter` shipped unprefixed September 2024 — older support constraints no longer apply.

### 6. Tailwind CSS v4 — Three-Layer Token Architecture

Tailwind v4 replaces `tailwind.config.js` with CSS-native `@theme` directives. Rust-based engine delivers 5x faster builds, 100x faster incremental. Design tokens are CSS custom properties available at runtime — no rebuild needed for theme switching.

- **Three-Layer Token Hierarchy**:
  - **Base Tokens (Primitives)**: Raw values with no semantic meaning — the palette. Use OKLCH for perceptual uniformity.
  - **Semantic Tokens (Purpose-Driven)**: What colors mean — `background`, `surface`, `text`, `error`, `border`. These shift between light/dark/high-contrast modes.
  - **Component Tokens (Variants)**: How colors apply to specific UI elements — `button-primary-bg`, `card-surface`, `input-border`.
- **`@theme` Directive**: Defining `--color-primary-500` automatically generates `bg-primary-500`, `text-primary-500`, `border-primary-500`, etc. CSS variables are the primary artifact; utilities are accessors on top.
- **Runtime Theme Switching**: Override CSS variables at `:root` or `[data-theme]` selector level — no rebuild. Multi-theme design systems use this for instant dark mode / brand switching.
- **Container Queries**: Native container query support for component-scoped responsive design.

### 7. shadcn/ui — Source Ownership & Dual-Base Architecture

shadcn/ui distributes **source code**, not compiled packages. Components are copied into your project — you own every line. No `node_modules/shadcn-button` version lock.

- **Base UI is Default (July 2026)**: New projects default to Base UI (by the same team that built Radix). Radix remains fully supported — both ship for every component. Base UI is at v1.6.0 with 6M+ weekly downloads. Projects created via `npx shadcn init` pick Base UI 2:1 over Radix.
- **Headless Primitives**: Accessibility (ARIA, keyboard navigation, focus management, screen reader support) inherited from the underlying primitive — building inclusive applications by default.
- **`class-variance-authority` (CVA)**: Type-safe, systematic styling variants. TypeScript knows exactly which variants are available. All definitions in one place for consistency.
- **`asChild` Pattern**: Slot primitive allows swapping the underlying DOM element while maintaining styles and functionality.
- **Registry Protocol**: CLI-backed HTTP API serves components as static JSON, transformed per-project (aliases, icon libraries, CSS frameworks).

### 8. Purposeful Micro-Interactions — Spring Physics & Functional Feedback

In 2026, motion is the primary layer carrying brand voice. Linear and cubic-bezier easing read as "older product." Spring physics (mass, stiffness, damping) feel alive.

- **Four Purposes Test**: Every micro-interaction MUST serve at least one: (1) **Feedback** — confirm action happened (button press state). (2) **Status** — communicate system state (progress, loading, success). (3) **Transitions** — show spatial relationships (modal slide from trigger point). (4) **Guidance** — direct attention (pulse on first-use element, shake on invalid input). If it doesn't serve one, cut it.
- **Duration**: Micro-interactions $100$–$400\text{ms}$. State confirmation feedback MUST appear within $< 100\text{ms}$ to close the action-perception loop.
- **Spring Physics**: Use mass, stiffness, damping parameters (Framer Motion, React Spring, GSAP modern builds). Slight overshoot on entry, gentle settle on exit.
- **Scroll-Driven Animation**: CSS `scroll-timeline` and `view-timeline` enable choreographed hero sections and section transitions without JavaScript.
- **View Transitions API**: Native browser API for smooth page/state transitions.
- **Haptic-Style Visual Feedback**: Tactile response patterns on web — scale reduction on press, ripple effects, weight shifts.
- **Trigger-Rules-Feedback-Loops Framework** (Dan Saffer): Trigger (user/system/time-based) → Rules (what happens) → Feedback (visual/audio/haptic output) → Loops & Modes (continuation/edge cases).

### 9. Dark Mode — OKLCH Luminance Hierarchy

HSL is NOT perceptually uniform — two colors with identical HSL lightness can look wildly different in brightness. OKLCH (CSS Color Module Level 4) is perceptually uniform: same L value = same perceived brightness. Production usage at 18% and rising 1.4 points/month (State of CSS 2025).

- **OKLCH Over HSL**: `oklch(L C H)` where L matches what the eye sees. Use for all color scales — steps stay even.
- **Four Surface Elevation Levels**: (1) Base background (darkest, where content sits). (2) Primary elevated (cards, panels, sidebars). (3) Secondary elevated (nested cards, hover/active states). (4) Overlay (modals, tooltips, dropdowns).
- **Luminance Hierarchy, Not Shadows**: Shadows do not read on dark backgrounds — there is no contrast between a dark shadow and a dark background. As surfaces elevate, they get **lighter** (5–8% luminance step per level), tinted toward the primary brand color. Material You calls this "tonal elevation."
- **Contrast Engineering**: Pure dark background (`oklch(0.10 0 0)` / `#09090b`) with high-contrast text (`oklch(0.92 0 0)` / `#f4f4f5`). Pair shades from opposite ends of the scale — never mid-against-mid (400 on 500 almost never passes).
- **Accessibility**: WCAG 4.5:1 body text, 3:1 large text or UI, 7:1 AAA. APCA perceptual contrast: aim Lc 60+ large, Lc 75+ body. Re-check every pairing in dark mode — do not just invert.

### 10. Design System Governance & Token Consistency

- **Three-Tier Token System**: Primitives (raw values) → Semantic (purpose-driven) → Component (variants). Encode relationships once, derive both light and dark themes from the same semantic layer.
- **Strict Token Adherence**: Spacing (`gap-2`, `gap-4`, `p-6`), typography scale, border radius (`rounded-xl`), shadow elevation. Reject ad-hoc hardcoded offsets in favor of dynamic container math.
- **W3C Design Token Spec**: Design token specifications at W3C Candidate Recommendation — use standard token formats for interoperability.
- **Accessibility Baked In**: Contrast ratios encoded in token relationships, not checked after the fact. Impossible pairings are structurally prevented, not audited later.

### 11. Mobile Thumb-Zone Ergonomics (NEW — addresses Efficiency gap)

The 44px tap target rule is based on 3.5" screens held at arm's length. Phones are now 6.1–6.9". Users hold them lower, often one-handed, often while walking. The rule is incomplete.

- **Functional Area**: The thumb-reachable surface of the touchscreen. Primary actions MUST be placed within the functional area for the user's dominant hand. Designing for the wrong hand shows no ergonomic benefit.
- **Grip-Specific Accessibility Maps**: One-handed vs two-handed, portrait vs landscape — each grip has different reachability zones. Design recommendations should account for user grip.
- **Target Separation > Target Size**: Above ~40pt, target separation (spacing between targets) has a larger effect on error rate than target size. Two 44pt buttons 4pt apart produce ~11% mistap rate. WCAG 2.5.8 spacing exceptions acknowledge this.
- **Optimal Target Size**: Research shows 110–140rpx is optimal for one-handed thumb interaction. 44pt (~7mm) is a minimum, not a comfort target. Thumb pads are ~25mm vs fingertip ~16–20mm.
- **Destructive Action Placement**: Secondary/destructive actions placed where reaching is intentional — outside the easy thumb zone — to prevent accidental activation.
- **RSI Prevention**: Exaggerated thumb stretching causes repetitive strain injury. Functional area design significantly reduces muscle contraction (EMG-verified).

### 12. Undo, Cancel & Recovery Patterns (NEW — addresses Efficiency gap)

Three patterns protect destructive actions. Confirmation dialogs are the wrong default for most actions — they cause dialog fatigue where users dismiss without reading.

- **Confirmation Dialogs**: Right answer ONLY for genuinely irreversible actions used infrequently where the user can recognize the mistake when interrupted. Body text MUST describe the exact consequence ("Send this email to 1,247 recipients. This cannot be cancelled once sent."), never generic ("Are you sure?").
- **Undo Windows**: Commit the action immediately, display a transient toast with undo button (5–10 second window). Best for fast-twitch reversal where users notice mistakes immediately. Psychologically less expensive than confirmation — power users develop muscle memory for destructive actions, knowing undo is the safety net.
  - **Bulk Undo**: 50 destructive actions in a row → one undo toast restoring all 50, not 50 separate toasts.
  - **Toast Persistence**: Stay until timeout or deliberate dismissal — disappearing on unrelated interaction is frustrating.
  - **Accessibility**: Toast and undo button MUST be keyboard-accessible and screen-reader-announced.
- **Soft Delete**: Commit immediately, remove from view, keep recoverable via trash/archive/admin tool for a retention window. Catches slower second-thoughts (hours/days, not seconds) and cases where a different user realizes the mistake.
- **Decision Framework**: Pattern selection based on (1) reversibility (irreversible / user-reversible / admin-reversible / reversible-with-effort), (2) frequency, (3) recovery cost, (4) cognitive load, (5) trust/transparency. Most production apps use 3–4 patterns in different parts of the product.
- **Anti-Pattern**: Dialog before every destructive action regardless of severity. Produces dialog fatigue — the dialog stops being a safety net and becomes only friction.

### 13. Performance Perception — Loading State Strategy (NEW — addresses Efficiency gap)

Skeleton screens won the loading-state war around 2018, but most implementations make perceived performance worse, not better. The Viget study (136 participants) found skeleton screen users estimated waiting LONGER than spinner users and rated the experience more negatively.

- **Decision Framework by Duration**:
  - $< 400\text{ms}$: Show nothing — a flash of skeleton-then-content is more jarring than a beat of delay. Delay the loading state.
  - $400\text{ms}$–$3\text{s}$: Skeleton screen IF the layout is predictable (feed of cards, profile page, product grid — not a dashboard with conditional widgets). Skeleton MUST match real layout closely (same row count, similar block sizes, similar rhythm).
  - $3\text{s}$–$10\text{s}$: Progress indicator with honest progress information. Skeletons that linger make users suspect something is broken.
  - $> 10\text{s}$: Progress bar with estimated time or step count. Users need to know the wait is bounded.
- **Optimistic UI**: Show the expected result immediately, reconcile with server response. MUST build rollback as carefully as the happy path — failed optimistic updates need graceful reversion.
- **Content-First Rendering**: Show what you have, more is on the way. Works when initial paint is meaningful and subsequent loads feel instant.
- **Skeleton Matching Rule**: If the skeleton shows three cards and the response returns one, you have replaced a spinner with a flicker. An undefined skeleton is worse than a spinner.
- **Progressive Loading**: Load critical path content first (above-the-fold, primary text), defer secondary content. Prioritize LCP element rendering.

### 14. Color System — OKLCH Perceptual Uniformity (NEW)

- **OKLCH**: `oklch(L C H / A)` — Lightness (0–1, matches perceived brightness), Chroma (saturation), Hue (degrees), Alpha. Standardized in CSS Color Module Level 4. All major browsers support it.
- **50–950 Scales**: Build comprehensive scales so every role has a shade. Light backgrounds want dark text (50 bg + 700+ text). Dark backgrounds want light text (900 bg + 100–200 text). Never mid-against-mid.
- **Semantic Naming**: Export semantic names (`surface-base`, `text-primary`, `border-subtle`), not raw hex. Tokens carry meaning; values can change without touching components.
- **Material 3 Dynamic Color**: Architecture for runtime color extraction from user wallpapers/brand inputs — design systems should support dynamic color schemes.

### 15. Information Architecture & Cognitive Load (NEW)

- **5-Step Design Thinking**: Empathize → Define → Ideate → Prototype → Test. Applies to products at every scale.
- **Alternative Frameworks**: Double Diamond (Discover, Define, Develop, Deliver) for organizational/service design. Lean UX (Think, Make, Check) for Agile teams. Google HEART (Happiness, Engagement, Adoption, Retention, Task Success) for large products at scale.
- **Cognitive Chunking**: Break complex information into scannable units. Bento grids, card layouts, and progressive disclosure all serve this principle.
- **Progressive Disclosure**: Show essential information first, reveal advanced options on demand. Reduces initial cognitive load without hiding capability.
- **Consistent Spatial Memory**: Help mechanisms, navigation, and error feedback in fixed relative positions across all pages (WCAG 3.2.6).

### 16. Competitive Landscape Awareness (NEW)

The UI/UX skill ecosystem in 2026:

- **Tools**: Figma dominates (67% of job listings, real-time collaboration, Dev Mode). Framer for prototyping. FigJam for ideation.
- **Courses**: Google UX Design Professional Certificate (entry-level, $294). Designlab UX Academy (portfolio-focused, $5,500, 1-on-1 mentorship, job guarantee). Skillshare ($168/yr, short classes).
- **Frameworks**: Design Thinking (Stanford d.school), Double Diamond, Lean UX, Google HEART.
- **Key Insight**: Entry-level roles now draw 300–500 applications. Portfolios (3–5 case studies with original research) differentiate, not certificates. Only 55% of companies conduct UX testing — competitive opportunity.
- **Ciel's Position**: `ui_ux_mastery` fills a gap no existing Ciel seed skill covers — design governance, accessibility enforcement, performance budgeting, and agentic UI patterns with Council oversight. No overlap with the 33 existing infrastructure-focused seed skills.

### 17. Anti-Slop Design Principles — Defeating Distributional Convergence (NEW)

AI-generated UI converges on a statistical average — the "centroid look" — because language models predict the most probable token from training data. Point ten different products at the same prompt and you get the same output: Inter typeface, purple-to-indigo gradient, centered hero, three rounded cards, emoji as icons. This is **structural, not a capability gap**. The model is succeeding at a different task than intended: generating something defensibly plausible rather than something specifically right.

**The Centroid Look — Patterns That Signal AI Slop:**

- **Color**: Blue-to-purple gradient (blue-600 → purple-600), violet primary (#7c3aed — shadcn/ui default that escaped), gradient text on dark backgrounds
- **Typography**: Inter everywhere (or Roboto). No secondary typeface. No typographic personality
- **Layout**: Centered hero → one-line subhead → three identical feature cards → centered CTA. The skeleton that satisfies symmetry without requiring a hierarchy decision
- **Icons**: Emoji as feature icons (🚀✨🔒 arranged in rows of three) instead of custom or branded iconography
- **Contrast**: CTA buttons at ~2.5:1 contrast — technically present, practically invisible
- **Motion**: Fade-up-on-scroll on every section, identical easing everywhere
- **Copy**: Marketing copy that names no specific feature. "Transform your workflow" / "harness the power" / "in today's digital landscape"
- **Density**: White space so aggressive it reads as absence of content rather than breathing room

**Anti-Convergence Mandates:**

1. **Brand Specificity**: Every design decision MUST be defensible against "why this and not the default?" If the answer is "it's what the model produced," it's slop. Override at least 3 centroid defaults per page (color, type, layout, or icon system).
2. **Point of View**: A design without a point of view is slop regardless of polish. The design MUST communicate a specific brand identity, not a generic "modern web" aesthetic. If you removed the logo, could anyone tell whose product it is?
3. **Hierarchy Decisions**: The three-card grid is the absence of a hierarchy decision. When content matters differently, size it differently. Bento Grid tier hierarchy (Standard 4) exists precisely to prevent this.
4. **Typographic Range**: Use at least two typefaces with intentional contrast — not Inter + Inter. The typeface choice is a brand signal (Aesthetics lens), not an efficiency mandate. For utility-first internal tools where efficiency is paramount, a single typeface is acceptable.
5. **Icon Authenticity**: No emoji as feature icons in production/marketing surfaces. Emoji are universally recognized (zero learning curve) and acceptable for internal tools or rapid prototyping. For user-facing product surfaces, use custom SVG, branded iconography, or a deliberately chosen icon library with consistent stroke weight.
6. **Copy Specificity**: Marketing copy MUST name specific features, specific numbers, specific outcomes. Generic advice that "could apply to anyone" is the textbook definition of slop.
7. **Contrast Verification**: All CTA buttons MUST meet WCAG 3:1 contrast minimum (Standard 3). The ~2.5:1 AI default fails accessibility AND converts poorly.

**The Slop Test**: Four or more centroid patterns on the same page = AI slop in full. One can be coincidence. Four is the average asserting itself because no constraints were provided.

**Distributional Convergence Mechanism**: The model reverts to the center of its training distribution. The center of "modern web UI" is a very specific, very repeated look. The fix is **process and context, not a cleverer prompt** — provide brand constraints, design tokens, explicit anti-defaults, and reference designs that diverge from the centroid.

### 18. Content Provenance & Craft Signals (NEW)

In an era where AI-generated content accounts for 44% of uploads on some platforms but draws only 1-3% of streams, audiences are developing slop detection instincts. The "struggle premium" — visible evidence of human effort — drives perceived value. Designs MUST signal their provenance and craft.

**Content Provenance — C2PA & EU AI Act Compliance:**

- **C2PA Content Credentials**: A widely adopted open standard for provenance metadata (Adobe, Microsoft, Google, OpenAI). JSON-LD manifest embedded in content files, signed with X.509 certificates, tamper-evident. Embedding by format: JPEG (APP11/JUMBF), PNG (caBX chunk), PDF (XMP), text/HTML (HTTP header or .c2pa sidecar). **Security caveat**: Independent analysis (IACR eprint 2026/804) identifies known limitations — timestamp agreement failures, inadequate certificate revocation, validator inconsistency. C2PA is one approach among possible solutions, not a formally approved standard under EU AI Act. Implementation MUST follow C2PA Security Considerations: validate manifest data before rendering, treat manifest content as untrusted input, prevent XSS/UI injection from malicious manifests.
- **EU AI Act Article 50** (effective August 2, 2026): Requires machine-readable marks on AI-generated content AND perceivable disclosure at point of interaction — not just metadata. AI-generated UI artifacts (images, illustrations, generated layouts) distributed in EU markets SHOULD carry provenance metadata. **Exemption**: Source code is exempt from marking obligations under EU guidelines. C2PA is a leading candidate approach but the EU Commission has not named an approved technical standard.
- **Disclosure Hierarchy**: AI-assisted (human-controlled, human-edited) vs AI-generated (model-produced, minimally edited). The distinction matters — "slop is not defined by whether AI wrote it, but by whether a human took responsibility for it."

**Visible Effort Signals — The Struggle Premium:**
Research shows visible effort cues have statistically significant effects on perceived authenticity:

1. **Process Documentation** (23.1% recognition rate): Show the design process — wireframes, iterations, rejected alternatives. A design that appears fully-formed from nowhere reads as AI-generated.
2. **Time Investment** (15.6%): Document time spent on key decisions. "We spent 3 weeks on the navigation pattern because..." signals craft.
3. **Written Rationale** (15.0%): Every major design decision should have a written explanation. Not "it looks good" — "we chose this because it solves [specific problem] for [specific user]."
4. **Mistakes & Iteration**: Show changes made during creation. Perfect outputs without visible process feel less meaningful (51.5% agreement). Bob Ross "happy accidents" — experimentation is the root of creation.
5. **Specificity**: Real names, real dates, real places, real numbers. Generic statements are the #1 slop signal. "Designed for 12,000 daily active users in the healthcare vertical" vs "designed for modern teams."

**The Goalpost Effect**: Exposure to AI-made products shifts evaluation criteria — qualities less applicable to AI become MORE valuable in human-made work. After seeing AI slop, evaluators prioritize intentionality, process visibility, emotional depth, and specific craft. Designing WITH this in mind means leading with what AI can't do.

**Anti-Slop Linting for Generated UI Code (advisory, context-aware):**

- **Em-dash density**: AI overuses em-dashes (—) where commas or nothing would do. Flag > 2 per 100 words in UI copy. Advisory — em-dashes are legitimate punctuation; flag density, not presence.
- **Vocabulary tells**: "delve," "tapestry," "nuanced," "multifaceted," "harness," "transform," "cutting-edge" in UI copy are AI slop signals. Context-aware — these words may be appropriate in specific contexts; flag clusters, not individual uses. Replace with specific, concrete language where the word adds no information.
- **Structural uniformity**: Every paragraph the same length, every section the same structure = AI default. Vary rhythm intentionally.
- **List addiction**: Bullets where prose would be clearer. Not everything is a list.
- **Hedging**: "it's worth noting," "it's important to remember," "it's crucial to consider" — hollow qualifiers that say a lot while conveying little. Cut them.
- **No personal voice**: If the UI copy reads like it could be sent to anyone in any industry without modification, it's slop. Inject brand voice and point of view.

**Craft Signals — Application Guidance:**

- `ui.craft_signals` is **optional for simple artifacts** (single components, small fixes). Process documentation and provenance metadata are proportional to artifact scope — a button component does not need a 3-week process log.
- Process documentation serves **trust and provenance**, not interaction speed. It belongs in the artifact's metadata, changelog, or design rationale — not in the user-facing UI.
- C2PA provenance embedding is **regulatory compliance friction** for EU distribution, not an efficiency optimization. Apply when distributing to EU markets, not for internal development.

---

## Design Council Scoring Rubric Integration

Each UI/UX artifact submitted to Ciel is evaluated across the 5 Design Council lenses using these standards:

- **Clarity Lens**: Evaluates Bento Grid intent hierarchy, information architecture, cognitive chunking, typography, and visual hierarchy (Standards 4, 15).
- **Inclusion Lens**: Enforces WCAG 2.2 success criteria (Focus Not Obscured, Target Size, Dragging Movements, Consistent Help, Redundant Entry) and WCAG 3.0 forward readiness. Any score $\le 3.0$ triggers an absolute veto (Standard 3).
- **Efficiency Lens**: Enforces Core Web Vitals field data thresholds (INP $\le 200\text{ms}$, LCP $\le 2.5\text{s}$, CLS $\le 0.1$), thumb-zone ergonomics, undo/cancel recovery patterns, and performance perception loading strategy (Standards 2, 11, 12, 13).
- **Aesthetics Lens**: Evaluates Liquid Glass three-layer composite, OKLCH dark mode luminance hierarchy, spring-physics micro-interactions, and color system perceptual uniformity (Standards 5, 8, 9, 14).
- **Actionability Lens**: Evaluates Agentic UX transparency patterns (Intent Preview, Autonomy Dial, Action Receipts, Audit Trail), micro-interaction feedback, conversion affordance, and anti-slop brand specificity (Standards 1, 8, 17).
- **Craft & Provenance Lens** (NEW): Evaluates content provenance (C2PA, EU AI Act Art.50), visible effort signals (process documentation, rationale, specificity), and anti-slop linting (distributional convergence, centroid look detection, vocabulary tells) (Standards 17, 18).

---

## Research Sources

This document synthesizes findings from 27 web searches (July 2026) across:

- W3C WCAG 2.2 Recommendation & WCAG 3.0 Working Draft (March 2026)
- web.dev Core Web Vitals documentation & CrUX field data analysis
- Smashing Magazine, HatchWorks, Thiago Patriota, transparencypatterns.com — Agentic UX pattern libraries (16+ patterns)
- Tailwind CSS v4 official docs & migration guides
- shadcn/ui changelog (July 2026 — Base UI default)
- IEEE DataPort & ergonomics research — thumb-zone reachability studies
- 137Foundry & UX Patterns Guide — undo/cancel/soft-delete decision frameworks
- 72Technologies & Codexical — skeleton screen vs spinner science (Viget study)
- Creative Alive & UIGuides — 2026 micro-interaction motion UX rules
- Muzli & framingui — dark mode design token systems
- SaaSFrame & studiomeyer — bento grid intent hierarchy research
- Setproduct & lucky.graphics — liquid glass composite material architecture
- Coursera, edubracket, UXCrush — UI/UX competitive landscape 2026
- Wikipedia, Columbia IGP Report, arXiv (Kommers et al.) — AI slop definition, 7Vs framework, three prototypical properties
- Superdesign, Sailop, Built In, booplex — distributional convergence, 90+ AI design patterns, centroid look
- C2PA Specification 2.4, EU AI Act Article 50, US Copyright Office Part 2 Report — content provenance, legal framework
- arXiv (Struggle Premium), Observer, Frontiers in Psychology — visible effort signals, lay theory violations, goalpost-moving effect
- Fast Company, Fortune, The Next Web — dead internet theory, bot majority June 2026, slop ceiling
- The Verge, The Bunker, Another Rodeo, Paula Righetti — indie web revival, handmade web, hand-coded as hand-made
- bestprompt.art, SurePrompts, ArtisticMonk — prompt engineering as craft, six-slot brief, deliberate underspecification
