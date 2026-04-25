---
name: messages-ops
version: 1.0.0
format: skill/1.0
description: Evidence-first retrieval and thread management for instant messaging surfaces (iMessage, DMs, OTPs).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(read|check).*(message|text|dm|code|otp)"
    confidence: 0.95
  - pattern: "look in (imessage|twitter|x) dms"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Messages Ops (Instant Messaging)

This skill manages live message retrieval from instant messaging surfaces. It is specialized for high-speed, evidence-based tasks like retrieving One-Time Passwords (OTPs) or inspecting DM threads.

## Workflow
1. **Resolve Surface**: Identify if the message is in a local database (iMessage), a browser-gated surface (X/Twitter), or a specific app.
2. **Search (Focused)**: For codes/OTPs, search the most recent 5-minute window first.
3. **Evidence Capture**: Return the exact text of the message along with the sender and timestamp.
4. **Handoff**: If a message requires a long-form response, hand off to `brand-voice` or `email-ops` if appropriate.

## Guardrails
- **Read-Only**: Default to read-only retrieval. Do not send messages unless explicitly requested with a specific draft.
- **Privacy**: Redact or ignore sensitive personal messages that are unrelated to the engineering task.
- **Auth Blockers**: If a surface requires MFA or manual login, stop and report the exact blocker to the human user.

## Anti-Patterns
- **Generic Search**: Searching all messages for "code" without specifying the service (e.g., "GitHub" or "Stripe").
- **Improvised DB Access**: Attempting to write raw SQL to local message databases without using established helper scripts.
- **Mixing Surfaces**: Using `messages-ops` for mailbox work (use `email-ops` instead).
