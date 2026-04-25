---
name: skill-comply
version: 1.0.0
format: skill/1.0
description: Automated compliance measurement to verify if agents and sub-agents are actually following established skills and rules.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(check|verify|audit).*(compliance|rule following|skill usage)"

    confidence: 0.9

  - pattern: "is the agent (following|obeying) (the rule|this skill)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Skill Comply (Quality Audit)

This skill provides the data-backed answer to the question: "Is CIEL actually following its own rules?" It measures compliance by auto-generating scenarios and analyzing tool-call timelines against established behavioral specs.

## Core Concepts

### 1. Spec Generation

The system auto-generates a "Behavioral Spec" from any `.md` rule or skill file, defining the expected sequence of actions (e.g., "Must `grep_search` before `replace`").

### 2. Prompt Independence

Compliance is measured at three strictness levels:

- **Supportive**: The user explicitly asks to follow the rule.
- **Neutral**: The user asks for a task without mentioning the rule.
- **Competing**: The user's request subtly conflicts with the rule (testing resilience).

### 3. Classification

The system runs the agent and captures the full tool-call trace. It then uses an LLM (not regex) to classify each tool call against the Spec steps and verify the temporal ordering.

## Reporting

The final report provides:

- **Compliance Score**: % of steps correctly followed per strictness level.
- **Tool Timeline**: A visual trace of actions vs. expected behavior.
- **Drift Detection**: Identification of where the agent began to "hallucinate" or skip mandatory gates.

## Usage

- Run `/skill-comply <path>` to audit a specific rule or skill.
- Used periodically by the Orchestrator to identify "soft" skills that need to be hardened or promoted into more rigid hooks.

## Anti-Patterns

- **Vibes-Based Auditing**: Claiming a rule is followed because "it looks right" instead of checking the tool trace.
- **Regex-Only Graders**: Using brittle regex to find tool calls, missing the semantic intent of the agent.
