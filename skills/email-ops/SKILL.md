---
name: email-ops
description: Evidence-first mailbox triage, drafting, and send-verification workflow.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
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
