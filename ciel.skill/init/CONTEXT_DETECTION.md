# CONTEXT_DETECTION

Detect project context at local init. Feeds calibration, rule extraction, and routing bias.

## Inputs

- project root (`cwd` by default).
- filesystem listing (top-level + one level deep).
- presence of common manifests.
- git state.

## Signals

| File / feature | Inference |
| --- | --- |
| `package.json` | Node.js; inspect `dependencies` for framework |
| `Cargo.toml` | Rust; parse `[package]` for crate type |
| `pyproject.toml`/`setup.py`/`requirements.txt` | Python |
| `go.mod` | Go |
| `pom.xml`/`build.gradle` | JVM |
| `Gemfile` | Ruby |
| `next.config.*`, `nuxt.config.*` | Next.js, Nuxt |
| `vite.config.*`, `webpack.config.*` | Build tool |
| `Dockerfile` | Containerized |
| `.github/workflows/` | GitHub Actions |
| `CLAUDE.md` / `GEMINI.md` | Existing host context file |
| `.claude/agents/` / `.gemini/agents/` | Existing agent definitions |
| `.editorconfig`, `.prettierrc`, `rustfmt.toml`, `pyproject.toml [tool.black]` | Formatting preferences |
| `LICENSE` | License family |
| `CODEOWNERS` | Team structure hint |

## Output

`<project>/.ciel/project.json`:

```json
{
  "id": "<partition_hash>",
  "root": "/abs/path",
  "language": "rust",
  "frameworks": ["axum", "tokio"],
  "build_tools": ["cargo"],
  "test_tools": ["cargo test"],
  "formatter": "rustfmt",
  "linter": "clippy",
  "ci": ["github-actions"],
  "license": "Apache-2.0",
  "monorepo": false,
  "runtime_hint": "claude-code",
  "detected_at": "..."
}
```

## Rule Extraction

Existing `CLAUDE.md` / `GEMINI.md` are parsed for human-stated rules which become entries in `configuration/local/rules.config.md`. Detected + user-declared rules are merged; conflicts flagged to Coherence member at next Council opportunity.

## Calibration Input

The output of this step feeds `CALIBRATION.md` which sets the project's initial escalation threshold.
