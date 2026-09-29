# Hyperframes Composition Brief: Ciel

## Objective
Create a short launch-style brag video for Ciel — an autonomous orchestration
intelligence whose Council of Five votes on its own changes.

## Output
- Composition directory: `brag-output/composition/`
- Rendered video: `brag-output/brag.mp4`
- Format: landscape — 1920x1080
- Duration: 117 seconds (director's cut — user-requested ~2min; single track
  covers it; music ducks 85.5–91.4 for the pre-title beat of silence)

## Source Material
- Project root: `/Users/mey/Ciel` (the Ciel repo itself — jxoesneon/Ciel)
- Primary files read: `README.md`, `ciel.skill/SKILL.md`,
  `ciel.skill/assets/images/banner.jpg` (copied to `assets/ciel-banner.jpg`),
  real session artifacts (council verdict 7.5 weighted, 180-skill index)
- Product name: Ciel
- Tagline / strongest claim: "AI Orchestration & Self-Evolution" — the banner
  tagline; stronger: it literally convenes a five-member council to approve
  its own work, and this video is itself council-approved output.
- Key UI or visual moment to recreate: the Council-of-Five verdict — five
  lens rows (Coherence 8, Capability 9, Safety 7, Efficiency 6, Evolution 7),
  weighted bar to 7.45, PASS stamp. Plus the skill-lifecycle pipeline, the
  registry count-up, the sanitizer evidence, and the polyhedron banner as
  reveal/outro visual.
- Copy that must appear verbatim:
  - "This AI votes on its own commits."
  - "CIEL" + "autonomous partner intelligence"
  - "Coherence 8 · Capability 9 · Safety 7 · Efficiency 6 · Evolution 7"
  - "7.45 — PASS"
  - "ROUTE → FETCH → SCAN → COUNCIL → VALIDATE → INSTALL" +
    "sandboxed → validated · 11 skills this session"
  - "180 skills indexed · 180 / 180" / "+11 acquired & validated — this session"
  - "1,622 secrets redacted" / "0 residual hits — verified"
  - "This video? Ciel made it." + "github.com/jxoesneon/Ciel"

## Creative Direction
- Tone preset: `cinematic`
- Creative direction: enterprise trailer for an entity that is genuinely
  governed — every on-screen claim is verifiably true.
- Interpretation: big type, long holds, dramatic reveals; SFX restraint so
  the council table lands like a verdict, not a slot machine.
- Angle: the grandiose claims are literally true; the proof is that this
  video was produced by the governed system itself.
- Hook: mono-font typed line — "This AI votes on its own commits." — cursor,
  near-silence, dark navy.
- Outro: polyhedron swell, "This video? Ciel made it.", repo URL.
- Avoid: generic SaaS language, abstract filler, waveform/equalizer visuals,
  anything implying a claim that isn't true.

## Visual Identity
- Background: #060D1F → #0A1628 (deep navy)
- Text: #E2E8F0
- Accent: #22D3EE circuit cyan; glow violet #A855F7 → #E879F9
- PASS/verdict green: #34D399 (verdict stamp only)
- Display font: Orbitron (Google Fonts)
- Mono font: JetBrains Mono (terminal/log lines)
- Visual references from the project: `assets/ciel-banner.jpg` (glowing
  polyhedron, circuit traces); council table = real data from this session's
  `~/.ciel/logs/council.log` verdict

## Storyboard
Use `brag-output/brag-plan.md` as the creative contract.

