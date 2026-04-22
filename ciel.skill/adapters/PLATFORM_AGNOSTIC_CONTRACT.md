# PLATFORM_AGNOSTIC_CONTRACT

Ciel's brain is OS-neutral. This contract defines how adapters MUST handle OS-specific differences to maintain a consistent internal reasoning surface.

## 1. Path Normalization (The "/" Rule)

Ciel's internal memory, skill registry, and reasoning always use **POSIX-style forward slashes (`/`)** for all paths.

- **Adapters MUST:**
  - Translate incoming host paths (e.g., `C:\Users\Eduardo\.ciel`) to Ciel-neutral paths (`/Users/Eduardo/.ciel` or similar internal abstraction) before surfacing to the model.
  - Translate outgoing Ciel paths to host-native paths before executing shell or filesystem tools.
  - Standardize on `~` for the user's home directory.

## 2. Universal Command Set (The "Coreutils" Shim)

Ciel reasons using a standardized set of shell commands. Adapters MUST map these to the host's native equivalent.

| Universal Command | Expected Behavior | Windows (PowerShell) Mapping | Unix (POSIX) Mapping |
| --- | --- | --- | --- |
| `ls <path>` | List directory contents | `Get-ChildItem -Path <path>` | `ls <path>` |
| `cat <file>` | Output file content | `Get-Content -Path <file> -Raw` | `cat <file>` |
| `grep <regex> <path>` | Search file contents | `Select-String -Pattern <regex> -Path <path>` | `grep -E <regex> <path>` |
| `rm -rf <path>` | Force remove path | `Remove-Item -Path <path> -Recurse -Force` | `rm -rf <path>` |
| `mkdir -p <path>` | Create directory tree | `New-Item -ItemType Directory -Path <path> -Force` | `mkdir -p <path>` |
| `cp -r <src> <dest>` | Recursive copy | `Copy-Item -Path <src> -Destination <dest> -Recurse -Force` | `cp -r <src> <dest>` |
| `mv <src> <dest>` | Move/Rename | `Move-Item -Path <src> -Destination <dest> -Force` | `mv <src> <dest>` |
| `env` | List environment variables | `Get-ChildItem Env:` | `env` |

## 3. Shell Execution Standard

- All shell commands executed by Ciel MUST be wrapped by the adapter's `shell()` function.
- The adapter is responsible for setting the correct execution policy (e.g., `-ExecutionPolicy Bypass` for PowerShell).
- Environment variables MUST be normalized (e.g., `$HOME` vs `%USERPROFILE%`).

## 4. Encoding Guarantee

- All text files handled by Ciel MUST be **UTF-8 encoded** without BOM.
- Adapters MUST handle line ending conversion (`\n` internally, potentially `\r\n` on host disk if requested, though `\n` is preferred for cross-platform compatibility).

## 5. Bootstrap Neutrality

- OS-specific scripts (`install.sh`, `install.ps1`) are deprecated in favor of a single **`init/scripts/setup.py`** or a host-runtime-native JS script.
- The bootstrap sequence MUST use the **Universal Command Set** via the adapter even during its own first run.

## 6. Implementation Checklist for Adapters

- [ ] Path translator (forward ↔ backslash).
- [ ] Command mapper (POSIX ↔ PowerShell).
- [ ] Encoding validator (Force UTF-8).
- [ ] Environment normalizer ($HOME, $PATH, etc.).
