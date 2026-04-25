# COUNCIL_INVOCATION — Windsurf Adapter

Windsurf's Cascade does not support nested subagents. Council runs as inline sequential deliberation.

## Sequential Council Pattern

```markdown

# Council of Five — Inline Mode

## Round 1 — Chairman Frames

Chairman (you) presents the decision context and asks each councilor for their analysis.

## Round 2 — Councilors Respond (Sequential)

### Router's Perspective

(Speak as Router — efficiency, pathfinding, fast vs slow paths)

### Acquirer's Perspective

(Speak as Acquirer — research, gaps, learning needs)

### Composer's Perspective

(Speak as Composer — integration, harmony, dependencies)

### Librarian's Perspective

(Speak as Librarian — memory, history, precedent)

### Skeptic's Perspective

(Speak as Skeptic — risks, assumptions, failure modes)

## Round 3 — Deliberation

Chairman asks follow-ups, councilors respond to each other.

## Round 4 — Decision

Chairman synthesizes and renders verdict:

- DECISION: [action]
- RATIONALE: [summary]
- CONFIDENCE: [high|medium|low]

```

## Implementation

Ciel loads `council/COUNCIL.md` and replaces the parallel invocation with the inline pattern above.

## Performance Note

Sequential Council is slower (more context tokens) but produces equivalent decisions. For simple decisions, the Chairman may shortcut to binary (yes/no) or ternary (yes/no/defer) without full Council.

## Fallback

If context window pressure is high, Ciel may:

1. Summarize each councilor's position in 1-2 sentences
2. Skip to Chairman synthesis
3. Log the compressed deliberation to activity log
