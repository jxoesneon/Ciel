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
    - pattern: "^orchestrate$"
      skill: ciel
      confidence: 0.95
      type: alias
    
  # Functional triggers - what the skill does
  functional:
    - pattern: "(route|orchestrate|coordinate|manage).*(skill|agent|task)"
      skill: ciel
      confidence: 0.9
      type: capability
      examples: ["route this to a skill", "orchestrate my agents"]
    
  # Domain triggers - subject area
  domain:
    - pattern: "(skill|capability|tool).*(find|search|discover|get|acquire)"
      skill: ciel
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
