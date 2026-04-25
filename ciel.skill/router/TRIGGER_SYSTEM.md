# TRIGGER_SYSTEM — Comprehensive Activation System

Ciel's trigger system provides intelligent skill activation through pattern matching, confidence scoring, and dynamic trigger generation.

## Architecture Overview

```text
User Request
    │
    ▼
┌─────────────────────┐
│ TRIGGER_MATCH       │ ← Fast pattern matching against compiled triggers
│ (direct/functional/   │   Confidence threshold: 0.85
│  domain/intent)     │
└────────┬────────────┘
         │ Hit? ───────────────┐
         │                      │
         ▼                      ▼
    ┌─────────┐         ┌─────────────┐
    │ ROUTE   │         │ FAST_PATH   │
    │ EXECUTE │         │ (fallback)  │
    └─────────┘         └──────┬──────┘
                               │
                               ▼
                        ┌─────────────┐
                        │ REASONING   │
                        │ PATH        │
                        └─────────────┘
```

## System Components

| Component | Purpose | Location |
| --- | --- | --- |
| `TRIGGER_REGISTRY.md` | Central trigger storage with categories | `router/` |
| `TRIGGER_GENERATOR.md` | Dynamic trigger generation pipeline | `router/` |
| `ROUTE_REGISTRY.md` | Route entries with trigger references | `router/` |
| `LOCAL_DISCOVERY.md` | Foreign skill discovery & ingestion | `acquisition/` |
| `ADAPTATION_MAPPINGS.md` | Format conversion reference | `acquisition/` |

## Trigger Categories

### 1. Direct Triggers (Confidence: 0.95-1.0)

Exact skill names and aliases.

```yaml
triggers:

  - pattern: "^ciel$"           # Exact match

    skill: ciel
    confidence: 1.0

  - pattern: "^orchestrate$"     # Alias

    skill: ciel
    confidence: 0.95
```

### 2. Functional Triggers (Confidence: 0.7-0.9)

Capability-based patterns describing what the skill does.

```yaml
triggers:

  - pattern: "(route|orchestrate).*(skill|agent)"

    skill: ciel
    confidence: 0.9
    examples:

      - "route this to a skill"
      - "orchestrate my agents"

```

### 3. Domain Triggers (Confidence: 0.6-0.8)

Subject matter patterns.

```yaml
triggers:

  - pattern: "(skill|tool).*(find|search|acquire)"

    skill: ciel
    confidence: 0.85
    context: "when seeking capabilities"
```

### 4. Intent Triggers (Confidence: 0.5-0.7)

User goal patterns.

```yaml
triggers:

  - pattern: "(self.?improve|evolve|upgrade).*(skill|system)"

    skill: ciel
    confidence: 0.9
    meta_request: true
```

### 5. Composite Triggers (Confidence: 0.6-0.8)

Multi-step workflow patterns.

```yaml
triggers:

  - patterns:
      - "find.*skill"
      - "then.*use.*it"

    workflow: [discover, execute]
    confidence: 0.8
```

## Confidence Scoring

```yaml
scoring_algorithm:
  base_score: 1.0
  
  modifiers:
    pattern_specificity: +0.1    # Longer patterns score higher
    word_order_match: +0.05       # Exact phrase order bonus
    context_alignment: +0.15      # Project context match
    history_bonus: +0.1          # Recent successful activation
    frequency_penalty: -0.05      # Overused generic patterns
    ambiguity_penalty: -0.15      # Multiple skill matches
    
  floor_thresholds:
    fast_path: 0.85
    reasoning_path: 0.60
    acquisition_path: 0.0         # Always attempt
```

## Dynamic Trigger Generation Pipeline

When a skill is added, the pipeline automatically extracts and generates triggers:

### Stage 1: Extract

Sources for trigger extraction:

- `SKILL.md` frontmatter `triggers:`
- `SKILL.md` description keywords
- Skill name and variants
- Command names (from `commands/`)
- Tool/function signatures
- Example usage patterns

### Stage 2: Generate

Additional triggers generated via:

- Synonym expansion (search → find, lookup, query)
- Verb conjugation (analyze → analyzing, analyzed)
- Preposition variants ("search for" vs "search")
- Domain hierarchy expansion

### Stage 3: Score

Confidence assignment based on:

- Source reliability (frontmatter > generated)
- Pattern specificity (exact > regex)
- Historical performance (if available)

### Stage 4: Validate

- Conflict detection with existing triggers
- Test case validation
- Performance benchmarking

### Stage 5: Register

Atomic update of TRIGGER_REGISTRY with compiled patterns.

### Stage 6: Deploy

Runtime-specific trigger cache updates.

## Local Skill Discovery & Ingestion

### Discovery Scan Paths

```yaml
discovery_paths:
  claude_code:

    - ~/.claude/skills/*/
    - ./.claude/skills/*/
    
  gemini_cli:

    - ~/.gemini/skills/*/
    - ./.gemini/skills/*/
    
  windsurf:

    - ~/.windsurf/skills/*/
    - ./.windsurf/skills/*/
    - ~/.codeium/windsurf/skills/*/

```

