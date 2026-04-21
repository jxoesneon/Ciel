# TRIGGER_GENERATOR — Dynamic Trigger Pipeline

Generates and manages activation triggers when skills are added, updated, or removed.

## Pipeline Stages

```
Skill Added
    │
    ▼
┌─────────────────┐
│ Stage 1: EXTRACT│ ← Parse SKILL.md, extract all trigger sources
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Stage 2: GENERATE│ ← Create additional triggers from description
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Stage 3: SCORE  │ ← Assign confidence based on specificity
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Stage 4: VALIDATE│ ← Check conflicts, test examples
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Stage 5: REGISTER│ ← Add to TRIGGER_REGISTRY, compile patterns
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Stage 6: DEPLOY │ ← Update runtime trigger caches
└─────────────────┘
```

## Stage 1: Extract

Sources for trigger extraction:

```yaml
extract_sources:
  frontmatter:
    path: "SKILL.md"
    fields:
      - name           # Direct trigger
      - triggers       # Explicit list
      - description    # Keyword extraction
      - keywords       # Domain tags
      
  filename:
    pattern: "{name}.skill"
    transforms:
      - lowercase
      - strip_version
      - split_camelcase
      
  directory_structure:
    scan: ["*.md", "docs/*.md"]
    extract: headers, command names, API endpoints
    
  code_signatures:
    scan: ["*.{js,py,ts,go,rs}"]
    extract: function names, CLI args, tool definitions
```

## Stage 2: Generate

Generates additional triggers from extracted data:

```yaml
generation_rules:
  # From skill name
  name_variants:
    input: "web_search"
    outputs: ["web search", "websearch", "search web", "websearcher"]
    
  # From description
  description_keywords:
    input: "Fetches web pages and extracts clean markdown content"
    outputs:
      - pattern: "(fetch|get|retrieve).*(web|page|url)"
      - pattern: "(extract|parse).*(markdown|content|text)"
      - pattern: "web.*scraping"
      
  # Functional synonyms
  capability_synonyms:
    "search": ["find", "lookup", "query", "discover", "locate"]
    "read": ["fetch", "get", "load", "retrieve"]
    "write": ["save", "store", "create", "update"]
    
  # Domain expansion
  domain_hierarchy:
    "web_search": ["web", "internet", "online", "url", "http"]
    "filesystem": ["file", "directory", "folder", "path", "fs"]
```

## Stage 3: Score

Confidence scoring algorithm:

```yaml
scoring:
  base_scores:
    frontmatter_trigger: 0.95
    name_exact: 0.90
    name_variant: 0.80
    description_keyword: 0.70
    generated_synonym: 0.60
    domain_hierarchy: 0.50
    
  modifiers:
    length_bonus: "+(0.02 * word_count)"  # Longer = more specific
    ambiguity_penalty: "-(0.1 * conflict_count)"
    history_bonus: "+(0.05 * success_rate)"
```

## Stage 4: Validate

```yaml
validation_checks:
  conflicts:
    - Check pattern overlap with existing triggers
    - Flag if confidence difference < 0.1
    - Report to Council if conflict detected
    
  test_cases:
    - Run all examples from SKILL.md
    - Run generated examples
    - Verify top-3 match accuracy > 0.9
    
  performance:
    - Compile regex time < 10ms
    - Match time < 1ms per pattern
```

## Stage 5: Register

```yaml
registration:
  atomic_update: true
  transaction:
    - Backup current registry
    - Add new triggers
    - Compile patterns
    - Verify compilation
    - Commit or rollback
    
  metadata:
    skill_id: "<skill_name>"
    version: "<skill_version>"
    generated_at: "<timestamp>"
    generator_version: "1.0.0"
```

## Stage 6: Deploy

```yaml
deployment:
  runtime_sync:
    - Update in-memory trigger cache
    - Notify active sessions (if applicable)
    - Write to runtime-specific location
    
  claude_code:
    path: ".claude/triggers/ciel_triggers.json"
    
  gemini_cli:
    path: ".gemini/triggers/ciel_triggers.json"
    
  windsurf:
    inject_into: ".windsurf/rules"
```

## Usage

### Manual Trigger Generation

```bash
# Generate triggers for specific skill
./scripts/generate-triggers.sh ~/.ciel/skills/web_search/

# Regenerate all triggers
./scripts/regenerate-all-triggers.sh

# Test trigger matching
./scripts/test-triggers.sh "find me a web page"
```

### Automatic Generation

Hook into skill installation:

```yaml
# In acquisition pipeline
post_install:
  - trigger: run TRIGGER_GENERATOR
  - validate: test 10 example prompts
  - deploy: update all runtimes
```

## Trigger Templates

### Common Patterns

```yaml
templates:
  # CLI tool pattern
  cli_tool:
    patterns:
      - "{name}"
      - "run {name}"
      - "{name} command"
      - "use {name}"
    examples: ["git status", "run docker", "npm install"]
    
  # File operation pattern
  file_operation:
    patterns:
      - "(read|write|list|delete).*{domain}"
      - "{domain}.*(file|directory|folder)"
    examples: ["list files", "read directory", "delete folder"]
    
  # Search pattern
  search_tool:
    patterns:
      - "search.*{domain}"
      - "find.*{domain}"
      - "lookup.*{domain}"
      - "query.*{domain}"
    examples: ["search web", "find files", "lookup command"]
    
  # Analysis pattern
  analyzer:
    patterns:
      - "(analyze|check|audit|scan|review).*{domain}"
      - "{domain}.*(analysis|report|check)"
    examples: ["analyze code", "audit dependencies", "security scan"]
```

## Conflict Resolution

When two skills have overlapping triggers:

```yaml
resolution_strategy:
  1_keep_higher_confidence: true
  2_context_disambiguation: "Use project context to prefer"
  3_council_override: "Manual resolution for persistent conflicts"
  4_composite_prompt: "Ask user to clarify"
```

## Related

- `TRIGGER_REGISTRY.md` — Central trigger storage
- `FAST_PATH.md` — Trigger matching in routing
- `acquisition/HARMONIZATION.md` — Skill standardization
