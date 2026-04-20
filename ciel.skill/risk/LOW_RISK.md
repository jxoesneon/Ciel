# LOW_RISK

Low-risk policy.

## Rule

Ciel acts autonomously. No judge. No Council. Log only.

## Examples (typical)

- Read files.
- List directories.
- Search / grep.
- Print computed results.
- Write to project workspace files that will be reviewed by the user before commit.
- Local `git add`, `git status`, `git diff`.

## Boundaries

Low-risk stops at:

- anything touching `.env*`, `secrets/`, keychain,
- anything writing outside the project root or `~/.ciel/`,
- any network call with side effects,
- any install of a system package.

Border-crossing auto-upgrades to `mid` and invokes the LLM judge.

## Logging

Every low-risk op still writes an `activity.log` entry with structured envelope. Transparent by design.

## Override

Cannot be overridden downward — low risk is already the floor. Can be overridden upward per-project via `configuration/local/escalation.config.md`.
