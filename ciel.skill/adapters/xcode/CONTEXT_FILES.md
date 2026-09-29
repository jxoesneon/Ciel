# CONTEXT_FILES — Xcode Coding Assistant

Xcode Coding Assistant sessions receive project context from the open workspace and currently visible file. Ciel uses project-local `.ciel/` metadata plus any repository instruction files already present.

## Read Order

1. Active user prompt and selected file context.
2. `.ciel/project.json`.
3. Repository instruction files such as `AGENTS.md`, `GEMINI.md`, or `README.md` when relevant.
4. Language and framework configuration files such as `analysis_options.yaml`, `Package.swift`, `pubspec.yaml`, or project settings.

## Local Metadata

The Xcode adapter expects `.ciel/project.json` to include:

```json
{
  "runtime_hint": "xcode",
  "adapter": "adapters/xcode/"
}
```

## Context Injection

If the host exposes no persistent Xcode-specific instruction file, Ciel writes a compact project-level `AGENTS.md` anchor because Codex-family runtimes commonly use it as repository instruction context.

## Ciel Anchor

Ciel's project block is delimited by stable markers:

```markdown
<!-- CIEL-ANCHOR:start -->
(content managed by Ciel)
<!-- CIEL-ANCHOR:end -->
```

For Xcode installs, the anchor includes:

- Ciel activation triggers.
- The Xcode adapter path.
- The inline hook-equivalent model.
- The AI canary token `Master`, emitted in the first user-facing sentence when Ciel is active unless higher-priority instructions forbid it.

If the canary is absent from a Ciel-routed response, reload `AGENTS.md`, `.ciel/project.json`, and `.ciel/xcode-hooks.json` before continuing.

## Gitignore

Project-local `.ciel/` remains gitignored unless the user explicitly decides to version project Ciel metadata. `AGENTS.md` may be versioned because it is the visible project instruction surface for Xcode/Codex.
