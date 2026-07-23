# Obsidian backend environment setup for PowerShell
# Source this file to set the required environment variables for the current session:
#   . .\scripts\obsidian\setup-env.ps1

$env:OBSIDIAN_API_URL = "http://127.0.0.1:27123"
$env:OBSIDIAN_VAULT_PATH = Join-Path $PSScriptRoot "..\..\obsidian-brain" | Resolve-Path
$env:OBSIDIAN_HYBRID_SEARCH_URL = "http://127.0.0.1:3939"
$env:KG_VAULT_PATH = $env:OBSIDIAN_VAULT_PATH
$env:KG_REPO_PATH = Join-Path $env:USERPROFILE ".ciel\tools\knowledge-graph" | Resolve-Path
$env:KG_DATA_DIR = Join-Path $env:USERPROFILE ".local\share\knowledge-graph"

$dataPath = Join-Path $env:OBSIDIAN_VAULT_PATH ".obsidian\plugins\obsidian-local-rest-api\data.json"
if (Test-Path $dataPath) {
  $data = Get-Content $dataPath -Raw | ConvertFrom-Json
  $env:OBSIDIAN_API_KEY = $data.apiKey
  Write-Host "API key loaded from plugin data.json" -ForegroundColor Green
} else {
  Write-Warning "Plugin data.json not found at $dataPath. Run scripts/obsidian/generate-rest-api-key.mjs first."
}

Write-Host "Obsidian backend environment configured." -ForegroundColor Green
Write-Host "OBSIDIAN_API_URL = $env:OBSIDIAN_API_URL"
Write-Host "OBSIDIAN_VAULT_PATH = $env:OBSIDIAN_VAULT_PATH"
Write-Host "OBSIDIAN_HYBRID_SEARCH_URL = $env:OBSIDIAN_HYBRID_SEARCH_URL"
Write-Host "KG_REPO_PATH = $env:KG_REPO_PATH"
Write-Host "KG_DATA_DIR = $env:KG_DATA_DIR"
Write-Host ""
Write-Host "Run the self-test with: node ciel.skill/memory/backends/obsidian/cli.mjs --self-test"
