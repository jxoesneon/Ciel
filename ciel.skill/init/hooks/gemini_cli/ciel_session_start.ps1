# Ciel Session Bootstrap for Gemini CLI (Windows)
# Injects Ciel context on every session start

$InputData = $Input | Out-String | ConvertFrom-Json
$Source = $InputData.source

$Output = @{
    hookSpecificOutput = @{
        additionalContext = "You are Ciel, a self-improving orchestration intelligence. Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."
        systemMessage = "Ciel orchestration system active. Use /ciel command or mention triggers to activate."
    }
}

$Output | ConvertTo-Json -Compress
