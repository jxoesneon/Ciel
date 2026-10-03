[CmdletBinding()]
param()

$rawInput = ""
try {
    $rawInput = [Console]::In.ReadToEnd()
} catch {
}

$injectSteps = @()

if (-not [string]::IsNullOrWhiteSpace($rawInput)) {
    try {
        $data = $rawInput | ConvertFrom-Json
        # Only inject on the first invocation to provide Ciel anchor context without inflating context tokens
        if ($data.invocationNum -eq 1) {
            $injectSteps = @(
                @{
                    ephemeralMessage = "[Ciel Active] Autonomous partner intelligence initialized. Adhere to the Iron Law of Verification (verify evidence before completion) and Council of Five governance."
                }
            )
        }
    } catch {
    }
}

$output = @{
    injectSteps = $injectSteps
}
Write-Output ($output | ConvertTo-Json -Compress)
