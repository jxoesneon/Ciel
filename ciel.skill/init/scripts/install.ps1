# Ciel — single-call setup (Windows PowerShell)
# Runs bootstrap: creates ~/.ciel/, installs mempalace-rs, git-inits, verifies.

$ErrorActionPreference = "Stop"

$CielHome    = if ($env:CIEL_HOME) { $env:CIEL_HOME } else { Join-Path $HOME ".ciel" }
$CielVersion = if ($env:CIEL_VERSION) { $env:CIEL_VERSION } else { "1.2.0" }
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
allow_privileged
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

# --- 2b. Devin runtime - disable AI attribution -------------------------------
# Ciel's no-attribution mandate: durable artifacts (commits, PRs, issues,
# release notes, code comments, docs) must never carry "Generated with" /
# "Co-Authored-By" trailers, nor mention Ciel, the Council of Five, or the
# host runtime. Devin CLI injects such trailers unless `attribution` is false
# in ~/.config/devin/config.json - enforce it here; the Devin SessionStart
# hook re-verifies and self-heals on every session.
$DevinCfgDir = Join-Path $HOME ".config\devin"
if ((Test-Path $DevinCfgDir) -or (Need "devin")) {
    New-Item -ItemType Directory -Force -Path $DevinCfgDir | Out-Null
    $cfgPath = Join-Path $DevinCfgDir "config.json"
    try {
        $cfg = [pscustomobject]@{}
        if (Test-Path $cfgPath) { $cfg = Get-Content $cfgPath -Raw | ConvertFrom-Json }
        $cfg | Add-Member -NotePropertyName attribution -NotePropertyValue $false -Force
        $cfg | ConvertTo-Json -Depth 10 | Set-Content -Path $cfgPath
        Say "Devin attribution disabled (attribution=false in config.json)."
    } catch {
        Warn "Could not set Devin attribution flag; set `"attribution`": false in $cfgPath manually."
    }
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

# --- 8. Lifecycle hooks -------------------------------------------------------
$HookSrc = Join-Path $PSScriptRoot "..\hooks"
if (Test-Path $HookSrc) {
    New-Item -ItemType Directory -Force -Path (Join-Path $CielHome "hooks") | Out-Null
    Copy-Item -Recurse -Force -Path (Join-Path $HookSrc "*") -Destination (Join-Path $CielHome "hooks")
    Say "Lifecycle hooks installed."
} else {
    Warn "Hook payload directory not found; skipping hook install."
}

# --- 9. Risk policy ------------------------------------------------------------
$RiskSrc = Join-Path $PSScriptRoot "..\..\risk"
if (Test-Path (Join-Path $RiskSrc "policy.json")) {
    New-Item -ItemType Directory -Force -Path (Join-Path $CielHome "risk") | Out-Null
    Copy-Item -Path (Join-Path $RiskSrc "policy.yaml"), (Join-Path $RiskSrc "policy.json") -Destination (Join-Path $CielHome "risk")
    foreach ($seed in @("attribution_gate", "attribution_allowlist.txt")) {
        $seedPath = Join-Path $RiskSrc $seed
        if (Test-Path $seedPath) { Copy-Item -Path $seedPath -Destination (Join-Path $CielHome "risk") }
    }
    Say "Risk policy installed."
} else {
    Warn "Risk policy payload not found; hooks will use built-in fallback rules."
}

# --- 10. ciel-rs binary --------------------------------------------------------
# Optional fast path: the hooks prefer $CIEL_HOME/bin/ciel(.exe) and fall back
# to the embedded Python bodies when it is absent - this step never blocks the
# install. Resolution: cargo build from bundled source -> prebuilt download
# (CIEL_BIN_URL override) -> Python fallback.
$RsSrc  = Join-Path $PSScriptRoot "..\ciel-rs"
$BinDir = Join-Path $CielHome "bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$BinInstalled = $false

if ((Test-Path $RsSrc) -and (Need "cargo")) {
    Say "Building ciel-rs (cargo build --release)..."
    cargo build --release --manifest-path (Join-Path $RsSrc "Cargo.toml") --quiet
    if ($LASTEXITCODE -eq 0 -and (Test-Path (Join-Path $RsSrc "target\release\ciel.exe"))) {
        Copy-Item -Path (Join-Path $RsSrc "target\release\ciel.exe") -Destination (Join-Path $BinDir "ciel.exe")
        Say "ciel-rs installed to $BinDir\ciel.exe"
        $BinInstalled = $true
    } else {
        Warn "cargo build failed; trying prebuilt artifact."
    }
}

if (-not $BinInstalled -and ((Need "curl.exe") -or (Need "curl"))) {
    $plat = "windows-$env:PROCESSOR_ARCHITECTURE".ToLower()
    $url = if ($env:CIEL_BIN_URL) { $env:CIEL_BIN_URL } else {
        $base = if ($env:CIEL_RELEASE_BASE) { $env:CIEL_RELEASE_BASE } else {
            "https://github.com/jxoesneon/Ciel/releases/download/v$CielVersion" }
        "$base/ciel-$CielVersion-$plat.exe"
    }
    $tmpbin = Join-Path $env:TEMP "ciel-bin-$([guid]::NewGuid().ToString('N')).exe"
    $tmpsum = "$tmpbin.sha256"
    try {
        Invoke-WebRequest -Uri $url -OutFile $tmpbin -ErrorAction Stop
        Invoke-WebRequest -Uri "$url.sha256" -OutFile $tmpsum -ErrorAction Stop
        $expect = ((Get-Content $tmpsum)[0] -split '\s+')[0]
        $actual = (Get-FileHash $tmpbin -Algorithm SHA256).Hash.ToLower()
        if ($expect -eq $actual) {
            Copy-Item -Path $tmpbin -Destination (Join-Path $BinDir "ciel.exe")
            Say "ciel-rs prebuilt installed ($plat, sha256 verified)."
            $BinInstalled = $true
        } else {
            Warn "Checksum mismatch - refusing unverified binary."
        }
    } catch {
        Warn "Prebuilt download failed for $plat."
    } finally {
        Remove-Item $tmpbin, $tmpsum -ErrorAction SilentlyContinue
    }
}

if (-not $BinInstalled) {
    Warn "No ciel-rs binary available; hooks will use the Python fallback path."
}

# --- 11. Completion gate -------------------------------------------------------
# The System-1 completion gate ships in the bundle so verify_evidence.py and
# System-1 workflows can reach it on installed hosts; `ciel verify-completion`
# remains the binary fast path.
$GateSrc = Join-Path $PSScriptRoot "verify_completion.py"
if (Test-Path $GateSrc) {
    New-Item -ItemType Directory -Force -Path (Join-Path $CielHome "scripts") | Out-Null
    Copy-Item -Path $GateSrc -Destination (Join-Path $CielHome "scripts")
    Say "Completion gate installed to $CielHome\scripts."
}

# --- 12. Verify ---------------------------------------------------------------
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
