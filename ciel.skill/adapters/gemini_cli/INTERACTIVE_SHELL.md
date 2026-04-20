# INTERACTIVE_SHELL — Gemini CLI

Gemini CLI supports interactive shell tool calling — `vim`, `rebase -i`, REPLs, TUIs, etc. Ciel routes here for operations that genuinely require interactivity.

## When Ciel Enters Interactive Shell

- Rebase/merge conflict resolution with an editor open.
- `gh repo create` interactive wizard when user prefers the prompt flow.
- Language REPL exploration when `seed_skills/code_analysis/SKILL.md` needs live evaluation.
- Database CLI when a transaction is in progress and cannot be scripted.

## Hand-off

Ciel captures a pre-shell snapshot (git status, open files), then yields. Post-shell:

- re-capture state,
- diff,
- score outcome,
- log.

## Safety

Interactive shell sessions are treated as *elevated* — a persistent terminal with unpredictable effects. Safety member requires:

- operation categorized mid-risk at minimum,
- `allowedTools`-style restriction to a specific command,
- time-boxed session (default 10 minutes).

## Alternatives

If the same result is achievable via scriptable commands (e.g., `git rebase` with `--exec`), Ciel prefers the scripted path for reproducibility and improvement-loop learning.
