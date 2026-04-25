---
name: google-workspace-management
version: 1.0.0
format: skill/1.0
description: CIEL's framework for managing Google Drive, Docs, Sheets, and Slides via CLI and MCP tools.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(manage|edit|clean|summarize|gws).*(google|drive|gdoc|gsheet|gmail|calendar)"
    confidence: 0.9
  - pattern: "gws auth setup"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Google Workspace Management (The Asset Layer)

This skill formalizes the operation of Google Workspace using the `gws` CLI and native MCP tools.

## Asset Operations (CLI & MCP)
1. **Gmail**: List, Read, and Draft. Mandate dry-run verification before sending any email.
2. **Drive**: Search by ID/Owner. upload/Download binary assets. Verify listing after upload.
3. **Calendar**: Check schedule and create Meet spaces. Sync links to Gmail drafts.
4. **Sheets/Docs**: Use range-aware edits (Sheets) and index-aware surgery (Docs). Prohibit full rewrites.

## Workspace Workflow (Multi-Step)
- **Collect & Archive**: Gmail attachment -> Drive Folder -> Summarize in Doc.
- **Meet & Greet**: Calendar event -> Generate Meet Link -> Email link to attendees.
- **Surgical Sheet Edit**: Identify the "Working System" (formulas/ranges) before inserting columns.

## Authentication & Setup
- **Hard Gate**: Guide users through `gws auth setup` and `gws auth login` if credentials are missing.
- **Scope**: Request only the minimum required scopes (Gmail.readonly, Drive.file, etc.).

## Anti-Patterns
- **The Blind Send**: Sending an automated email without showing the draft to the user first.
- **Stale Duplicate**: Editing an old file version because of drive search ambiguity.
- **PII Leakage**: Sharing a Sheet with public permissions that contains un-redacted PII.
