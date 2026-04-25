# Validate that ciel.skill/ satisfies the spec (PowerShell version)
$ErrorActionPreference = "Continue"

$ROOT = Resolve-Path "$PSScriptRoot\.."
$SKILL = "$ROOT\ciel.skill"
$global:FAILED_COUNT = 0

function say($msg) { Write-Host "[validate] $msg" -ForegroundColor Cyan }
function fail($msg) { Write-Host "[validate] FAIL: $msg" -ForegroundColor Red; $global:FAILED_COUNT++ }
function ok($msg) { Write-Host "[validate] ok: $msg" -ForegroundColor Green }

if (-not (Test-Path $SKILL)) {
    fail "$SKILL missing"
    exit 1
}

# ---------------------------------------------------------------- Expected files
$EXPECTED_FILES = @(
  "CHANGELOG.md"
  "MANIFEST.md"
  "SKILL.md"
  "acquisition\ACQUISITION.md"
  "acquisition\COMPOSITION.md"
  "acquisition\HARMONIZATION.md"
  "acquisition\SANDBOX.md"
  "acquisition\SOURCES.md"
  "acquisition\TIER_1_REGISTRY.md"
  "acquisition\TIER_2_MCP.md"
  "acquisition\TIER_3_WEB.md"
  "acquisition\TRUST_MODEL.md"
  "adapters\ADAPTER_CONTRACT.md"
  "adapters\AUTO_ACTIVATION.md"
  "adapters\PLATFORM_AGNOSTIC_CONTRACT.md"
  "adapters\claude_code\ADAPTER.md"
  "adapters\claude_code\COMPUTER_USE.md"
  "adapters\claude_code\CONTEXT_FILES.md"
  "adapters\claude_code\COUNCIL_INVOCATION.md"
  "adapters\claude_code\HOOKS.md"
  "adapters\claude_code\MCP.md"
  "adapters\claude_code\PERMISSIONS.md"
  "adapters\claude_code\PROMPT_CACHE.md"
  "adapters\claude_code\REMOTE_CONTROL.md"
  "adapters\claude_code\SLASH_COMMANDS.md"
  "adapters\claude_code\SUBAGENTS.md"
  "adapters\claude_code\ULTRAPLAN.md"
  "adapters\gemini_cli\A2A.md"
  "adapters\gemini_cli\ADAPTER.md"
  "adapters\gemini_cli\CHECKPOINTING.md"
  "adapters\gemini_cli\CONTEXT_FILES.md"
  "adapters\gemini_cli\COUNCIL_INVOCATION.md"
  "adapters\gemini_cli\EXTENSIONS.md"
  "adapters\gemini_cli\HOOKS.md"
  "adapters\gemini_cli\INTERACTIVE_SHELL.md"
  "adapters\gemini_cli\MODEL_ROUTING.md"
  "adapters\gemini_cli\MULTIMODAL.md"
  "adapters\gemini_cli\PLAN_MODE.md"
  "adapters\gemini_cli\PLATFORM_AGNOSTIC_HOOKS.md"
  "adapters\gemini_cli\SUBAGENTS.md"
  "adapters\gemini_cli\TOKEN_CACHE.md"
  "adapters\generic\ADAPTER.md"
  "adapters\generic\CAPABILITY_PROBE.md"
  "adapters\generic\RESEARCH_PROTOCOL.md"
  "adapters\windsurf\ADAPTER.md"
  "adapters\windsurf\CHECKPOINTS.md"
  "adapters\windsurf\CONTEXT_FILES.md"
  "adapters\windsurf\COUNCIL_INVOCATION.md"
  "adapters\windsurf\HOOKS.md"
  "adapters\windsurf\MCP.md"
  "adapters\windsurf\SPACES.md"
  "adapters\windsurf\SUBAGENTS.md"
  "adapters\windsurf\WORKFLOWS.md"
  "assets\images\banner.jpg"
  "configuration\CONFIGURATION.md"
  "configuration\DEFAULTS.md"
  "configuration\SCHEMA.md"
  "configuration\TUNING.md"
  "configuration\global\acquisition.config.md"
  "configuration\global\adapters.config.md"
  "configuration\global\ciel.config.md"
  "configuration\global\council.config.md"
  "configuration\global\improvement.config.md"
  "configuration\global\memory.config.md"
  "configuration\global\observability.config.md"
  "configuration\global\risk.config.md"
  "configuration\global\router.config.md"
  "configuration\local\escalation.config.md"
  "configuration\local\overrides.config.md"
  "configuration\local\project.config.md"
  "configuration\local\rules.config.md"
  "core\AUTONOMY.md"
  "core\AWARENESS.md"
  "core\CONSTITUTION.md"
  "core\IDENTITY.md"
  "council\ANONYMIZATION.md"
  "council\CHAIRMAN.md"
  "council\COUNCIL.md"
  "council\ESCALATION.md"
  "council\invocation_scopes\HIGH_RISK_OPS.md"
  "council\invocation_scopes\PROMOTION.md"
  "council\invocation_scopes\SELF_MODIFICATION.md"
  "council\invocation_scopes\SKILL_CONFLICT.md"
  "council\invocation_scopes\SKILL_INTEGRATION.md"
  "council\members\CAPABILITY.md"
  "council\members\COHERENCE.md"
  "council\members\EFFICIENCY.md"
  "council\members\EVOLUTION.md"
  "council\members\SAFETY.md"
  "council\rubrics\CONFLICT_RUBRIC.md"
  "council\rubrics\PROMOTION_RUBRIC.md"
  "council\rubrics\SCORING.md"
  "council\rubrics\VETO_CONDITIONS.md"
  "domains\DOMAINS.md"
  "domains\GLOBAL.md"
  "domains\ISOLATION.md"
  "domains\LOCAL.md"
  "domains\MULTI_RUNTIME.md"
  "domains\PROMOTION.md"
  "init\BACKUP.md"
  "init\BOOTSTRAP.md"
  "init\CALIBRATION.md"
  "init\CONTEXT_DETECTION.md"
  "init\GITIGNORE.md"
  "init\gitignore.template"
  "init\GIT_SETUP.md"
  "init\INIT.md"
  "init\INTEGRITY.md"
  "init\integrity.sample.json"
  "init\LOCAL_SYNC.md"
  "init\OVERRIDE.md"
  "init\commands\claude_code\ciel.md"
  "init\commands\gemini_cli\ciel.md"
  "init\hooks\claude_code\ciel_auto_activate.sh"
  "init\hooks\claude_code\ciel_session_start.sh"
  "init\hooks\windsurf\post_cascade_response.sh"
  "init\hooks\windsurf\pre_user_prompt.sh"
  "init\scripts\install.ps1"
  "init\scripts\install.sh"
  "init\scripts\setup.py"
  "init\scripts\verify.sh"
  "init\workflows\windsurf\ciel-acquire.md"
  "init\workflows\windsurf\ciel-council.md"
  "init\workflows\windsurf\ciel-improve.md"
  "memory\ARTIFACTS.md"
  "memory\FALLBACK.md"
  "memory\GLOBAL_STORE.md"
  "memory\HEALTH_CHECK.md"
  "memory\INSTALL.md"
  "memory\LOCAL_STORE.md"
  "memory\MEMORY.md"
  "memory\MEMPALACE.md"
  "memory\MERGE_SEMANTICS.md"
  "memory\PARTITION.md"
  "memory\backends\CUSTOM.md"
  "memory\backends\FILESYSTEM.md"
  "memory\backends\SQLITE.md"
  "observability\ACTIVITY_LOG.md"
  "observability\DEBUG.md"
  "observability\OBSERVABILITY.md"
  "observability\OTEL.md"
  "observability\SESSION_SUMMARY.md"
  "prompts\PROMPTS.md"
  "prompts\acquisition\harmonization.md"
  "prompts\acquisition\trust_judgment.md"
  "prompts\acquisition\web_extraction.md"
  "prompts\council\capability_stage1.md"
  "prompts\council\capability_stage2.md"
  "prompts\council\chairman_synthesis.md"
  "prompts\council\coherence_stage1.md"
  "prompts\council\coherence_stage2.md"
  "prompts\council\efficiency_stage1.md"
  "prompts\council\efficiency_stage2.md"
  "prompts\council\evolution_stage1.md"
  "prompts\council\evolution_stage2.md"
  "prompts\council\safety_stage1.md"
  "prompts\council\safety_stage2.md"
  "prompts\risk\classification.md"
  "prompts\risk\llm_judge.md"
  "prompts\router\gap_detection.md"
  "prompts\router\reasoning_path.md"
  "prompts\router\runtime_detection.md"
  "prompts\self_improvement\improvement_proposal.md"
  "prompts\self_improvement\outcome_scoring.md"
  "prompts\self_improvement\regression_judgment.md"
  "registry\COHERENCE_SWEEP.md"
  "registry\CONFLICT_DETECTION.md"
  "registry\INDEXING.md"
  "registry\REGISTRY.md"
  "registry\SCHEMA.md"
  "registry\VERSIONING.md"
  "risk\CLASSIFICATION.md"
  "risk\CRITICAL_RISK.md"
  "risk\ESCALATION_LADDER.md"
  "risk\LLM_JUDGE.md"
  "risk\LOW_RISK.md"
  "risk\MID_HIGH_RISK.md"
  "risk\RISK.md"
  "router\ACQUISITION_PATH.md"
  "router\CONTEXT_BUDGET.md"
  "router\FAST_PATH.md"
  "router\REASONING_PATH.md"
  "router\ROUTER.md"
  "router\ROUTE_REGISTRY.md"
  "router\RUNTIME_DETECTION.md"
  "router\TRIGGER_GENERATOR.md"
  "router\TRIGGER_REGISTRY.md"
  "router\TRIGGER_SYSTEM.md"
  "seed_skills\SEED_SKILLS.md"
  "seed_skills\api_client\SKILL.md"
  "seed_skills\archive_manager\SKILL.md"
  "seed_skills\cicd_integration\SKILL.md"
  "seed_skills\code_analysis\SKILL.md"
  "seed_skills\code_generation\SKILL.md"
  "seed_skills\code_review\SKILL.md"
  "seed_skills\context_summarizer\SKILL.md"
  "seed_skills\council_runner\SKILL.md"
  "seed_skills\database_client\SKILL.md"
  "seed_skills\dependency_audit\SKILL.md"
  "seed_skills\diff_patch\SKILL.md"
  "seed_skills\docker\SKILL.md"
  "seed_skills\documentation\SKILL.md"
  "seed_skills\environment_detection\SKILL.md"
  "seed_skills\filesystem\SKILL.md"
  "seed_skills\git\SKILL.md"
  "seed_skills\json_yaml_toml_parser\SKILL.md"
  "seed_skills\linter_formatter\SKILL.md"
  "seed_skills\log_analyzer\SKILL.md"
  "seed_skills\markdown_processor\SKILL.md"
  "seed_skills\mcp_manager\SKILL.md"
  "seed_skills\mempalace_manager\SKILL.md"
  "seed_skills\package_manager\SKILL.md"
  "seed_skills\project_analyzer\SKILL.md"
  "seed_skills\research\SKILL.md"
  "seed_skills\runtime_adapter_builder\SKILL.md"
  "seed_skills\secrets_manager\SKILL.md"
  "seed_skills\shell\PLATFORM_AGNOSTIC_MAPPING.md"
  "seed_skills\shell\SKILL.md"
  "seed_skills\skill_builder\SKILL.md"
  "seed_skills\skill_installer\SKILL.md"
  "seed_skills\test_runner\SKILL.md"
  "seed_skills\web_fetch\SKILL.md"
  "seed_skills\web_search\SKILL.md"
  "self_improvement\GLOBAL_IMPROVEMENT.md"
  "self_improvement\GROWTH_SIGNAL_20260422_AMNESIAC_RECOVERY.md"
  "self_improvement\GROWTH_SIGNAL_20260422_BLIND_DELETION.md"
  "self_improvement\LOCAL_IMPROVEMENT.md"
  "self_improvement\OUTCOME_SCORING.md"
  "self_improvement\REGRESSION_DETECTION.md"
  "self_improvement\ROLLBACK.md"
  "self_improvement\SELF_IMPROVEMENT.md"
  "self_improvement\TRIGGERS.md"
  "templates\activity_log_entry.template.md"
  "templates\adapter.template.md"
  "templates\config.template.md"
  "templates\council_vote.template.md"
  "templates\improvement_proposal.template.md"
  "templates\risk_assessment.template.md"
  "templates\skill.template.md"
  "templates\subagent.template.md"
  "templates\TEMPLATES.md"
)

