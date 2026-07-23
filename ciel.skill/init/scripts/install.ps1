# Ciel — single-call setup (Windows PowerShell)
# Runs bootstrap: creates ~/.ciel/, installs Obsidian backend deps, git-inits, verifies.

$ErrorActionPreference = "Stop"

$CielHome    = if ($env:CIEL_HOME) { $env:CIEL_HOME } else { Join-Path $HOME ".ciel" }
$CielVersion = if ($env:CIEL_VERSION) { $env:CIEL_VERSION } else { "1.0.0" }
$Log         = Join-Path $CielHome "bootstrap.log"

function Say($msg)  { Write-Host "[ciel] $msg" -ForegroundColor Cyan; Add-Content -Path $Log -Value "[ciel] $msg" }
function Warn($msg) { Write-Warning "[ciel] $msg"; Add-Content -Path $Log -Value "[ciel] WARN $msg" }
function Die($msg)  { Write-Error "[ciel] $msg"; Add-Content -Path $Log -Value "[ciel] FAIL $msg"; exit 1 }
function Need($cmd) { $null -ne (Get-Command $cmd -ErrorAction SilentlyContinue) }

New-Item -ItemType Directory -Force -Path $CielHome | Out-Null
"" | Set-Content -Path $Log

Say "Ciel $CielVersion - single-call setup"
Say "CIEL_HOME=$CielHome"

# --- 1. Directory skeleton ---------------------------------------------------
$dirs = @('skills','registry','council','improvements','high_risk','acquisition','checkpoints','archive','.attic','sandbox','backups','integrity','runtimes')
foreach ($d in $dirs) { New-Item -ItemType Directory -Force -Path (Join-Path $CielHome $d) | Out-Null }
Say "Seed skill directory ready."

# --- 2. Git init -------------------------------------------------------------
if (Need "git") {
    if (-not (Test-Path (Join-Path $CielHome ".git"))) {
        Push-Location $CielHome
        try {
            git init -q
            git checkout -q -b main
            @'
.cache/
activity.log
backups/
archive/
fs_backend/
*.db
checkpoints/
.attic/
sandbox/
'@ | Set-Content -Path ".gitignore"
            git add -A
            git commit -q -m "genesis: Ciel cold start @ $CielVersion"
            Say "Git repository initialized."
        } finally { Pop-Location }
    } else {
        Say "Git repository already present."
    }
} else {
    Warn "git not found; skipping git setup."
}

# --- 3. Node.js toolchain ----------------------------------------------------
$SkipObsidianBackend = $false
if (-not (Need "node") -or -not (Need "npm")) {
    Warn "Node.js toolchain (node/npm) not found. Obsidian backend requires it."
    if ($env:CIEL_AUTO_INSTALL_NODE -eq "1") {
        Say "Auto-install of Node.js is not implemented in this bootstrap; please install Node.js and re-run."
    }
    $SkipObsidianBackend = $true
}

# --- 4. Obsidian backend dependencies ----------------------------------------
$ObsidianBackendDir = Join-Path $PSScriptRoot "..\..\memory\backends\obsidian" | Resolve-Path
if (-not $SkipObsidianBackend) {
    Say "Installing Obsidian backend dependencies (npm install)..."
    try {
        Push-Location $ObsidianBackendDir
        npm install | Out-Null
        Pop-Location
        Say "Obsidian backend dependencies installed."
    } catch {
        Warn "npm install failed for Obsidian backend; will fall back"
        $SkipObsidianBackend = $true
    }
}

# --- 5. Fallback backend -----------------------------------------------------
if ($SkipObsidianBackend) {
    if (Need "sqlite3") {
        Say "Configuring SQLite fallback backend."
        New-Item -ItemType File -Force -Path (Join-Path $CielHome "ciel.db") | Out-Null
    } else {
        Warn "sqlite3 not found; falling back to filesystem KV backend."
        New-Item -ItemType Directory -Force -Path (Join-Path $CielHome "fs_backend") | Out-Null
    }
}

# --- 6. Integrity seed -------------------------------------------------------
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
@"
{ "schema": 1, "version": "$CielVersion", "timestamp": "$now", "files": {} }
"@ | Set-Content -Path (Join-Path $CielHome "INTEGRITY.json")
Say "Integrity seed written."

# --- 7. Activity log ---------------------------------------------------------
Add-Content -Path (Join-Path $CielHome "activity.log") -Value "{`"ts`":`"$now`",`"kind`":`"bootstrap`",`"version`":`"$CielVersion`"}"

# --- 8. Verify ---------------------------------------------------------------
Say "Running verification..."
$FailedCheck = $false

# 8.1 Core Files Check
$coreFiles = @("SKILL.md", "MANIFEST.md", "router/ROUTER.md", "core/CONSTITUTION.md")
foreach ($f in $coreFiles) {
    if (-not (Test-Path (Join-Path $CielHome $f))) { Warn "Missing core file in home: $f"; $FailedCheck = $true }
}

# 8.2 Integrity Check
if (-not (Test-Path (Join-Path $CielHome "INTEGRITY.json"))) { Warn "Integrity seed missing"; $FailedCheck = $true }

# 8.3 Git Check
if (-not (Test-Path (Join-Path $CielHome ".git"))) { Warn "Git repo not initialized in home"; $FailedCheck = $true }

# 8.4 Obsidian backend check
if (-not $SkipObsidianBackend -and (Need "node")) {
    try {
        $selfTest = & node (Join-Path $ObsidianBackendDir "cli.mjs") --self-test 2>&1
        if ($LASTEXITCODE -ne 0) { Warn "Obsidian backend self-test failed"; $FailedCheck = $true }
        else { Say "Obsidian backend self-test passed." }
    } catch {
        Warn "Could not run Obsidian backend self-test: $_"; $FailedCheck = $true
    }
}

if ($FailedCheck) {
    Warn "One or more verification checks failed. Check bootstrap.log."
} else {
    Say "All verification checks passed."
}

Say "Ciel bootstrap complete."
