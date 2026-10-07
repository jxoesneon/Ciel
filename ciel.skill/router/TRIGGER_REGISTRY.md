# TRIGGER_REGISTRY — Activation Trigger System

Central registry for all skill activation triggers. Supports pattern matching, confidence scoring, and dynamic trigger generation.

## Registry Format

```yaml
registry_version: "1.0.0"
last_updated: "2025-01-20T10:00:00Z"

triggers:
  # Direct triggers - skill name/alias
  direct:

    - pattern: "^ciel$"

      skill: ciel
      confidence: 1.0
      type: exact_name

    - pattern: "^(taste|taste.?skill|anti.?slop)$"
      skill: taste
      confidence: 1.0
      type: exact_name

    - pattern: "(taste|anti.?slop|clean.?ui|design.?dials|redesign.?ui)"
      skill: taste
      confidence: 0.95
      type: alias

    - pattern: "^(impeccable)$"
      skill: impeccable
      confidence: 1.0
      type: exact_name

    - pattern: "^(frontend.?design)$"
      skill: frontend-design
      confidence: 1.0
      type: exact_name

    - pattern: "^(ui.?ux.?pro.?max|uipro)$"
      skill: ui-ux-pro-max
      confidence: 1.0
      type: exact_name

    - pattern: "^(animate|animate.?expo|animation.?vocabulary|apple.?design|ask.?sonner|break.?ui|emil.?design.?eng|find.?animation.?opportunities|improve.?animations|mobile.?native|pick.?ui.?library|prototype|review.?animations|write.?swift)$"
      skill: emil-design-eng
      confidence: 1.0
      type: exact_name

    - pattern: "^(e2e|tester.?army|tester.?army.?e2e)$"
      skill: e2e
      confidence: 1.0
      type: exact_name

  # Functional triggers - what the skill does
  functional:
    - pattern: "(audit|redesign|improve|enhance).*(frontend|ui|css|landing|portfolio)"
      skill: taste
      confidence: 0.9
      type: capability
      examples: ["redesign this frontend", "audit this page for slop", "set design dials"]

    - pattern: "(route|orchestrate|coordinate|manage).*(skill|agent|task)"
      skill: ciel
      confidence: 0.9
      type: capability
      examples: ["route this to a skill", "orchestrate my agents"]

    - pattern: "(write|scaffold|debug|run).*(agentic|natural.?language|plain.?english).*(e2e|end.?to.?end|browser).*(test|spec)|bug.?bash|e2e\.config|agent\.(act|assert|waitFor|extract)"
      skill: e2e
      confidence: 0.9
      type: capability
      examples: ["write agentic e2e tests", "bug bash this app", "debug a failing e2e run", "set up e2e.config.ts"]

  # Domain triggers - subject area
  domain:
    - pattern: "(frontend|design|typography|anti.?slop|tailwind|ui.?taste)"
      skill: taste
      confidence: 0.85
      type: intent

    - pattern: "(design.?audit|design.?critique|polish.?ui|harden.?ui|design.?review)"
      skill: impeccable
      confidence: 0.85
      type: intent

    - pattern: "(color.?palette|font.?pairing|ux.?guideline|chart.?type|ui.?style|design.?intelligence)"
      skill: ui-ux-pro-max
      confidence: 0.85
      type: intent

    - pattern: "(animate|animation|motion|transition|micro.?interaction|spring|gesture|easing)"
      skill: emil-design-eng
      confidence: 0.85
      type: intent

    - pattern: "(skill|capability|tool).*(find|search|discover|get|acquire)"
      skill: ciel
      confidence: 0.85
      type: intent

    - pattern: "(agentic|ai.?driven).*(e2e|end.?to.?end|ui).*(test|qa)|replay.?cache|e2e.*(init|run|explore|mcp|report)"
      skill: e2e
      confidence: 0.85
      type: intent

  # Intent triggers - user goal
  intent:

    - pattern: "(self.?improve|evolve|upgrade|enhance).*(skill|system|yourself)"

      skill: ciel
      confidence: 0.9
      type: meta_request

  # Composite triggers - multi-skill patterns
  composite:

    - patterns: ["find.*skill", "then.*use.*it"]

      workflow: [discover, execute]
      confidence: 0.8

# Compiled patterns for performance

compiled:
  fast_path: "/^(ciel|orchestrate|route this|you there|hey you)$/i"
  reasoning_path: "/(skill.*(find|acquire|search)|orchestrate|self.?improve|are you|can you|will you|do you)/i"
```

## Trigger Categories

| Category | Description | Example | Confidence Range |
| --- | --- | --- | --- |
| **direct** | Exact skill names/aliases | "ciel", "filesystem" | 0.95-1.0 |
| **functional** | Capability description | "list files", "search web" | 0.7-0.9 |
| **domain** | Subject matter | "docker", "git", "markdown" | 0.6-0.8 |
| **intent** | User goal | "can you", "are you", "I need to" + domain | 0.5-0.7 |
| **contextual** | Project context | "this codebase", "current project" | 0.4-0.6 |
| **composite** | Multi-step workflows | "find and fix" | 0.6-0.8 |

## Confidence Scoring

```yaml
scoring_factors:
  pattern_match: 1.0        # Base match score
  word_order: 0.1           # Bonus for exact phrase order
  context_match: 0.15       # Project context alignment
  history_bonus: 0.1        # Recent successful use
  frequency_penalty: -0.05  # Overused generic patterns
```

## Dynamic Trigger Generation

When new skill added, triggers auto-extracted from:

- `SKILL.md` frontmatter `triggers:` list
- `SKILL.md` description keywords
- File name and directory conventions
- Example usage patterns

See `TRIGGER_GENERATOR.md` for pipeline details.

## Registry Maintenance

- **Auto-update**: On skill install/remove
- **Compaction**: Monthly regex optimization
- **Conflict resolution**: Manual for <0.1 confidence gaps
- **Audit log**: All trigger changes logged
