# AUTO-ACTIVATION — Cross-Platform Skill Auto-Run Guide

Ciel implements auto-activation mechanisms on all three supported runtimes, using each platform's native hook and context systems.

## Platform Comparison

| Mechanism | Claude Code | Gemini CLI | Windsurf | Devin for Terminal |
| --- | --- | --- | --- | --- |
| **Native Auto-Discovery** | Description-based (unreliable) | `activate_skill` tool (best) | Description-based | Description-based |
| **Hook-Based Activation** | `UserPromptSubmit` | `BeforeModel` | `pre_user_prompt` | `UserPromptSubmit` |
| **Session Bootstrap** | `SessionStart` | `SessionStart` | N/A | `SessionStart` |
| **Context Injection** | `CLAUDE.md` | `GEMINI.md` | `.windsurf/rules` | `AGENTS.md` / `CLAUDE.md` |
| **Manual Commands** | Slash commands | Commands | Workflows | Slash commands (`/ciel`) |
| **Pre-Tool Safety** | `PreToolUse` | `BeforeTool` | `pre_*` events | `PreToolUse` |
| **Post-Tool Scoring** | `PostToolUse` | `AfterTool` | `post_*` events | `PostToolUse` |

## Recommended Auto-Activation Strategy

### Tier 1: Session Bootstrap (All Platforms)

Inject Ciel identity at session start:

- **Claude Code**: `SessionStart` hook
- **Gemini CLI**: `SessionStart` hook
- **Windsurf**: `.windsurf/rules` (persistent context)
- **Devin for Terminal**: `SessionStart` hook in `~/.config/devin/config.json`

### Tier 2: Trigger Phrase Detection (All Platforms)

Force skill activation on trigger words:

- **Claude Code**: `UserPromptSubmit` hook with regex
- **Gemini CLI**: `BeforeModel` hook with context injection
- **Windsurf**: `pre_user_prompt` hook
- **Devin for Terminal**: `UserPromptSubmit` hook in `.devin/hooks.v1.json`

**Common Trigger Patterns:**

```regex
ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council
```

### Tier 3: Native Skill Discovery (Gemini Best)

Let the runtime match skill descriptions:

- **Gemini**: Excellent native support via `activate_skill`
- **Claude/Windsurf**: Supplement with strong SKILL.md descriptions

### Tier 4: Manual Override (All Platforms)

Explicit user invocation:

- **Claude Code**: `/ciel` slash command
- **Gemini CLI**: `ciel` command
- **Windsurf**: `@ciel` mention or `/ciel` workflow
- **Devin for Terminal**: `/ciel` slash command (via `.devin/skills/ciel/SKILL.md`)

## Implementation Checklist

### Claude Code

- [ ] `SessionStart` hook injects Ciel context
- [ ] `UserPromptSubmit` hook detects triggers
- [ ] `PreToolUse` hook for safety gating
- [ ] `CLAUDE.md` with Ciel identity block
- [ ] `.claude/commands/ciel.md` slash command

### Gemini CLI

- [ ] `SessionStart` hook for bootstrap
- [ ] `BeforeModel` hook for trigger detection
- [ ] `BeforeTool` hook for safety
- [ ] `GEMINI.md` with triggers documented
- [ ] Strong SKILL.md description for auto-discovery
- [ ] `.gemini/commands/ciel.md` command

### Windsurf

- [ ] `.windsurf/rules` with Ciel context
- [ ] `pre_user_prompt` hook for triggers
- [ ] `pre_write_code`/`pre_run_command` for safety
- [ ] `post_cascade_response` for logging
- [ ] `.windsurf/workflows/ciel-*.md` workflows
- [ ] Strong SKILL.md description

### Devin for Terminal

- [ ] `SessionStart` hook in `~/.config/devin/config.json` for bootstrap
- [ ] `UserPromptSubmit` hook in `.devin/hooks.v1.json` for triggers
- [ ] `PreToolUse` hook for safety gating
- [ ] `PostToolUse` hook for outcome scoring
- [ ] `AGENTS.md` or `CLAUDE.md` with Ciel identity block
- [ ] `.devin/skills/ciel/SKILL.md` project-scoped skill
- [ ] `.devin/config.json` with scope-based permissions
- [ ] Strong SKILL.md description for auto-discovery

## Cross-Platform Hook Equivalents

| Ciel Function | Claude Code | Gemini CLI | Windsurf | Devin for Terminal |
| --- | --- | --- | --- | --- |
| Session bootstrap | `SessionStart` | `SessionStart` | N/A (use rules) | `SessionStart` |
| Trigger detection | `UserPromptSubmit` | `BeforeModel` | `pre_user_prompt` | `UserPromptSubmit` |
| Pre-tool safety | `PreToolUse` | `BeforeTool` | `pre_*` | `PreToolUse` |
| Post-tool logging | `PostToolUse` | `AfterTool` | `post_*` | `PostToolUse` |
| Subagent tracking | `SubagentStart/Stop` | `Before/AfterAgent` | N/A | N/A (root only) |
| Compression | `PreCompact` | `PreCompress` | N/A | `PostCompaction` |

## Key Insight

**Gemini CLI has the best native auto-activation** via the `activate_skill` tool. Claude Code and Windsurf require hook-based trigger detection for reliable activation.

**Universal Pattern:**

1. Session hook injects identity
2. Prompt hook detects triggers and forces activation
3. Tool hooks provide safety gating
4. Post-tool hooks enable self-improvement

See per-platform docs:

- `adapters/claude_code/HOOKS.md`
- `adapters/gemini_cli/HOOKS.md`
- `adapters/windsurf/HOOKS.md`
- `adapters/devin/HOOKS.md`
