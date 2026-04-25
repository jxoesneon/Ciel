#!/usr/bin/env powershell
# Build Ciel-vX.Y.Z.skill from ciel.skill/ (PowerShell version)
# Usage: .\scripts\build-skill.ps1 -Version <version>
#   version: semver without leading 'v' (e.g. 1.2.3 or 1.2.3-rc.1)
# Outputs to dist/:
#   Ciel-vX.Y.Z.skill          (ZIP archive; the installable bundle)
#   Ciel-vX.Y.Z.skill.sha256   (SHA-256 of the .skill file)
#   SHA256SUMS                 (aggregate)

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

# Normalize: strip any leading 'v'
$Version = $Version -replace '^v', ''

# Validate semver (loose; accepts pre-release + build metadata)
$semverPattern = '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
if (-not ($Version -match $semverPattern)) {
    Write-Error "error: version '$Version' is not valid semver"
    exit 2
}

$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$SRC = Join-Path $ROOT "ciel.skill"
$DIST = Join-Path $ROOT "dist"
$ARTIFACT = "Ciel-v${Version}.skill"

if (-not (Test-Path $SRC -PathType Container)) {
    Write-Error "error: source dir not found: $SRC"
    exit 1
}

# Cross-check the version against SKILL.md + MANIFEST.md
$skillContent = Get-Content (Join-Path $SRC "SKILL.md") -Raw
$manifestContent = Get-Content (Join-Path $SRC "MANIFEST.md") -Raw

if ($skillContent -match '^version:\s*(.+)') {
    $declSkill = $matches[1].Trim()
} else {
    $declSkill = $null
}

if ($manifestContent -match '^- \*\*Version\*\*:\s*`(.+?)`') {
    $declManifest = $matches[1].Trim()
} else {
    $declManifest = $null
}

if ($declSkill -and $declSkill -ne $Version) {
    Write-Error "error: SKILL.md declares version '$declSkill' but build requested '$Version'"
    exit 3
}

if ($declManifest -and $declManifest -ne $Version) {
    Write-Warning "MANIFEST.md declares version '$declManifest' but build requested '$Version'"
}

Write-Host "==> Building $ARTIFACT"
New-Item -ItemType Directory -Force -Path $DIST | Out-Null
Remove-Item -Path (Join-Path $DIST "*") -Include "*.skill","*.sha256","SHA256SUMS" -ErrorAction SilentlyContinue

# Stage a clean copy (exclude OS / editor cruft)
$STAGE = Join-Path $env:TEMP ("ciel-build-" + [Guid]::NewGuid().ToString().Substring(0, 8))
New-Item -ItemType Directory -Force -Path $STAGE | Out-Null

$excludePatterns = @('.DS_Store', 'Thumbs.db', '*.swp', '*~', '.idea', '.vscode', '*.bak', '*.orig')

function Copy-Filtered {
    param($Source, $Dest, $Exclude)
    Get-ChildItem -Path $Source -Recurse | ForEach-Object {
        $shouldExclude = $false
        foreach ($pattern in $Exclude) {
            if ($_.Name -like $pattern) {
                $shouldExclude = $true
                break
            }
        }
        if (-not $shouldExclude) {
            $relPath = $_.FullName.Substring($Source.Length + 1)
            $destPath = Join-Path $Dest $relPath
            if ($_.PSIsContainer) {
                New-Item -ItemType Directory -Force -Path $destPath | Out-Null
            } else {
                Copy-Item $_.FullName $destPath -Force
            }
        }
    }
}

$STAGE_CIEL = Join-Path $STAGE "ciel.skill"
Copy-Filtered -Source $SRC -Dest $STAGE_CIEL -Exclude $excludePatterns

# Ensure install scripts retain exec bit ( informational on Windows)
$installSh = Join-Path $STAGE_CIEL "init/scripts/install.sh"
$verifySh = Join-Path $STAGE_CIEL "init/scripts/verify.sh"

# Create ZIP using Compress-Archive
$zipPath = Join-Path $STAGE $ARTIFACT
$compressSrc = Join-Path $STAGE_CIEL "*"
Compress-Archive -Path $compressSrc -DestinationPath $zipPath -Force

# Move to dist
Move-Item $zipPath (Join-Path $DIST $ARTIFACT) -Force

# Hashes using Get-FileHash
$artifactPath = Join-Path $DIST $ARTIFACT
$hash = Get-FileHash $artifactPath -Algorithm SHA256
$hash.Hash | Out-File -FilePath "$artifactPath.sha256" -NoNewline
$hashLine = $hash.Hash + "  $ARTIFACT"
$hashLine | Out-File -FilePath (Join-Path $DIST "SHA256SUMS") -NoNewline

# Summary
Get-ChildItem $artifactPath, "$artifactPath.sha256" | Format-Table -Property Name, Length, LastWriteTime -AutoSize
Write-Host ""
Write-Host "==> Build complete: $artifactPath"

# Cleanup
Remove-Item -Recurse -Force $STAGE
