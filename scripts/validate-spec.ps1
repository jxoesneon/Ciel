#!/usr/bin/env powershell
# Validate that ciel.skill/ satisfies the spec (PowerShell version)
# Usage: .\scripts\validate-spec.ps1
# Exits non-zero on any failure. Suitable for CI gate on Windows.

$ErrorActionPreference = "Stop"

$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$SKILL = Join-Path $ROOT "ciel.skill"
$FAILED = 0

function Say($msg) {
    Write-Host "[validate] $msg" -ForegroundColor Cyan
}

function Fail($msg) {
    Write-Host "[validate] FAIL: $msg" -ForegroundColor Red
    $script:FAILED++
}

function Ok($msg) {
    Write-Host "[validate] ok: $msg" -ForegroundColor Green
}

if (-not (Test-Path $SKILL -PathType Container)) {
    Fail "$SKILL missing"
    exit 1
}

# ---------------------------------------------------------------- Expected files
$EXPECTED_FILES = @(
    "SKILL.md", "MANIFEST.md", "CHANGELOG.md",
    "core/IDENTITY.md", "core/CONSTITUTION.md", "core/AUTONOMY.md", "core/AWARENESS.md",
    "router/ROUTER.md", "router/FAST_PATH.md", "router/REASONING_PATH.md",
    "router/ACQUISITION_PATH.md", "router/RUNTIME_DETECTION.md",
    "router/ROUTE_REGISTRY.md", "router/CONTEXT_BUDGET.md",
    "adapters/ADAPTER_CONTRACT.md",
    "adapters/claude_code/ADAPTER.md", "adapters/claude_code/HOOKS.md",
    "adapters/claude_code/SUBAGENTS.md", "adapters/claude_code/SLASH_COMMANDS.md",
    "adapters/claude_code/MCP.md", "adapters/claude_code/COMPUTER_USE.md",
    "adapters/claude_code/ULTRAPLAN.md", "adapters/claude_code/PROMPT_CACHE.md",
    "adapters/claude_code/PERMISSIONS.md", "adapters/claude_code/REMOTE_CONTROL.md",
    "adapters/claude_code/COUNCIL_INVOCATION.md", "adapters/claude_code/CONTEXT_FILES.md",
    "adapters/gemini_cli/ADAPTER.md", "adapters/gemini_cli/HOOKS.md",
    "adapters/gemini_cli/SUBAGENTS.md", "adapters/gemini_cli/EXTENSIONS.md",
    "adapters/gemini_cli/PLAN_MODE.md", "adapters/gemini_cli/CHECKPOINTING.md",
    "adapters/gemini_cli/A2A.md", "adapters/gemini_cli/MULTIMODAL.md",
    "adapters/gemini_cli/TOKEN_CACHE.md", "adapters/gemini_cli/MODEL_ROUTING.md",
    "adapters/gemini_cli/INTERACTIVE_SHELL.md", "adapters/gemini_cli/COUNCIL_INVOCATION.md",
    "adapters/gemini_cli/CONTEXT_FILES.md",
    "adapters/windsurf/ADAPTER.md", "adapters/windsurf/HOOKS.md",
    "adapters/windsurf/SUBAGENTS.md", "adapters/windsurf/MCP.md",
    "adapters/windsurf/SPACES.md", "adapters/windsurf/WORKFLOWS.md",
    "adapters/windsurf/CHECKPOINTS.md", "adapters/windsurf/CONTEXT_FILES.md",
    "adapters/windsurf/COUNCIL_INVOCATION.md",
    "adapters/generic/ADAPTER.md", "adapters/generic/RESEARCH_PROTOCOL.md",
    "adapters/generic/CAPABILITY_PROBE.md",
    "council/COUNCIL.md", "council/CHAIRMAN.md", "council/ANONYMIZATION.md", "council/ESCALATION.md",
    "council/members/COHERENCE.md", "council/members/CAPABILITY.md", "council/members/SAFETY.md",
    "council/members/EFFICIENCY.md", "council/members/EVOLUTION.md",
    "council/rubrics/SCORING.md", "council/rubrics/VETO_CONDITIONS.md",
    "council/rubrics/PROMOTION_RUBRIC.md", "council/rubrics/CONFLICT_RUBRIC.md",
    "council/invocation_scopes/SKILL_INTEGRATION.md", "council/invocation_scopes/SKILL_CONFLICT.md",
    "council/invocation_scopes/SELF_MODIFICATION.md", "council/invocation_scopes/PROMOTION.md",
    "council/invocation_scopes/HIGH_RISK_OPS.md",
    "registry/REGISTRY.md", "registry/SCHEMA.md", "registry/INDEXING.md",
    "registry/VERSIONING.md", "registry/CONFLICT_DETECTION.md", "registry/COHERENCE_SWEEP.md",
    "acquisition/ACQUISITION.md", "acquisition/TIER_1_REGISTRY.md",
    "acquisition/TIER_2_MCP.md", "acquisition/TIER_3_WEB.md",
    "acquisition/COMPOSITION.md", "acquisition/HARMONIZATION.md",
    "acquisition/TRUST_MODEL.md", "acquisition/SANDBOX.md", "acquisition/SOURCES.md",
    "memory/MEMORY.md", "memory/MEMPALACE.md", "memory/PARTITION.md", "memory/INSTALL.md",
    "memory/FALLBACK.md", "memory/GLOBAL_STORE.md", "memory/LOCAL_STORE.md",
    "memory/MERGE_SEMANTICS.md", "memory/HEALTH_CHECK.md",
    "memory/backends/SQLITE.md", "memory/backends/FILESYSTEM.md", "memory/backends/CUSTOM.md",
    "init/INIT.md", "init/BOOTSTRAP.md", "init/CONTEXT_DETECTION.md",
    "init/CALIBRATION.md", "init/OVERRIDE.md", "init/INTEGRITY.md",
    "init/GIT_SETUP.md", "init/GITIGNORE.md", "init/LOCAL_SYNC.md", "init/BACKUP.md",
    "init/scripts/install.sh", "init/scripts/install.ps1", "init/scripts/verify.sh",
    "self_improvement/SELF_IMPROVEMENT.md", "self_improvement/TRIGGERS.md",
    "self_improvement/OUTCOME_SCORING.md", "self_improvement/REGRESSION_DETECTION.md",
    "self_improvement/ROLLBACK.md", "self_improvement/GLOBAL_IMPROVEMENT.md",
    "self_improvement/LOCAL_IMPROVEMENT.md",
    "risk/RISK.md", "risk/CLASSIFICATION.md", "risk/LOW_RISK.md",
    "risk/MID_HIGH_RISK.md", "risk/CRITICAL_RISK.md", "risk/LLM_JUDGE.md",
    "risk/ESCALATION_LADDER.md",
    "domains/DOMAINS.md", "domains/GLOBAL.md", "domains/LOCAL.md",
    "domains/PROMOTION.md", "domains/ISOLATION.md", "domains/MULTI_RUNTIME.md",
    "observability/OBSERVABILITY.md", "observability/ACTIVITY_LOG.md",
    "observability/OTEL.md", "observability/SESSION_SUMMARY.md", "observability/DEBUG.md",
    "configuration/CONFIGURATION.md", "configuration/SCHEMA.md",
    "configuration/DEFAULTS.md", "configuration/TUNING.md",
    "configuration/global/ciel.config.md", "configuration/global/router.config.md",
    "configuration/global/council.config.md", "configuration/global/acquisition.config.md",
    "configuration/global/memory.config.md", "configuration/global/risk.config.md",
    "configuration/global/improvement.config.md", "configuration/global/observability.config.md",
    "configuration/global/adapters.config.md",
    "configuration/local/project.config.md", "configuration/local/rules.config.md",
    "configuration/local/overrides.config.md", "configuration/local/escalation.config.md",
    "prompts/PROMPTS.md",
    "prompts/council/coherence_stage1.md", "prompts/council/coherence_stage2.md",
    "prompts/council/capability_stage1.md", "prompts/council/capability_stage2.md",
    "prompts/council/safety_stage1.md", "prompts/council/safety_stage2.md",
    "prompts/council/efficiency_stage1.md", "prompts/council/efficiency_stage2.md",
    "prompts/council/evolution_stage1.md", "prompts/council/evolution_stage2.md",
    "prompts/council/chairman_synthesis.md",
    "prompts/router/reasoning_path.md", "prompts/router/gap_detection.md",
    "prompts/router/runtime_detection.md",
    "prompts/acquisition/web_extraction.md", "prompts/acquisition/harmonization.md",
    "prompts/acquisition/trust_judgment.md",
    "prompts/risk/classification.md", "prompts/risk/llm_judge.md",
    "prompts/self_improvement/outcome_scoring.md",
    "prompts/self_improvement/regression_judgment.md",
    "prompts/self_improvement/improvement_proposal.md",
    "templates/TEMPLATES.md", "templates/skill.template.md", "templates/subagent.template.md",
    "templates/council_vote.template.md", "templates/activity_log_entry.template.md",
    "templates/adapter.template.md", "templates/config.template.md",
    "templates/risk_assessment.template.md", "templates/improvement_proposal.template.md"
)

