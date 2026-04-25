---
name: social-and-lead-intelligence
version: 1.0.0
format: skill/1.0
description: CIEL's framework for lead scoring, social graph analysis, and warm-path outreach.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(find|score|rank).*(lead|prospect|social graph|outreach)"
    confidence: 0.9
  - pattern: "warm path discovery"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Social Intelligence (The Network Layer)

This skill formalizes the extraction of value from social graphs. it prioritizes weighted relationship math and voice-matched outreach.

## Signal Scoring (prospecting)
1. **Vertical Match (30%)**: Alignment with target industry/keywords.
2. **Activity (20%)**: Recent original posts or replies on topic (X/LinkedIn).
3. **Influence (10%)**: Follower count and engagement rate.
4. **Engagement (5%)**: Direct interaction with the user's content.

## Mutual Ranking (The Bridge Model)
- **Math**: `Score = Σ (w(target) * λ^(hops-1))`. 1-hop = 1.0; 2-hop = 0.5.
- **Tiers**:
  - **Tier 1**: Direct Bridge. Request warm intro blurb.
  - **Tier 2**: 1-hop overlap. Use shared connection as social proof in cold outbound.
  - **Tier 3**: No bridge. Use direct cold outbound based on recent timeline signal.

## Outreach Engineering
- **Voice Profile**: Run `brand-voice` on the user's last 20 posts before drafting.
- **X/LinkedIn Rule**: Never post identical copy. Adapting length, tone, and formatting per platform is MANDATORY.
- **Drafting**: Create drafts in Apple Mail or the social platform. PROHIBIT auto-sending without user confirmation.

## Anti-Patterns
- **Generic Vibe**: Drafting "Would love to connect" without a specific receipt from the target's timeline.
- **Social Proof Stacking**: Listing 10 mutuals in a first DM (feels like a bot).
- **Amnesiac Threads**: Posting an X thread without waiting for the previous tweet's ID, causing orphans.
