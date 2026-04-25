#!/usr/bin/env powershell
# Validate YAML frontmatter across core, skills, and agents (PowerShell)
# Usage: .\scripts\validate-frontmatter.ps1

$ErrorActionPreference = "Stop"

$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$FAILED = 0

function Say($msg) {
    Write-Host "[fm] $msg" -ForegroundColor Cyan
}

function Fail($msg) {
    Write-Host "[fm] FAIL: $msg" -ForegroundColor Red
    $script:FAILED++
}

Say "Extracting frontmatter from core, skills, and agents..."

$TARGETS = @(
    (Join-Path $ROOT "ciel.skill"),
    (Join-Path $ROOT "skills"),
    (Join-Path $ROOT "agents")
)

$fm_regex = '(?s)^---\s*\n(.*?)\n---\s*\n'
$checked = 0

foreach ($target in $TARGETS) {
    if (-not (Test-Path $target -PathType Container)) { continue }
    
    $files = Get-ChildItem -Path $target -Filter "*.md" -Recurse
    foreach ($file in $files) {
        $content = Get-Content $file.FullName -Raw
        if ($content -match $fm_regex) {
            $checked++
            $fm = $Matches[1]
            
            # Audit check: Ensure license and tags are present in harmonized skills
            if ($file.FullName -match "\\skills\\" -and $file.Name -eq "SKILL.md") {
                $relPath = $file.FullName.Substring($ROOT.Length + 1)
                if ($fm -notmatch '(?m)^license:') {
                    Fail "$relPath : missing 'license' key"
                }
                if ($fm -notmatch '(?m)^tags:') {
                    Fail "$relPath : missing 'tags' key"
                }
            }
        }
    }
}

Write-Host "[fm] checked $checked frontmatter blocks; $FAILED invalid"

if ($FAILED -eq 0) {
    exit 0
} else {
    exit 1
}
