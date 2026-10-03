[CmdletBinding()]
param()

$rawInput = ""
try {
    $rawInput = [Console]::In.ReadToEnd()
} catch {
}

$decision = "allow"
$reason = "Ciel pre-flight check passed."

if (-not [string]::IsNullOrWhiteSpace($rawInput)) {
    try {
        $data = $rawInput | ConvertFrom-Json
        $toolCall = $data.toolCall
        $toolName = if ($toolCall) { $toolCall.name } else { "" }
        $args = if ($toolCall) { $toolCall.args } else { $null }

        # Safety Guard: Intercept destructive commands
        if ($toolName -eq "run_command" -and $args -and $args.CommandLine) {
            $cmd = [string]$args.CommandLine
            $criticalPatterns = @(
                'rmdir\s+/[sS]',
                'Remove-Item\s+.*-Recurse\s+.*[C-Zc-z]:\\',
                'Format-Volume',
                'diskpart',
                'drop\s+database'
            )
            foreach ($pattern in $criticalPatterns) {
                if ($cmd -match $pattern) {
                    $decision = "ask"
                    $reason = "Ciel Council Safety Guard: High-risk command detected matching '$pattern'."
                    break
                }
            }
        }

        # Activity logging
        $cielHome = if ($env:CIEL_HOME) { $env:CIEL_HOME } else { Join-Path $HOME ".ciel" }
        $activityLog = Join-Path $cielHome "activity.log"
        if (Test-Path $cielHome) {
            $ts = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            $logEntry = @{
                ts = $ts
                kind = "pre_tool"
                tool = $toolName
                decision = $decision
            }
            Add-Content -Path $activityLog -Value ($logEntry | ConvertTo-Json -Compress) -ErrorAction SilentlyContinue
        }
    } catch {
        $decision = "allow"
    }
}

$output = @{
    decision = $decision
    reason = $reason
}
Write-Output ($output | ConvertTo-Json -Compress)
