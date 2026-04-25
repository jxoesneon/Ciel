---
name: neural-intelligence-generation
version: 1.0.0
format: skill/1.0
description: Neural search via Exa and multimodal generation via fal.ai.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(search|generate).*(exa|neural|fal.ai|image|video|audio)"
    confidence: 0.9
  - pattern: "exa web search"
    confidence: 1.0
---

# CIEL ADAPTATION: Neural Intelligence (Exa & fal.ai)

This skill formalizes the use of neural search and generative media. it prioritizes high-signal retrieval and multimodal consistency.

## Neural Search (Exa)
1. **Context First**: Use `get_code_context_exa` for API usage or library patterns. Target 3000-5000 tokens.
2. **Signal Filter**: Use `web_search_exa` with `category: "company"` or `research paper` to filter noise.
3. **Livecrawl**: Default to `livecrawl: "fallback"` to ensure current data for breaking news.

## Generative Media (fal.ai)
- **Image**: Start with `nano-banana-2` for iteration; switch to `nano-banana-pro` for final high-fidelity assets.
- **Video**: Use `seedance-1-0-pro` for text-to-video. Use `kling-video/v3/pro` if native audio is required.
- **Consistency**: Use the same `seed` when iterating on prompt wording to maintain visual composition.

## Multimodal Workflows
- **Image-to-Video**: Always upload the source image first, then pass the URL to the video model for controlled motion.
- **Video-to-Audio**: Use `thinksound` to generate ambient audio that matches the visual motion.

## Anti-Patterns
- **The 50-Result Dump**: Requesting 50 Exa results without a specific filter, drowning in low-signal noise.
- **Zero-Seed Drift**: Iterating on a prompt without a seed, causing the visual style to jump radically between versions.
- **Blind Generation**: Running expensive video models without checking `estimate_cost` first.
