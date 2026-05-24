---
name: skill-discovery
version: 1.0.0
description: Discover and install capabilities from Ciel's skill ecosystem. Find tools for any task, evaluate options, and integrate into your workflow.
format: skill/1.0
author: Ciel Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "skill discovery"
  - "find a skill"
  - "discover skill"
  - "what skill can"
  - "install skill"
  - "skill for"
  - "how do I"
  - "can you help me with"
entrypoint: SKILL.md
---

# Skill Discovery

Navigate Ciel's vast capability landscape. Find the right tool for any job, evaluate your options, and integrate seamlessly into your workflow.

## When to Activate

Invoke skill-discovery when:

- You need a capability and wonder "does Ciel have a skill for this?"
- You're facing a task and don't know which skill to use
- You want to explore available tools in a domain
- You're extending your project's Ciel integration
- You ask questions like:
  - "How do I [do X]?"
  - "Find a skill for [task]"
  - "What can help me with [problem]?"
  - "Is there a tool that [capability]?"

## Discovery Methods

### 1. Intent-Based Search

Describe what you want to accomplish:

> "I need to analyze this Docker container's security"

Skill-discovery will route to: `security-review`, `docker-patterns`

### 2. Domain Exploration

Browse by category:

| Domain | Skills |
|--------|--------|
| **Testing** | python-testing, e2e-testing, tdd-workflow, benchmark |
| **Security** | security-review, security-scan |
| **DevOps** | docker-patterns, deployment-patterns, git-workflow |
| **Data** | postgres-patterns, database-migrations |
| **Research** | deep-research, exa-search |

### 3. Capability Matching

Match your need to skill capabilities:

- **"I need to test code"** → Testing framework skills
- **"I need to deploy"** → deployment-patterns, docker-patterns
- **"I need to debug"** → systematic-debugging
- **"I need to write docs"** → article-writing, content-engine

## Evaluation Criteria

When multiple skills match, evaluate by:

| Criterion | Questions to Ask |
|-----------|------------------|
| **Scope** | Does it solve my exact problem or a broader one? |
| **Maturity** | Is this a core skill or experimental? |
| **Integration** | Does it work with my runtime? |
| **Overlap** | Does it duplicate existing skills? |
| **Quality** | Well-documented with clear examples? |

## Installation

Once discovered, install via your runtime:

```bash
# Skills are auto-discovered by Ciel's router
# Just invoke by trigger phrase
```

## Discovery Workflow

```
User Need
    │
    ▼
┌─────────────────────┐
│ Describe the task   │  "I need to analyze API security"
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Match to domain     │  → Security domain
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Find candidates     │  → security-review, security-scan
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Evaluate & select   │  → security-review (broader scope)
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Invoke skill        │  "security-review"
└─────────────────────┘
```

## Tips for Effective Discovery

### Be Specific

| ❌ Vague | ✅ Specific |
|----------|-------------|
| "Help me with code" | "Help me write Python tests" |
| "Fix my app" | "Debug a Django deployment issue" |
| "Do research" | "Search for recent AI papers on X" |

### Mention Constraints

- **"for Python"** → Routes to python-testing, not generic
- **"for production"** → Routes to deployment-patterns, docker-patterns
- **"for beginners"** → Routes to onboarding skills

### Explore Related Skills

After discovering one skill, explore its ecosystem:

- `python-testing` → Related: `tdd-workflow`, `benchmark`, `e2e-testing`
- `security-review` → Related: `security-scan`, `systematic-debugging`
- `backend-patterns` → Related: `api-design`, `database-migrations`, `hexagonal-architecture`

## Common Discovery Patterns

### Pattern 1: Task → Skill Mapping

| Task | Discovered Skill |
|------|------------------|
| Write tests for Django app | python-testing + django-testing |
| Deploy to Kubernetes | deployment-patterns + docker-patterns |
| Debug failing CI | systematic-debugging + git-workflow |
| Research competitors | deep-research + exa-search |
| Write blog post | article-writing + content-engine |

### Pattern 2: Domain Exploration

Start broad, narrow down:

```
"I need help with deployment"
    → deployment-patterns (general)
        → docker-patterns (containerization)
            → backend-patterns (app structure)
                → api-design (interface design)
```

### Pattern 3: Skill Chaining

Multiple skills for complex tasks:

```
"Build a secure API"
    → api-design (architecture)
    → backend-patterns (implementation)
    → security-review (audit)
    → python-testing (validation)
```

## Skill Registry

Ciel maintains a living registry at:

- **Router**: `router/ROUTE_REGISTRY.md`
- **Triggers**: `router/TRIGGER_REGISTRY.md`
- **Backlog**: `config/adaptation_queue.csv`

## Provenance

This skill was inspired by the ECC ecosystem but written fresh for Ciel. See `PROVENANCE.md`.

## Related Skills

- `find-skills` — The ECC skill that inspired this adaptation
- `orchestration` — For complex multi-skill workflows
- `council` — For ambiguous skill selection decisions

## Version History

- 1.0.0 — Initial adaptation from ECC inspiration
