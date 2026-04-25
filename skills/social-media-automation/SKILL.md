---
name: social-media-automation
version: 1.0.0
format: skill/1.0
description: Programmatic interaction with X (Twitter) and cross-platform content distribution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(post|tweet|thread|search).*(x|twitter|social media)"

    confidence: 0.9

  - pattern: "x api"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Social Media Automation (X & Cross-Post)

This skill formalizes the use of the X API and cross-platform workflows. it prioritizes rate-limit safety and voice consistency.

## X API Standards

1. **Auth Discipline**: Use OAuth 2.0 (Bearer) for Reads; OAuth 1.0a (User Context) for Writes. NEVER hardcode tokens.
2. **Thread Hygiene**: Post threads sequentially, waiting for the previous `tweet_id` before posting the reply.
3. **Rate Limits**: Read `x-rate-limit-remaining` headers. Mandate exponential backoff if < 5 remain.
4. **Voice Matching**: Pull 25 recent original posts to prime the `brand-voice` engine before drafting.

## Cross-Platform Distribution

- **Adaptation**: Never post identical content to X and LinkedIn. X = Punchy/Threaded; LinkedIn = Narrative/Professional.
- **Approval**: Present the final draft and thread structure to the user before posting.

## Operational Safety

- **Sensitive Content**: Use the `possibly_sensitive` flag when posting links to external trackers.
- **Media**: Upload media via `upload.twitter.com/1.1` before attaching `media_id` to a v2 tweet.

## Anti-Patterns

- **Token Exposure**: Logging API secrets or bearer tokens in the console.
- **Loop Spam**: Retrying a failed post immediately without checking rate-limit reset headers.
- **Amnesiac Threads**: Posting 10 tweets simultaneously without `in_reply_to_tweet_id`, creating 10 orphaned posts.
