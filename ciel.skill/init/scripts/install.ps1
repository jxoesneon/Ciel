# Ciel — single-call setup (Windows PowerShell)
# Runs bootstrap: creates ~/.ciel/, installs mempalace-rs, git-inits, verifies.

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

# --- 3. Rust toolchain -------------------------------------------------------
$SkipMempalace = $false
if (-not (Need "cargo")) {
    Warn "Rust toolchain not found."
    if ($env:CIEL_AUTO_INSTALL_RUST -eq "1") {
        Say "Installing Rust via rustup-init.exe (auto)..."
        $rustInit = Join-Path $env:TEMP "rustup-init.exe"
        Invoke-WebRequest "https://win.rustup.rs" -OutFile $rustInit
        & $rustInit -y --default-toolchain stable --profile minimal | Out-Null
        $cargoBin = Join-Path $HOME ".cargo\bin"
        $env:PATH = "$cargoBin;$env:PATH"
    } else {
        Warn "Set CIEL_AUTO_INSTALL_RUST=1 to auto-install. Falling back."
        $SkipMempalace = $true
    }
}

# --- 4. MemPalace-rs ---------------------------------------------------------
if (-not $SkipMempalace -and (Need "cargo")) {
    if (-not (Need "mempalace-rs")) {
        Say "Installing mempalace-rs (cargo install --locked)..."
        try { cargo install mempalace-rs --locked } catch { Warn "cargo install failed; will fall back"; $SkipMempalace = $true }
    } else {
        Say "mempalace-rs already installed."
    }
}

# --- 5. Fallback backend -----------------------------------------------------
if ($SkipMempalace) {
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

# 8.4 MemPalace Check
if (Need "mempalace-rs") {
    try { & mempalace-rs status | Out-Null } catch { Warn "mempalace-rs not responding correctly"; $FailedCheck = $true }
}

if ($FailedCheck) {
    Warn "One or more verification checks failed. Check bootstrap.log."
} else {
    Say "All verification checks passed."
}

Say "Ciel bootstrap complete."