Scene summary (director's cut, 117s):
1. The Claim — [0–3.6] — typed mono line, cursor, no label; near-quiet.
2. Wordmark — [3.4–7.8] — banner swell; CIEL letters on 3.82s cue.
3. The Promise — [7.6–14.2] — left-aligned lines on 8.74/10.93/12.55 cues.
4. The Council — [14.0–21.6] — rows on 14.73–19.1 beats; 7.45; PASS on 20.19.
5. The Pipeline — [21.4–29.0] — six nodes on 22.37–26.74 cue cluster;
   "sandboxed → validated" payoff on 27.3.
6. The Guilds — [28.8–39.6] — 10-card grid cascade, one per beat 32.74–37.64.
7. The Ladder — [39.4–47.5] — three ascending tiers on 40.38/41.47/42.56.
8. The Iron Law — [47.3–54.5] — law card on 48.02, sub on 49.11; quiet hold.
9. The Registry — [54.3–62.5] — 0→180 slow-roll landing on 61.1; stamp 61.65.
10. The Evidence — [62.3–69.5] — 1,622 lands 63.29; zero-residual stamp 64.38.
11. Runtimes — [69.3–77.5] — five runtime pills on 72.02–76.38.
12. Punchline — [77.3–84.0] — "Ciel made it." on 78.56; URL on 80.75; duck.
13. Title Card + Button — [83.8–117] — silence-break: CIEL + URL land on
    91.65; tags cascade beneath on 96.55/98.2/100.38 + version line; long
    hold, fade to black 115.6–116.9.

## Audio
- Audio role: cinematic support — music sits under, never competes.
- Audio arc: near-quiet typed hook → first swell on wordmark → rhythmic
  impacts under council rows → ticking count-up → decisive punchline hit,
  then fade under the outro hold.
- Music: `assets/music/happy-beats-business-moves-vol-12-by-ende-dot-app.mp3`
  (bundled; "steady and clean — polished/cinematic"), t=0, volume ~0.3,
  fade under final hold.
- Music cue guidance: bundled preset
  `assets/music/happy-beats-business-moves-vol-12-by-ende-dot-app.music-cues.json`
  (109.96 BPM) plus a regenerated analysis with `--window-duration 117.4`
  covering the full track. Locks used: promise lines **8.74/10.93/12.55**,
  PASS **20.19**, pipeline cluster **22.37–26.74**, guild cascade on the
  beat grid **32.74–37.64**, ladder **40.38/41.47/42.56**, iron law
  **48.02/49.11**, registry land **61.1**, evidence **63.29/64.38**, runtimes
  **72.02–76.38**, punchline **78.56/80.75**, title break **91.65**, button
  ticks **96.55/98.2/100.38**. Natural quiet stretches 51–61s and 81–91.6s
  serve as act breaks (volume-ducked). Beat grid ~0.545s.
- Audio-reactive treatment: subtle — polyhedron glow and score chips may
  breathe with RMS/bass. No waveform bars, no pulsing text.
- Audio-coupled moments:
  - Scene 1 — per-character key ticks (keyboard/*, randomized, quiet)
  - Scene 3 — soft impact per lens row; heavy bell on PASS stamp
  - Scene 4 — short ticks on count-up; switch on the "+11" stamp
  - Scene 5 — impactBell on the punchline landing
- SFX selection guidance: cinematic = 2–3 big cues; prefer
  `impact/impactBell_heavy_000/003` for verdict + punchline,
  `impact/impactSoft_medium_*` for row arrivals, `ui/switch*` or
  `ui/click*` for the "+11" stamp, `keyboard/keypress-*` for typed hook.
  Staged under `composition/assets/sfx/`.
- SFX analysis: `<skill-dir>/assets/sfx/sfx-analysis.md` (skill dir =
  `~/.agents/skills/brag/`).
- Exact SFX choice: Hyperframes decides filenames/timestamps/volume from the
  implemented animation.
- Audio files: music + cue JSON + candidate SFX already copied into
  `brag-output/composition/assets/`.

## Hyperframes Instructions
Load `hyperframes-core` (composition contract + `data-*` timing),
`hyperframes-animation` (motion), `hyperframes-creative` (design/audio),
`hyperframes-keyframes` (seek-safe keyframes), `hyperframes-cli`
(lint/check/render). /brag is its own workflow — do NOT enter the
`hyperframes` entry-point intent interview or its generic promo workflow.
Prefer native Hyperframes conventions.

Requirements:
- Show the real banner asset and the real council/pipeline/sanitizer data
  (verbatim copy list).
- All text readable — honor reading-time floors in the plan.
- Total duration 117s (director's cut per user request).
- Music + SFX layer included per above.
- Multiple strong-cue locks (PASS, pipeline nodes, registry land, sanitizer,
  final URL hit); beat-grid snapping for sequential elements.
- Run `npx hyperframes check` inside `brag-output/composition/` — zero errors
  is the gate before render.
