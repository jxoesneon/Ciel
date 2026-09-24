---
name: hermes_image_skin
version: 1.0.0
description: Author a Hermes CLI skin whose banner_hero is true image-to-ASCII art converted from any reference image.
triggers: [image skin, ascii banner, skin from image, hermes theme, banner art, ciel theme]
tags: [theming, ascii-art, hermes, creative, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: MIT
source:
  tier: 3
  origin: composed
  references:
    - https://github.com/Sahil-SS9/hermes-Custom-CLI-Themes
    - skill:creative/ascii-art
composes:
  - creative/ascii-art/SKILL.md        # density ramp + converter knowledge
  - ciel/seed_skills/json_yaml_toml_parser/SKILL.md  # skin YAML validation
flow: |
  1. creative/ascii-art: pick conversion approach (converter binary or Pillow pipeline)
  2. NEW primitive image_to_rich_markup: sample grid -> luminance ramp -> [#hex] runs
  3. json_yaml_toml_parser: validate emitted skin YAML before activation
---

# hermes_image_skin

Turn any reference image into a complete Hermes skin with an image-derived
`banner_hero` in rich markup (`[#rrggbb]char[/]` runs), following the
Sahil-SS9/hermes-Custom-CLI-Themes end-to-end style.

## When to Use

- User asks for a theme/skin based on an image ("make my terminal look like this artwork").
- Existing skins are palette-only; this adds true pictorial banner art.

## Procedure

1. **Gather inputs.** Reference image URL/path + theme identity (name, persona,
   accent hues). Download the image locally first (`curl -fsSL URL -o /tmp/ref.jpg`).
2. **Palette extraction.** Read dominant hues directly off the image (vision or
   quantize to ~8 colors). Derive: background (darkest), accent (most vivid),
   text (lightest), plus semantic ok/warn/error kept recognizable.
3. **Convert — Option A: ascii-image-converter** (fast preview):
   ```bash
   brew install TheZoraiz/ascii-image-converter/ascii-image-converter
   ascii-image-converter ref.jpg -d 64,60 --save-txt out/
   ```
   Its color output is ANSI truecolor; Hermes banners need rich markup, so use
   Option B for the final artifact.
4. **Convert — Option B: Pillow pipeline (recommended)**:
   - Resize to W columns (~64) x round(W * h/w * 0.5) rows (terminal cells are ~2x tall).
   - Quantize to 24 colors (`Image.quantize(method=MEDIANCUT)`) so run-length
     compression keeps the YAML compact (~26KB at 64x57).
   - Map luminance (0.2126R+0.7152G+0.0722B) onto ramp `" .:-=+*#%@"` — dark
     backgrounds stay sparse instead of becoming walls of `@`.
   - Emit run-length rich markup: consecutive same-char+same-color cells become
     one `[#hex]ccc[/]` run.
5. **Write the skin.** Full schema per `hermes-agent` themes reference — every
   top-level `colors` key present, `background` set, `ui_accent` >= 4.5:1 vs
   background. Inject art under `banner_hero: |-` indented exactly 2 spaces.
6. **Validate + activate.**
   ```bash
   python3 -c "import yaml; yaml.safe_load(open(skin_path))"
   hermes config set display.skin <name>   # never hand-edit config.yaml
   ```

## Pitfalls

- **The 3-space indent bug**: when generating indented block scalars line by
  line, the FIRST art line after `banner_logo: |-` / `banner_hero: |-` tends to
  get one extra space (leading-newline artifacts). Always re-validate YAML and
  grep the first art line's indent. Symptom: `expected <block end>, but found '['`.
- **ANSI escapes don't belong in skin files** — convert truecolor output to
  rich `[#hex]` markup.
- **CHAR_ASPECT = 0.5**, not 1.0 — otherwise the figure renders stretched tall.
- Keep spinner faces / branding / tool_emojis themed to the persona, not generic.

## I/O Contract

```yaml
io_contract:
  input: { image_path_or_url, skin_name, persona_notes? }
  output: { skin_yaml_path, activated: bool, preview_png_path? }
  idempotent: true
  side_effects: [fs_write, config_change]
```

## Safety

- Writes limited to `<hermes-home>/skins/<name>.yaml`; activation via
  `hermes config set` only. Low risk.
