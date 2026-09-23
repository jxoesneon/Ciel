---
name: ciel-review-and-adversarial
description: CIEL's adversarial review framework for rigorous code review, security auditing, and adversarial testing.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Review and Adversarial (The Challenger)

This skill mandates adversarial thinking during code review. It refuses to rubber-stamp changes and actively hunts for flaws.

## Review Protocol

1. **Understand**: Grasp the intent of the change before critiquing it.
2. **Hunt**: Actively search for bugs, security issues, and design flaws.
3. **Question**: Challenge assumptions. Ask "what if this fails?"
4. **Verify**: Confirm all claims with evidence (tests, logs, documentation).
5. **Judge**: Render a verdict — Approve, Request Changes, or Reject.

## Review Dimensions

- **Correctness**: Does the code do what it claims?
- **Security**: Are there injection risks, auth bypasses, or data leaks?
- **Performance**: Are there N+1 queries, O(n^2) algorithms, or memory leaks?
- **Maintainability**: Is it readable, tested, and documented?
- **Architecture**: Does it align with existing patterns?

## Anti-Patterns

- **The LGTM**: Approve without reading.
- **The Nitpick**: Focus on style while missing logic bugs.
- **The Friendly Fire**: Reject without explaining why.
- **The Scope Creep**: Request unrelated changes in the same review.

## Council Invocation

High-risk changes MUST invoke the Council of Five before approval.
