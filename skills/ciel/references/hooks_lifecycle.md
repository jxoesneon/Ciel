# Lifecycle Hooks Architecture

## PreToolUse Hook
Triggered immediately before any tool call is dispatched.
Responsibilities:
- Inspect tool parameters against path traversal and safety boundaries.
- Classify risk tier (Low, Mid/High, Critical).
- Intercept and block unauthorized destructive actions.

## PostToolUse Hook
Triggered immediately after a tool call returns successfully.
Responsibilities:
- Verify returned output format and integrity.
- Log execution metrics to activity logs.
- Update working context in MemPalace partition.

## PostToolUseFailure Hook
Triggered upon tool execution error.
Responsibilities:
- Extract exact error traceback and failure class.
- Structure automated recovery hypothesis.
- Record failure pattern in memory graph.
