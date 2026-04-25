---
name: video-operations
version: 1.0.0
format: skill/1.0
description: CIEL's framework for AI-assisted video editing, perception, and server-side processing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(edit|cut|transcode|index|stream).*(video|footage)"
    confidence: 0.9
  - pattern: "videodb"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Video Operations (Perception & Editing)

This skill manages both the "Perception" (understanding video) and "Editing" (cutting footage) of media. It prioritizes server-side processing via VideoDB and deterministic CLI tools like FFmpeg.

## Core Capabilities

### 1. Perception (VideoDB)
- **Ingest**: Upload local files or URLs for indexing.
- **Search**: Perform semantic search inside video (Visual, Spoken, Keyword).
- **Summarize**: Generate actionable summaries of desktop sessions or recordings.
- **Alert**: Set real-time triggers on visual events (e.g., "Alert when a login field appears").

### 2. Editing Pipeline (Compression over Generation)
- **Structure (Layer 2)**: Use CIEL to plan the edit decision list (EDL) from transcripts.
- **Deterministic Cuts (Layer 3)**: Use FFmpeg for splitting, trimming, and concatenation.
- **Augmentation (Layer 5)**: Generate specific assets (Voiceover, Music, SFX) using ElevenLabs or fal.ai.

### 3. Re-framing
- YouTube (16:9) -> TikTok/Reels (9:16) using `ffmpeg` center-crop or `videodb.reframe(mode="smart")`.

## Operational Protocol
- **Edit, Don't Generate**: Focus on structuring real footage rather than prompt-to-video generation.
- **Pure Functions**: Keep processing logic separate from caching (use `content-hash-cache-pattern`).
- **Server-Side First**: Prefer VideoDB for heavy transcoding and indexing tasks.

## Anti-Patterns
- **Local Encoding Churn**: Running 4-hour FFmpeg encodes on the host machine when VideoDB is available.
- **Generation-Only**: Trying to build a tutorial entirely from AI-generated clips without real footage.
- **Amnesiac Perception**: Searching a video repeatedly without building a visual index.
