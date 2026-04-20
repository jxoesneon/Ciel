# PERMISSIONS — Claude Code

Claude Code's `allowedTools` / `deny` rules are Ciel's front-line containment. Ciel *reads* the host's permission config at init and shapes her behaviour accordingly; she does not silently *loosen* it.

## Read at Init

Parsed from `.claude/settings.json` + `~/.claude/settings.json`:

```json
{
  "permissions": {
    "allow": ["Read", "Edit", "Bash(npm:*)", "WebFetch(domain:github.com)"],
    "deny":  ["Bash(rm:*)", "WebFetch(domain:*.internal)"]
  }
}
```

Ciel caches the parsed policy and consults it in `PreToolUse` before she even runs her own risk classifier — host deny is absolute.

## Augmentation Rules

Ciel may **propose** adding a permission rule when:

- a Council-approved new skill requires a specific tool/domain, and
- the rule is narrowly scoped (no `Bash(*)` blanket).

Proposals are surfaced to the user via escalation — Ciel never writes to `settings.json` silently. This is a Constitutional invariant.

## Deny Evasion — Forbidden

Ciel cannot use a differently-named tool to achieve what a `deny` rule forbids. If a user denies `Bash(rm:*)`, Ciel also refuses Python `os.remove` with paths that match the deny semantics. Safety member enforces.

## Permission Request Flow

1. Ciel detects required tool not currently allowed.
2. She routes through an alternative if one exists and is allowed.
3. Otherwise she escalates with:
   - the tool requested,
   - the reason,
   - a specific proposed rule,
   - a diff preview of `settings.json`.
4. User accepts / rejects.

## Audit

Every permission-touching event is in `activity.log`:

```json
{ "kind": "permission", "op": "check", "tool": "Bash", "args": "git push", "decision": "allow" }
```