Say "Checking $($EXPECTED_FILES.Count) expected files..."
foreach ($rel in $EXPECTED_FILES) {
    $path = Join-Path $SKILL $rel
    if (-not (Test-Path $path)) {
        Fail "missing file: $rel"
    }
}
if ($FAILED -eq 0) { Ok "all expected files present" }

# ---------------------------------------------------------------- Seed skills glob check
Say "Checking seed skills (glob)..."
$seedSkills = Get-ChildItem -Path (Join-Path $SKILL "seed_skills") -Filter "SKILL.md" -Recurse
if ($seedSkills.Count -lt 33) {
    Fail "expected at least 33 seed skills; found $($seedSkills.Count)"
} else {
    Ok "found $($seedSkills.Count) seed skills"
}

# ---------------------------------------------------------------- External skills frontmatter
Say "Checking external skills frontmatter..."
$externalSkills = Get-ChildItem -Path (Join-Path $ROOT "skills") -Filter "SKILL.md" -Recurse
foreach ($f in $externalSkills) {
    $content = Get-Content $f.FullName -TotalCount 30
    foreach ($key in @("name", "version", "format", "runtimes", "triggers", "license", "tags", "description")) {
        if (-not ($content -match "^$key`:")) {
            Fail "$($f.FullName): missing frontmatter key '$key'"
        }
    }
}

