# PLATFORM AGNOSTIC HOOKS — Gemini CLI

Ciel supports cross-platform hooks for Gemini CLI. Use the appropriate script extension for your platform.

## Hook Mapping

| Event | POSIX (.sh) | Windows (.ps1) | Universal (.py) |
| --- | --- | --- | --- |
| `tool.preinvoke` | `ciel_preflight.sh` | `ciel_preflight.ps1` | `ciel_preflight.py` |
| `tool.postinvoke` | `ciel_postflight.sh` | `ciel_postflight.ps1` | `ciel_postflight.py` |
| `tool.error` | `ciel_failure.sh` | `ciel_failure.ps1` | `ciel_failure.py` |
| `session.start` | `ciel_session_start.sh` | `ciel_session_start.ps1` | `ciel_session_start.py` |

## Implementation Strategy

1. **Auto-Detection:** The installer (`setup.py`) detects the OS and installs the matching hook set.
2. **Unified Logic:** Use Python for complex hook logic to maintain parity across platforms.
3. **Native Wrappers:** Small shell/powershell wrappers can call the unified Python hooks if the runtime requires specific script extensions.

## Example: Session Start (PowerShell)

```powershell

# ciel_session_start.ps1

$InputData = $Input | Out-String | ConvertFrom-Json
$Source = $InputData.source

$Output = @{
    hookSpecificOutput = @{
        additionalContext = "You are Ciel, a self-improving orchestration intelligence. Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."
        systemMessage = "Ciel orchestration system active. Use /ciel command or mention triggers to activate."
    }
}

$Output | ConvertTo-Json -Compress
```
