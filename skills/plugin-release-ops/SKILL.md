---
name: plugin-release-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Claude Code plugin marketplace releases. Handles version bumping across marketplace.json, plugin.json, and package.json with build verification, git tagging, and GitHub release creation.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ops"]
triggers:
  - pattern: "(plugin|marketplace).*(release|publish|version|bump)"
    confidence: 0.9
  - pattern: "release.*(plugin|marketplace\\.json|plugin\\.json)"
    confidence: 0.85
source: { tier: 2, origin: "claude-code-plugin-release" }
dependencies: { skills: [], mcp: [], system: ["git", "gh", "npm"] }
side_effects: ["shell", "network"]
---

# CIEL ADAPTATION: Plugin Release Ops

This skill orchestrates the full release cycle for Claude Code plugin marketplace packages. It targets `marketplace.json` and `plugin.json` — the plugin-specific manifest files — NOT generic repository releases (see `opensource-and-repo-ops` for that). Ciel's risk classifier gates public releases as critical, triggering user escalation before any push or publish action.

## Preparation

- **Analyze**: Classify the change as PATCH (bug fixes), MINOR (features), or MAJOR (breaking).
- **Environment**: Extract repository owner and name from `git remote -v`.
- **Paths**: Verify `package.json`, `.claude-plugin/marketplace.json`, and `plugin/.claude-plugin/plugin.json` all exist.
- **Release notes**: Draft detailed release notes BEFORE starting the version bump.

## Workflow

1. **Bump versions**: Increment version strings in all three config files simultaneously.
2. **Verify consistency**: `grep` the new version across all files to confirm they match.
3. **Build**: Run `npm run build` to generate fresh artifacts.
4. **Commit**: `git add -A && git commit -m "chore: bump version to X.Y.Z"` — include build artifacts.
5. **Tag**: `git tag -a vX.Y.Z -m "Version X.Y.Z"`.
6. **Push**: `git push origin main && git push origin vX.Y.Z`.
7. **Release**: `gh release create vX.Y.Z --title "vX.Y.Z" --notes "RELEASE_NOTES"`.
8. **Changelog**: Regenerate `CHANGELOG.md` via `gh api repos/{owner}/{repo}/releases --paginate | ./scripts/generate_changelog.js > CHANGELOG.md`.
9. **Sync**: Commit and push the updated `CHANGELOG.md`.
10. **Finalize**: Run `git status` — the working tree MUST be clean.

## Risk Classification

- **Public release = critical**: Git push, GitHub release creation, and npm publish are irreversible public actions.
- Ciel's autonomy ladder escalates all critical operations to the user before execution.
- Never auto-approve a push or release without explicit user confirmation.

## Checklist

- All config files have matching versions
- `npm run build` succeeded
- Git tag created and pushed
- GitHub release created with notes
- `CHANGELOG.md` updated and pushed
- `git status` shows clean tree

## Anti-Patterns

- **Partial version bump**: Updating `package.json` but forgetting `marketplace.json` or `plugin.json` — always grep all three.
- **Uncommitted artifacts**: Leaving build output unstaged — the workflow requires `git add -A` including artifacts.
- **Untagged release**: Creating a GitHub release without a corresponding annotated git tag.
- **Stale changelog**: Forgetting to regenerate and push `CHANGELOG.md` after the release.
