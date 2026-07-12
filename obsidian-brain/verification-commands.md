---
title: Verification Commands
type: index
tags: [index, verification, commands, ci]
status: active
created: 2026-07-11
---

# Verification Commands

Per-project commands for the Iron Law of verification. Run these after changes and before declaring a task complete.

## Vault integrity

```powershell
# Check frontmatter coverage and duplicate blocks
node -e "const fs=require('fs'),path=require('path'); const root='C:/Users/josee/Ciel/obsidian-brain'; let f=0,m=0,d=0; function w(dir){for(const e of fs.readdirSync(dir)){const p=path.join(dir,e); const s=fs.statSync(p); if(s.isDirectory()) w(p); else if(p.endsWith('.md')){f++; const t=fs.readFileSync(p,'utf8'); const x=t.match(/^---\n([\s\S]*?)\n---/); if(!x){if(!p.includes('.obsidian')&&!p.includes('__selftest')){m++; console.log('missing',path.relative(root,p));}} else if(t.slice(x[0].length).trimStart().startsWith('---')){d++; console.log('dup',path.relative(root,p));}}}} w(root); console.log('files',f,'missing',m,'duplicates',d);"
```

## Ciel (this vault / skill package)

| Check | Command | Path |
|---|---|---|
| Obsidian backend self-test | `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test` | `C:/Users/josee/Ciel` |
| Validate specs | `./scripts/validate-spec.sh` | `C:/Users/josee/Ciel` |
| Validate frontmatter | `./scripts/validate-frontmatter.sh` | `C:/Users/josee/Ciel` |
| Build `.skill` archive | `./scripts/build-skill.sh 1.0.0` | `C:/Users/josee/Ciel` |

## IPFS / dart_ipfs

| Check | Command | Path |
|---|---|---|
| Static analysis | `dart analyze --fatal-infos` | `C:/Users/josee/IPFS` |
| Unit tests | `dart test --reporter=compact --no-color` | `C:/Users/josee/IPFS` |
| Publish dry-run | `dart pub publish --dry-run` | `C:/Users/josee/IPFS` |

## X-Seed

| Check | Command | Path |
|---|---|---|
| Static analysis | `flutter analyze --fatal-infos` | `C:/Users/josee/X-Seed/x_seed` |
| Unit/integration tests | `flutter test` | `C:/Users/josee/X-Seed/x_seed` |
| Build (Play flavor) | `flutter build apk --debug --flavor play` | `C:/Users/josee/X-Seed/x_seed` |
| Build (Full flavor) | `flutter build apk --debug --flavor full` | `C:/Users/josee/X-Seed/x_seed` |

## Blindsight

| Check | Command | Path |
|---|---|---|
| Backend tests | `pytest` | `C:/Users/josee/blindsight` |
| Frontend dev server | `npm run dev` | `C:/Users/josee/blindsight` (verify exact script) |

## How to use

1. After making changes, run the relevant project commands.
2. If a command fails, fix the root cause, not just the symptom.
3. Update the project's overview or diary entry with the verification result.
4. Keep this page current when a project's toolchain changes.

## Related

- [[active]] — current priorities and blockers
- [[ciel/projects.md]] — project list
- [[ciel/kg/decisions/2026-07-11-obsidian-brain-cleanup-conventions]] — vault conventions
