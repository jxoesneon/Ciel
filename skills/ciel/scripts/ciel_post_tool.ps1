[CmdletBinding()]
param()

$rawInput = ""
try {
    $rawInput = [Console]::In.ReadToEnd()
} catch {
}

if (-not [string]::IsNullOrWhiteSpace($rawInput)) {
    try {
        $data = $rawInput | ConvertFrom-Json
        $cielHome = if ($env:CIEL_HOME) { $env:CIEL_HOME } else { Join-Path $HOME ".ciel" }
        $activityLog = Join-Path $cielHome "activity.log"
        if (Test-Path $cielHome) {
            $ts = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            $hasError = [bool]($data.error)
            $logObj = @{
                ts = $ts
                kind = if ($hasError) { "tool_failure" } else { "post_tool" }
                stepIdx = $data.stepIdx
                hasError = $hasError
            }
            Add-Content -Path $activityLog -Value ($logObj | ConvertTo-Json -Compress) -ErrorAction SilentlyContinue
        }
    } catch {
    }
}

# PostToolUse contract expects empty JSON object on stdout
Write-Output "{}"
