---
name: hermes-cli-skinning
version: 1.0.0
format: skill/1.0
description: CIEL's framework for CLI theming via image-to-ASCII banner conversion. Turns any reference image into a complete CLI skin with rich-markup pictorial banner art.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:design"]
triggers:
  - pattern: "(skin|theme|banner).*(image|ascii|art|picture)"
    confidence: 0.9
  - pattern: "(image|picture).*(skin|theme|banner|cli)"
    confidence: 0.85
  - pattern: "hermes.*(skin|theme|banner)"
    confidence: 0.8
source: { tier: 3, origin: "https://github.com/Sahil-SS9/hermes-Custom-CLI-Themes" }
dependencies: { skills: [], mcp: [], system: ["python3", "curl"] }
side_effects: ["shell", "network"]
---

# CIEL ADAPTATION: Hermes CLI Skinning

This skill converts any reference image into a complete CLI skin with an image-derived banner in rich markup (`[#rrggbb]char[/]` runs). While originally built for Hermes CLI, the image-to-ASCII pattern is reusable for any CLI banner system — the density ramp, luminance mapping, and run-length rich-markup compression are platform-agnostic primitives.

## When to Use

- User asks for a theme/skin based on an image ("make my terminal look like this artwork").
- Existing skins are palette-only; this adds true pictorial banner art.
- Any CLI theming task requiring image-to-ASCII conversion with color preservation.

## Procedure

1. **Gather inputs**: Reference image URL/path + theme identity (name, persona, accent hues). Download locally: `curl -fsSL URL -o /tmp/ref.jpg`.
2. **Palette extraction**: Read dominant hues (vision or quantize to ~8 colors). Derive: background (darkest), accent (most vivid), text (lightest), plus semantic ok/warn/error.
3. **Convert — Option A** (fast preview): `brew install TheZoraiz/ascii-image-converter; ascii-image-converter ref.jpg -d 64,60 --save-txt out/`. ANSI truecolor output — use Option B for final artifact.
4. **Convert — Option B** (recommended Pillow pipeline):
   - Resize to W columns (~64) × round(W × h/w × 0.5) rows (terminal cells are ~2x tall).
   - Quantize to 24 colors (`Image.quantize(method=MEDIANCUT)`) for compact YAML (~26KB at 64×57).
   - Map luminance (0.2126R+0.7152G+0.0722B) onto ramp `" .:-=+*#%@"`.
   - Emit run-length rich markup: consecutive same-char+same-color cells become one `[#hex]ccc[/]` run.
5. **Write the skin**: Full schema with all `colors` keys present, `background` set, `ui_accent` ≥ 4.5:1 vs background. Inject art under `banner_hero: |-` indented exactly 2 spaces.
6. **Validate + activate**: `python3 -c "import yaml; yaml.safe_load(open(skin_path))"` then `hermes config set display.skin <name>`.

## I/O Contract

- **Input**: image path/URL, skin name, optional persona notes.
- **Output**: skin YAML path, activation status, optional preview PNG.
- **Idempotent**: yes — re-running overwrites the same skin file.

## Anti-Patterns

- **The 3-space indent bug**: First art line after `banner_hero: |-` gets one extra space — always re-validate YAML and grep the first art line's indent.
- **ANSI escapes in skin files**: Convert truecolor output to rich `[#hex]` markup — raw ANSI breaks YAML parsing.
- **Wrong char aspect**: `CHAR_ASPECT = 0.5`, not 1.0 — otherwise figures render stretched tall.
- **Generic theming**: Keep spinner faces, branding, and tool_emojis themed to the persona, not generic defaults.
