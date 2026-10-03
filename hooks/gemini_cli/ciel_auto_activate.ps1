# Ciel Auto-Activation Hook for Gemini CLI (Windows)
# Intercepts trigger phrases to activate Ciel skill

$InputData = $Input | Out-String | ConvertFrom-Json
$Prompt = $InputData.prompt

$Triggers = @("ciel", "route this", "orchestrate", "find skill", "acquire skill", "self-improve")
$ShouldActivate = $false

foreach ($Trigger in $Triggers) {
    if ($Prompt -like "*$Trigger*") {
        $ShouldActivate = $true
        break
    }
}

if ($ShouldActivate) {
    $Output = @{
        decision = "activate"
        skill = "ciel"
        reason = "User prompt matched Ciel trigger phrase."
    }
} else {
    $Output = @{
        decision = "allow"
    }
}

$Output | ConvertTo-Json -Compress
