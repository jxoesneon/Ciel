# MULTIMODAL — Gemini CLI

Gemini CLI exposes multimodal generation through model extensions:

- **Imagen** — image generation
- **Veo** — video generation
- **Lyria** — audio generation

Ciel routes to these when a request includes media intent.

## Route Triggers

| Request shape | Target |
| --- | --- |
| "generate an image of …" / "make a logo…" | Imagen |
| "make a video of …" / "animate …" | Veo |
| "generate a voice-over …" / "produce audio of …" | Lyria |

## Cost & Consent

Multimodal calls are typically expensive. Ciel:

- classifies them as at least *mid-risk* (cost axis),
- invokes the LLM judge with an estimated cost,
- shows the estimate to the user if above `risk.config.cost_threshold`,
- caches generated assets in `.ciel/cache/media/` keyed by prompt hash to avoid re-generation.

## Artifact Handling

Generated assets are placed in a location the user specified (or the project's `assets/` folder by default) with a sidecar `.meta.json`:

```json
{ "prompt": "...", "model": "imagen-3", "seed": 123, "created": "2026-01-15T..." }
```

## Fallback

On Claude Code, no native multimodal generation. Ciel will:

- check for an MCP-exposed equivalent (e.g. `gemini-image-generator` MCP),
- if unavailable, escalate with a message that multimodal is unavailable on this runtime.

## Safety Prompts

All multimodal prompts pass Safety member's content filter before generation. Disallowed content triggers an immediate refusal with a logged rationale.
