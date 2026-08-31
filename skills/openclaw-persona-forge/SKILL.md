---
name: openclaw-persona-forge
version: 1.0.0
format: skill/1.0
description: CIEL's framework for AI persona generation. Forges complete character identities with soul descriptions, boundary rules, names, and avatar prompts via guided or gacha-random modes.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(persona|character|soul|identity).*(forge|create|design|generate)"
    confidence: 0.9
  - pattern: "(gacha|random|抽卡).*(persona|character|lobster|soul)"
    confidence: 0.85
  - pattern: "(lobster soul|虾魂|龙虾灵魂|openclaw)"
    confidence: 0.8
source: { tier: 2, origin: "openclaw-persona-forge" }
dependencies: { skills: [], mcp: [], system: ["python3"] }
side_effects: []
---

# CIEL ADAPTATION: OpenClaw Persona Forge

This skill generates complete AI agent personas — identity tension, soul description (SOUL.md), boundary rules, names, and avatar prompts. While originally built for OpenClaw's lobster-soul platform, the persona-generation pattern (identity × soul × rules × name × visual anchor) is reusable for any character-driven AI agent. The skill is pure generation — no network requests or file sending. Optional avatar image generation delegates to an environment-approved image skill.

## Core Formula

A good persona = **identity tension** + **boundary rules** + **personality flaws** + **name** + **visual anchor**. All five must reinforce each other.

## Modes

- **Guided mode**: Present 10 archetype directions (fallen restart, peak boredom, misplaced life, etc.). User selects or mixes.
- **Gacha mode**: `python3 ${SKILL_DIR}/gacha.py [count]` — true random from 8M combinations. Never improvise random — always run the script.
- **Refinement mode**: User provides existing SOUL.md draft; skill fills gaps (name, rules, avatar prompt).

## Generation Pipeline

1. **Direction**: Select archetype or gacha-roll. Mix-and-match allowed (e.g., "archetype 2's boredom + archetype 7's veteran warmth").
2. **Identity tension**: Past identity × current situation × inner contradiction → one-sentence soul. See `references/identity-tension.md`.
3. **Boundary rules**: Derive 2-4 rules in the character's own voice — not generic clauses. See `references/boundary-rules.md`.
4. **Name**: Provide 3 candidates with strategy type and pairing rationale. See `references/naming-system.md`.
5. **Avatar**: Fill 7 personalization variables, compose STYLE_BASE + description into prompt. If image skill available → auto-generate; else output prompt text for manual use (Gemini/ChatGPT/Midjourney). See `references/avatar-style.md`.
6. **Output**: Assemble complete persona, offer to write `SOUL.md` and `IDENTITY.md` files to user-specified directory.

## Quality Criteria

- Name alone hints at personality
- Boundary rules sound like the character speaking
- Clear personality flaw or limitation present
- Can imagine specific conversation scenarios
- Won't cause role fatigue after 30 days of use

## Error Handling

Core principle: **degrade, don't break**. Python unavailable → skip gacha, pick from 10 presets. Image skill missing → output prompt text. Image skill fails → retry once, then fall back to prompt text.

## Anti-Patterns

- **Extreme toxicity**: A persona that insults you daily becomes unbearable by day 3.
- **Over-roleplay**: Character breaks immersion when writing formal emails — persona must not block utility.
- **Excessive warmth**: Fails when honest criticism or negative feedback is needed.
- **Flawless persona**: A perfect character isn't a character — it's a manual. Flaws create depth.
- **Improvised gacha**: Never make up random results — always execute `gacha.py` for true randomness.