say "Checking $($EXPECTED_FILES.Count) expected files..."
foreach ($rel in $EXPECTED_FILES) {
    $path = Join-Path $SKILL $rel
    if (-not (Test-Path $path)) {
        fail "missing file: $rel"
    }
}
if ($global:FAILED_COUNT -eq 0) { ok "all expected files present" }

$validation_logic

# ---------------------------------------------------------------- Seed skills
say "Checking seed skills (glob)..."
$seeds = Get-ChildItem -Path "$SKILL\seed_skills" -Directory
ok "found $($seeds.Count) seed skills"

# ---------------------------------------------------------------- Secret patterns
say "Scanning for secret-like patterns in tracked content..."
$PATTERNS = @(
    'AKIA[0-9A-Z]{16}'             # AWS access key
    'ghp_[0-9A-Za-z]{36,}'         # GitHub PAT classic
    'github_pat_[0-9A-Za-z_]{70,}' # GitHub fine-grained
    'xoxb-[0-9A-Za-z-]+'           # Slack bot
    '-----BEGIN [A-Z ]*PRIVATE KEY-----'
)

foreach ($pat in $PATTERNS) {
    $matches = Get-ChildItem -Path $SKILL -Recurse -File | Select-String -Pattern $pat
    if ($matches) {
        fail "secret-like pattern '$pat' matched in $($matches.Count) places."
    }
}