# ---------------------------------------------------------------- Secret patterns
Say "Scanning for secret-like patterns in tracked content..."
$PATTERNS = @(
    'AKIA[0-9A-Z]{16}',             # AWS access key
    'ghp_[0-9A-Za-z]{36,}',         # GitHub PAT classic
    'github_pat_[0-9A-Za-z_]{70,}', # GitHub fine-grained
    'xoxb-[0-9A-Za-z-]+',           # Slack bot
    '-----BEGIN [A-Z ]*PRIVATE KEY-----'
)

foreach ($pat in $PATTERNS) {
    $findMatches = Get-ChildItem -Path $SKILL -Recurse -File | Select-String -Pattern $pat
    if ($findMatches) {
        Fail "secret-like pattern '$pat' matched in files: $($findMatches.Path -join ', ')"
    }
}

# ---------------------------------------------------------------- Version consistency
Say "Checking version consistency..."
$skillContent = Get-Content (Join-Path $SKILL "SKILL.md") -Raw
$manifestContent = Get-Content (Join-Path $SKILL "MANIFEST.md") -Raw

$verSkillMatch = [regex]::Match($skillContent, '(?m)^version:\s*([^\r\n]+)')
if ($verSkillMatch.Success) {
    $verSkill = $verSkillMatch.Groups[1].Value.Trim()
} else {
    Fail "could not extract version from SKILL.md"
    $verSkill = $null
}

$verManifestMatch = [regex]::Match($manifestContent, '(?m)^- \*\*Version\*\*:\s*`(.+?)`')
if ($verManifestMatch.Success) {
    $verManifest = $verManifestMatch.Groups[1].Value.Trim()
} else {
    Fail "could not extract version from MANIFEST.md (check format: - **Version**: \`X.Y.Z\`)"
    $verManifest = $null
}

if ($verSkill -and $verManifest -and $verSkill -ne $verManifest) {
    Fail "version mismatch: SKILL.md=$verSkill vs MANIFEST.md=$verManifest"
} elseif ($verSkill) {
    Ok "version consistent: $verSkill"
}

# ---------------------------------------------------------------- Summary
Write-Host ""
if ($FAILED -eq 0) {
    Write-Host "[validate] ALL CHECKS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Host "[validate] $FAILED CHECK(S) FAILED" -ForegroundColor Red
    exit 1
}