### Format Detection & Adaptation

| Source Format | Detection | Action |
| --- | --- | --- |
| Ciel-native | `SKILL.md` with `format: skill/1.0` | Register directly |
| Claude Code | `CLAUDE.md` present | Adapt to Ciel format |
| Gemini CLI | `GEMINI.md` present | Adapt to Ciel format |
| Windsurf | `.windsurf/` directory | Often compatible, minor tweaks |
| Generic | `README.md` or `package.json` | Extract metadata, create stub |

### Backup Strategy

```yaml
backup:
  trigger: pre_adaptation
  format: "{skill_name}_{timestamp}.bkp.zip"
  location: "~/.ciel/.attic/foreign_skills/"
  manifest: "MANIFEST.bkp.json"
  retention: 3 backups per skill
```

## Usage Commands

### Discover Local Skills

```bash

# Discover all

./scripts/discover-local-skills.sh --scan-all

# Discover specific runtime

./scripts/discover-local-skills.sh --runtime claude_code

# Output to file

./scripts/discover-local-skills.sh --output discovered.json
```

### Ingest Skills

```bash

# Ingest single skill with adaptation

./scripts/ingest-skill.sh ~/.claude/skills/my-skill/ --adapt

# Batch ingest

./scripts/ingest-skill.sh ~/.claude/skills/*/ --adapt --backup

# Dry run (simulate only)

./scripts/ingest-skill.sh ~/.claude/skills/my-skill/ --dry-run
```

### Generate Triggers

```bash

# Generate for specific skill

./scripts/generate-triggers.sh ~/.ciel/skills/my-skill/

# Test trigger matching

./scripts/test-triggers.sh "find me a web search skill"
```

## Integration with Routing

```yaml
routing_flow:
  1_trigger_match:

    - Load compiled patterns from TRIGGER_REGISTRY
    - Match in order: direct > functional > domain > intent
    - Confidence >= 0.85: fast path execution
    
  2_registry_lookup:

    - Fallback to tag-based routing
    - For backwards compatibility
    
  3_reasoning_path:

    - LLM-driven composition
    - Can reference trigger patterns as hints
    
  4_acquisition:

    - Gap detection triggers new skill acquisition
    - Auto-generation of triggers on skill install

```

## Registry Updates on Skill Lifecycle

### On Skill Install

1. Extract triggers from SKILL.md
2. Generate additional triggers
3. Register in TRIGGER_REGISTRY
4. Compile patterns for fast matching
5. Update ROUTE_REGISTRY with trigger references

### On Skill Update

1. Detect changed triggers
2. Re-generate trigger set
3. Validate no regression in matching
4. Atomic registry update

### On Skill Remove

1. Remove all associated triggers
2. Archive for 30 days (in case of rollback)
3. Compact registry

## Conflict Resolution

When multiple skills match the same trigger:

```yaml
resolution_strategy:
  1_higher_confidence_wins: true
  2_context_disambiguation:

    - Use project language/framework context
    - Prefer skills matching current domain

  3_frequency_penalty:

    - Apply decay to overused generic skills

  4_explicit_escalation:

    - Ask user: "Did you mean skill A or skill B?"

  5_council_override:

    - Manual resolution for persistent conflicts

```

## Safety & Trust

```yaml
trigger_safety:
  foreign_skill_triggers:
    default_confidence: 0.6  # Lower than native
    
  sandbox_recommendations:

    - Test trigger matching before deploy
    - Monitor for false positives
    - Review user override patterns
    
  audit_logging:

    - All trigger changes logged
    - Match statistics per trigger
    - User override tracking

```

## Best Practices

### For Skill Authors

1. **Define explicit triggers** in SKILL.md frontmatter
2. **Include examples** for each trigger pattern
3. **Use specific patterns** over generic ones
4. **Test trigger matching** before publishing

### For Users

1. **Use natural language** — triggers match conversational patterns
2. **Be specific** — "search web for python docs" > "search"
3. **Check available triggers** via `/ciel triggers` (future feature)

### For Ciel

1. **Prefer specificity** — High-confidence triggers first
2. **Learn from overrides** — User corrections improve scoring
3. **Monitor drift** — Detect when triggers become stale
4. **Suggest improvements** — Propose new triggers on gap detection

## Future Enhancements

- **Semantic matching** — Beyond regex to embeddings-based similarity
- **Contextual triggers** — Time, project phase, recent activity aware
- **User personalization** — Learn individual user's phrasing preferences
- **A/B trigger testing** — Test new patterns against existing

## Related Documents

- `TRIGGER_REGISTRY.md` — Trigger storage format
- `TRIGGER_GENERATOR.md` — Generation pipeline
- `LOCAL_DISCOVERY.md` — Foreign skill discovery
- `ADAPTATION_MAPPINGS.md` — Format conversion
- `ROUTE_REGISTRY.md` — Route entries with triggers
- `ROUTER.md` — Routing integration
