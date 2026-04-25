---
name: email-ops
version: 1.0.0
format: skill/1.0
description: Evidence-first mailbox triage, drafting, and send-verification workflow.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(read|send|triage|draft).*(email|mail|inbox)"

    confidence: 0.9

  - pattern: "reply to (the|this) email"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Email Ops (Mailbox Operations)

This skill manages formal mailbox interaction. It focuses on thread history, brand voice alignment, and explicit send verification.

## The Email Lifecycle

1. **Thread Context**: Before drafting a reply, the Orchestrator MUST read the preceding 2-3 messages in the thread to identify commitments or open loops.
2. **Voice Alignment**: Pull `brand-voice` for all external or investor-facing correspondence.
3. **Drafting**: Present the final `Subject` and `Body` to the user.
4. **Verification**: For live sends, confirm the message appears in the `Sent` folder or equivalent store before reporting success.

## Operational Rules

- **Account Resolution**: Explicitly identify which mailbox (Work, Personal, Project) is being used.
- **Draft-First**: Default to drafting. Only perform a live send if the user's directive is unambiguous (e.g., "Send this now").
- **Redaction**: Never expose secrets or sensitive metadata in session notes or logs.

## Anti-Patterns

- **Blind Replies**: Writing a response without reading the full thread history.
- **Unverified Sends**: Claiming an email was sent without checking the `Sent` folder.
- **Surface Blur**: Using `email-ops` for DM or text message tasks (use `messages-ops`).
