# MCP — Xcode Coding Assistant

Xcode Coding Assistant exposes project-aware functionality through MCP tools. Ciel treats these tools as the primary integration surface for Apple-platform projects.

## Preferred Tool Mapping

| Capability | Xcode MCP tool |
| --- | --- |
| Project listing | `XcodeLS` |
| File discovery | `XcodeGlob` |
| Text search | `XcodeGrep` |
| File read | `XcodeRead` |
| File edit | `XcodeUpdate`, `XcodeWrite`, `XcodeMV`, `XcodeRM` |
| Compiler diagnostics | `XcodeRefreshCodeIssuesInFile` |
| Full build | `BuildProject`, then `GetBuildLog` |
| Tests | `GetTestList`, `RunSomeTests`, `RunAllTests` |
| SwiftUI preview | `RenderPreview` |
| Apple documentation | `DocumentationSearch` |

## Invocation Rules

1. Prefer Xcode MCP tools over shell commands for project-structure operations.
2. Use shell only for repo-native tools that Xcode MCP does not expose, such as Flutter, package managers, or custom scripts.
3. Search Apple documentation through `DocumentationSearch` for evolving Apple APIs before relying on memory.
4. Keep MCP calls scoped to the active project unless the user explicitly requests cross-project work.

## Security

All non-read MCP calls pass through the Xcode hook-equivalent preflight in `HOOKS.md`. Arguments that may contain secrets are summarized rather than logged verbatim.
