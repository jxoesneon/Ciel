# WORKFLOWS — Windsurf Integration

Windsurf supports `.windsurf/workflows/*.md` files invoked via `/[workflow-name]` slash commands. Ciel provides structured workflows for common orchestration tasks.

## Overview

Workflows differ from Skills:

| Feature | Skills | Workflows |
| --- | --- | --- |
| Invocation | Automatic via triggers | Manual via `/command` |
| Content | Full SKILL.md + resources | Step-by-step markdown |
| Use case | Teach Cascade procedures | Guide through sequences |
| Location | `.windsurf/skills/` | `.windsurf/workflows/` |

## Ciel Workflow Suite

### `/ciel-council` — Council Deliberation

Guides through Council of Five decision-making:

```markdown

# Council Deliberation Workflow

## Context

User requests high-risk operation requiring Council judgment.

## Steps

1. **Load Councilors**
   - Load Coherence, Capability, Safety, Efficiency, Evolution personas
   - Confirm Chairman (self) is active

2. **Present Case**
   - Describe the proposed operation
   - Reference risk classification rubric
   - List alternatives considered

3. **Deliberation Rounds**
   - Round 1: Each councilor states position (approve/veto/abstain)
   - Round 2: Discussion of conflicts (if any)
   - Round 3: Safety veto consideration (if applicable)

4. **Synthesis**
   - Chairman weighs votes per Constitution
   - Document decision and rationale
   - Log to `~/.ciel/logs/council.log`

5. **Execution**
   - If approved: proceed with operation
   - If vetoed: explain blockage to user
   - If escalated: request user confirmation

```

### `/ciel-acquire` — Skill Acquisition

Guides through tiered skill acquisition:

```markdown

# Skill Acquisition Workflow

## Steps

1. **Need Analysis**
   - Define the capability gap
   - Check existing seed skills
   - Query local `~/.ciel/registry/`

2. **Registry Tier**
   - Search `~/.ciel/skills/` for match
   - If found: load and integrate
   - If not found: proceed to MCP tier

3. **MCP Tier**
   - Check configured MCP servers
   - Identify tool providing capability
   - If viable: use MCP directly
   - If not viable: proceed to web tier

4. **Web Tier**
   - Research available implementations
   - Verify license compatibility
   - Acquire via `web_fetch` + `skill_builder`

5. **Integration**
   - Harmonize with existing skills
   - Run Council approval if non-trivial
   - Promote to registry on success

```

### `/ciel-improve` — Self-Improvement

Guides through growth signal to improvement:

```markdown

# Self-Improvement Workflow

## Trigger

Growth signal detected from interaction outcome.

## Steps

1. **Signal Analysis**
   - Review trajectory for lessons
   - Identify pattern for abstraction
   - Score outcome vs. expectation

2. **Proposal Generation**
   - Draft improvement (config, prompt, or skill)
   - Estimate impact and risk
   - Prepare diff for review

3. **Council Gate**
   - Present to Council if non-trivial
   - Await majority approval
   - Handle Safety vetoes

4. **Apply**
   - Execute improvement
   - Git commit with message
   - Update CHANGELOG

5. **Observe**
   - Monitor subsequent interactions
   - Compare pre/post metrics
   - Rollback if regression detected

```

## Workflow Discovery

Windsurf discovers workflows from:

1. `.windsurf/workflows/` (current workspace)
2. `.windsurf/workflows/` (git root and parents)
3. `~/.codeium/windsurf/workflows/` (global)

Ciel installs its workflows to `~/.codeium/windsurf/workflows/ciel-*`.

## Creating Custom Workflows

Users can create project-specific workflows in `.windsurf/workflows/`:

```markdown
---
name: deploy-to-staging
description: Deploys the current branch to staging environment
---

# Deploy to Staging

## Pre-flight

- [ ] All tests passing
- [ ] Environment variables configured
- [ ] Staging database accessible

## Steps

1. Build production bundle
2. Run database migrations
3. Deploy to staging
4. Run smoke tests
5. Notify team

## Rollback

If step 4 fails: `git revert HEAD` and redeploy.
```

## Workflow vs Skill Decision Tree

Use a **Workflow** when:

- Steps must be followed in sequence
- User must confirm each phase
- Process is procedural (deploy, review)

Use a **Skill** when:

- Cascade should decide when to invoke
- Context + capability bundle needed
- Operation is reactive (route, analyze)

## Integration with Ciel

Ciel's Master Router can:

- Detect workflow invocation (`/` prefix)
- Delegate to workflow instead of skill
- Log workflow usage in `activity.log`

Future enhancement: Auto-convert frequently-used skill patterns to workflow recommendations.