# ---------------------------------------------------------------- Version consistency
say "Checking version consistency..."
$SKILL_CONTENT = Get-Content "$SKILL\SKILL.md" -Raw
$MANIFEST_CONTENT = Get-Content "$SKILL\MANIFEST.md" -Raw

$VER_SKILL = if ($SKILL_CONTENT -match 'version:\s*([^\r\n]+)') { $Matches[1].Trim() }
$VER_MANIFEST = if ($MANIFEST_CONTENT -match 'Version\*\*:\s*`([^`]+)`') { $Matches[1].Trim() }

if (-not $VER_SKILL) { fail "could not extract version from SKILL.md" }
if (-not $VER_MANIFEST) { fail "could not extract version from MANIFEST.md" }

if ($VER_SKILL -and $VER_MANIFEST -and $VER_SKILL -ne $VER_MANIFEST) {
    fail "version mismatch: SKILL.md=$VER_SKILL vs MANIFEST.md=$VER_MANIFEST"
} else {
    ok "version consistent: $VER_SKILL"
}

# ---------------------------------------------------------------- Summary
Write-Host ""
if ($global:FAILED_COUNT -eq 0) {
    Write-Host "[validate] ALL CHECKS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Host "[validate] $global:FAILED_COUNT CHECK(S) FAILED" -ForegroundColor Red
    exit 1
}
