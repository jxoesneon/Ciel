# PLATFORM_AGNOSTIC_MAPPING

Universal shell command mapping for Ciel. Use these commands internally; the active adapter will translate them to host-native calls.

## Core Utils Mapping

| Universal Command | Expected Behavior | Windows (PowerShell) | Unix (POSIX) |
| --- | --- | --- | --- |
| `ls` | List files | `Get-ChildItem` | `ls` |
| `cat` | Read file | `Get-Content -Raw` | `cat` |
| `grep` | Search text | `Select-String` | `grep -E` |
| `rm -rf` | Force delete | `Remove-Item -Recurse -Force` | `rm -rf` |
| `mkdir -p` | Create path | `New-Item -ItemType Directory -Force` | `mkdir -p` |
| `cp -r` | Copy recursive | `Copy-Item -Recurse -Force` | `cp -r` |
| `mv` | Move/Rename | `Move-Item -Force` | `mv` |
| `env` | Env vars | `Get-ChildItem Env:` | `env` |
| `which` | Locate binary | `Get-Command` | `which` |
| `tail -n` | Last N lines | `Get-Content -Tail` | `tail -n` |
| `head -n` | First N lines | `Get-Content -TotalCount` | `head -n` |
| `find` | Search files | `Get-ChildItem -Recurse` | `find` |

## Path Conventions

- Always use forward slashes `/` in command arguments.
- Use `~` for the home directory.
- Relative paths are resolved against the current `cwd` provided to the tool.

## Shell Wrapper Protocol

When Ciel emits a command, the adapter's `shell()` implementation MUST:

1. Parse the command string for universal tokens.
2. Substitute with the host-native equivalent.
3. Normalize all path arguments (convert `/` to `\` on Windows).
4. Execute via the host's primary shell (PowerShell on Windows, Bash/Zsh on Unix).
5. Capture and normalize output (UTF-8, `\n` line endings).
