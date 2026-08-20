#!/usr/bin/env bash
# Validate that ciel.skill/ satisfies the spec:
#   1. Every expected file is present.
#   2. Every *.md file has required frontmatter (where applicable).
#   3. All seed skills have name/version/triggers/tags in frontmatter.
#   4. install.sh / verify.sh are executable.
#   5. No secret-like strings in any tracked file.
#
# Exits non-zero on any failure. Suitable for CI gate.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SKILL="$ROOT/ciel.skill"
FAILED=0

say() { printf "\033[1;36m[validate]\033[0m %s\n" "$*"; }
fail() {
  printf "\033[1;31m[validate]\033[0m FAIL: %s\n" "$*" 1>&2
  FAILED=$((FAILED + 1))
}
ok() { printf "\033[1;32m[validate]\033[0m ok: %s\n" "$*"; }

[[ -d "$SKILL" ]] || {
  fail "$SKILL missing"
  exit 1
}

# ---------------------------------------------------------------- Expected files
EXPECTED_FILES=(
  "CHANGELOG.md"
  "MANIFEST.md"
  "SKILL.md"
  "acquisition/ACQUISITION.md"
  "acquisition/COMPOSITION.md"
  "acquisition/HARMONIZATION.md"
  "acquisition/SANDBOX.md"
  "acquisition/SOURCES.md"
  "acquisition/TIER_1_REGISTRY.md"
  "acquisition/TIER_2_MCP.md"
  "acquisition/TIER_3_WEB.md"
  "acquisition/TRUST_MODEL.md"
  "adapters/ADAPTER_CONTRACT.md"
  "adapters/AUTO_ACTIVATION.md"
  "adapters/PLATFORM_AGNOSTIC_CONTRACT.md"
  "adapters/claude_code/ADAPTER.md"
  "adapters/claude_code/COMPUTER_USE.md"
  "adapters/claude_code/CONTEXT_FILES.md"
  "adapters/claude_code/COUNCIL_INVOCATION.md"
  "adapters/claude_code/HOOKS.md"
  "adapters/claude_code/MCP.md"
  "adapters/claude_code/PERMISSIONS.md"
  "adapters/claude_code/PROMPT_CACHE.md"
  "adapters/claude_code/REMOTE_CONTROL.md"
  "adapters/claude_code/SLASH_COMMANDS.md"
  "adapters/claude_code/SUBAGENTS.md"
  "adapters/claude_code/ULTRAPLAN.md"
  "adapters/gemini_cli/A2A.md"
  "adapters/gemini_cli/ADAPTER.md"
  "adapters/gemini_cli/CHECKPOINTING.md"
  "adapters/gemini_cli/CONTEXT_FILES.md"
  "adapters/gemini_cli/COUNCIL_INVOCATION.md"
  "adapters/gemini_cli/EXTENSIONS.md"
  "adapters/gemini_cli/HOOKS.md"
  "adapters/gemini_cli/INTERACTIVE_SHELL.md"
  "adapters/gemini_cli/MODEL_ROUTING.md"
  "adapters/gemini_cli/MULTIMODAL.md"
  "adapters/gemini_cli/PLAN_MODE.md"
  "adapters/gemini_cli/PLATFORM_AGNOSTIC_HOOKS.md"
  "adapters/gemini_cli/SUBAGENTS.md"
  "adapters/gemini_cli/TOKEN_CACHE.md"
  "adapters/generic/ADAPTER.md"
  "adapters/generic/CAPABILITY_PROBE.md"
  "adapters/generic/RESEARCH_PROTOCOL.md"
  "adapters/windsurf/ADAPTER.md"
  "adapters/windsurf/CHECKPOINTS.md"
  "adapters/windsurf/CONTEXT_FILES.md"
  "adapters/windsurf/COUNCIL_INVOCATION.md"
  "adapters/windsurf/HOOKS.md"
  "adapters/windsurf/MCP.md"
  "adapters/windsurf/SPACES.md"
  "adapters/windsurf/SUBAGENTS.md"
  "adapters/windsurf/WORKFLOWS.md"
  "assets/images/banner.jpg"
  "configuration/CONFIGURATION.md"
  "configuration/DEFAULTS.md"
  "configuration/SCHEMA.md"
  "configuration/TUNING.md"
  "configuration/global/acquisition.config.md"
  "configuration/global/adapters.config.md"
  "configuration/global/ciel.config.md"
  "configuration/global/council.config.md"
  "configuration/global/improvement.config.md"
  "configuration/global/memory.config.md"
  "configuration/global/observability.config.md"
  "configuration/global/risk.config.md"
  "configuration/global/router.config.md"
  "configuration/local/escalation.config.md"
  "configuration/local/overrides.config.md"
  "configuration/local/project.config.md"
  "configuration/local/rules.config.md"
  "core/AUTONOMY.md"
  "core/AWARENESS.md"
  "core/CONSTITUTION.md"
  "core/IDENTITY.md"
  "council/ANONYMIZATION.md"
  "council/CHAIRMAN.md"
  "council/COUNCIL.md"
  "council/ESCALATION.md"
  "council/invocation_scopes/HIGH_RISK_OPS.md"
  "council/invocation_scopes/PROMOTION.md"
  "council/invocation_scopes/SELF_MODIFICATION.md"
  "council/invocation_scopes/SKILL_CONFLICT.md"
  "council/invocation_scopes/SKILL_INTEGRATION.md"
  "council/members/CAPABILITY.md"
  "council/members/COHERENCE.md"
  "council/members/EFFICIENCY.md"
  "council/members/EVOLUTION.md"
  "council/members/SAFETY.md"
  "council/rubrics/CONFLICT_RUBRIC.md"
  "council/rubrics/PROMOTION_RUBRIC.md"
  "council/rubrics/SCORING.md"
  "council/rubrics/VETO_CONDITIONS.md"
  "domains/DOMAINS.md"
  "domains/GLOBAL.md"
  "domains/ISOLATION.md"
  "domains/LOCAL.md"
  "domains/MULTI_RUNTIME.md"
  "domains/PROMOTION.md"
  "init/BACKUP.md"
  "init/BOOTSTRAP.md"
  "init/CALIBRATION.md"
  "init/CONTEXT_DETECTION.md"
  "init/GITIGNORE.md"
  "init/gitignore.template"
  "init/GIT_SETUP.md"
  "init/INIT.md"
  "init/INTEGRITY.md"
  "init/integrity.sample.json"
  "init/LOCAL_SYNC.md"
  "init/OVERRIDE.md"
  "init/commands/claude_code/ciel.md"
  "init/commands/gemini_cli/ciel.md"
  "init/hooks/claude_code/ciel_auto_activate.sh"
  "init/hooks/claude_code/ciel_session_start.sh"
  "init/hooks/windsurf/post_cascade_response.sh"
  "init/hooks/windsurf/pre_user_prompt.sh"
  "init/scripts/install.ps1"
  "init/scripts/install.sh"
  "init/scripts/setup.py"
  "init/scripts/verify.sh"
  "init/workflows/windsurf/ciel-acquire.md"
  "init/workflows/windsurf/ciel-council.md"
  "init/workflows/windsurf/ciel-improve.md"
  "memory/ARTIFACTS.md"
  "memory/FALLBACK.md"
  "memory/GLOBAL_STORE.md"
  "memory/HEALTH_CHECK.md"
  "memory/INSTALL.md"
  "memory/LOCAL_STORE.md"
  "memory/MEMORY.md"
  "memory/MEMPALACE.md"
  "memory/MERGE_SEMANTICS.md"
  "memory/PARTITION.md"
  "memory/backends/CUSTOM.md"
  "memory/backends/FILESYSTEM.md"
  "memory/backends/SQLITE.md"
  "observability/ACTIVITY_LOG.md"
  "observability/DEBUG.md"
  "observability/OBSERVABILITY.md"
  "observability/OTEL.md"
  "observability/SESSION_SUMMARY.md"
  "prompts/PROMPTS.md"
  "prompts/acquisition/harmonization.md"
  "prompts/acquisition/trust_judgment.md"
  "prompts/acquisition/web_extraction.md"
  "prompts/council/capability_stage1.md"
  "prompts/council/capability_stage2.md"
  "prompts/council/chairman_synthesis.md"
  "prompts/council/coherence_stage1.md"
  "prompts/council/coherence_stage2.md"
  "prompts/council/efficiency_stage1.md"
  "prompts/council/efficiency_stage2.md"
  "prompts/council/evolution_stage1.md"
  "prompts/council/evolution_stage2.md"
  "prompts/council/safety_stage1.md"
  "prompts/council/safety_stage2.md"
  "prompts/risk/classification.md"
  "prompts/risk/llm_judge.md"
  "prompts/router/gap_detection.md"
  "prompts/router/reasoning_path.md"
  "prompts/router/runtime_detection.md"
  "prompts/self_improvement/improvement_proposal.md"
  "prompts/self_improvement/outcome_scoring.md"
  "prompts/self_improvement/regression_judgment.md"
  "registry/COHERENCE_SWEEP.md"
  "registry/CONFLICT_DETECTION.md"
  "registry/INDEXING.md"
  "registry/REGISTRY.md"
  "registry/SCHEMA.md"
  "registry/VERSIONING.md"
  "risk/CLASSIFICATION.md"
  "risk/CRITICAL_RISK.md"
  "risk/ESCALATION_LADDER.md"
  "risk/LLM_JUDGE.md"
  "risk/LOW_RISK.md"
  "risk/MID_HIGH_RISK.md"
  "risk/RISK.md"
  "router/ACQUISITION_PATH.md"
  "router/CONTEXT_BUDGET.md"
  "router/FAST_PATH.md"
  "router/REASONING_PATH.md"
  "router/ROUTER.md"
  "router/ROUTE_REGISTRY.md"
  "router/RUNTIME_DETECTION.md"
  "router/TRIGGER_GENERATOR.md"
  "router/TRIGGER_REGISTRY.md"
  "router/TRIGGER_SYSTEM.md"
  "seed_skills/SEED_SKILLS.md"
  "seed_skills/api_client/SKILL.md"
  "seed_skills/archive_manager/SKILL.md"
  "seed_skills/cicd_integration/SKILL.md"
  "seed_skills/code_analysis/SKILL.md"
  "seed_skills/code_generation/SKILL.md"
  "seed_skills/code_review/SKILL.md"
  "seed_skills/context_summarizer/SKILL.md"
  "seed_skills/council_runner/SKILL.md"
  "seed_skills/database_client/SKILL.md"
  "seed_skills/dependency_audit/SKILL.md"
  "seed_skills/diff_patch/SKILL.md"
  "seed_skills/docker/SKILL.md"
  "seed_skills/documentation/SKILL.md"
  "seed_skills/environment_detection/SKILL.md"
  "seed_skills/filesystem/SKILL.md"
  "seed_skills/git/SKILL.md"
  "seed_skills/json_yaml_toml_parser/SKILL.md"
  "seed_skills/linter_formatter/SKILL.md"
  "seed_skills/log_analyzer/SKILL.md"
  "seed_skills/markdown_processor/SKILL.md"
  "seed_skills/mcp_manager/SKILL.md"
  "seed_skills/mempalace_manager/SKILL.md"
  "seed_skills/package_manager/SKILL.md"
  "seed_skills/project_analyzer/SKILL.md"
  "seed_skills/research/SKILL.md"
  "seed_skills/runtime_adapter_builder/SKILL.md"
  "seed_skills/secrets_manager/SKILL.md"
  "seed_skills/shell/PLATFORM_AGNOSTIC_MAPPING.md"
  "seed_skills/shell/SKILL.md"
  "seed_skills/skill_builder/SKILL.md"
  "seed_skills/skill_installer/SKILL.md"
  "seed_skills/test_runner/SKILL.md"
  "seed_skills/web_fetch/SKILL.md"
  "seed_skills/web_search/SKILL.md"
  "self_improvement/GLOBAL_IMPROVEMENT.md"
  "self_improvement/GROWTH_SIGNAL_20260422_AMNESIAC_RECOVERY.md"
  "self_improvement/GROWTH_SIGNAL_20260422_BLIND_DELETION.md"
  "self_improvement/LOCAL_IMPROVEMENT.md"
  "self_improvement/OUTCOME_SCORING.md"
  "self_improvement/REGRESSION_DETECTION.md"
  "self_improvement/ROLLBACK.md"
  "self_improvement/SELF_IMPROVEMENT.md"
  "self_improvement/TRIGGERS.md"
  "templates/activity_log_entry.template.md"
  "templates/adapter.template.md"
  "templates/config.template.md"
  "templates/council_vote.template.md"
  "templates/improvement_proposal.template.md"
  "templates/risk_assessment.template.md"
  "templates/skill.template.md"
  "templates/subagent.template.md"
  "templates/TEMPLATES.md"
)

say "Checking ${#EXPECTED_FILES[@]} expected files..."
for rel in "${EXPECTED_FILES[@]}"; do
  if [[ ! -f "$SKILL/$rel" ]]; then
    fail "missing file: $rel"
  fi
done
[[ $FAILED -eq 0 ]] && ok "all expected files present"

# ---------------------------------------------------------------- Exec bits
for s in init/scripts/install.sh init/scripts/verify.sh; do
  if [[ -f "$SKILL/$s" && ! -x "$SKILL/$s" ]]; then
    fail "not executable: $s"
  fi
done

# ---------------------------------------------------------------- Seed SKILL.md frontmatter
say "Checking seed-skill frontmatter..."
for f in "$SKILL"/seed_skills/*/SKILL.md; do
  [[ -f "$f" ]] || continue
  head -n 30 "$f" >"$f.head.$$"
  for key in name version description triggers tags runtimes license; do
    if ! grep -qE "^${key}:" "$f.head.$$"; then
      fail "$(realpath --relative-to="$SKILL" "$f" 2>/dev/null || echo "$f"): missing frontmatter key '${key}'"
    fi
  done
  rm -f "$f.head.$$"
done

# ---------------------------------------------------------------- External skills frontmatter
say "Checking external skills frontmatter..."
for f in "$ROOT"/skills/*/SKILL.md; do
  [[ -f "$f" ]] || continue
  head -n 30 "$f" >"$f.head.$$"
  for key in name version format runtimes triggers license tags description; do
    if ! grep -qE "^${key}:" "$f.head.$$"; then
      fail "$(realpath --relative-to="$ROOT" "$f" 2>/dev/null || echo "$f"): missing frontmatter key '${key}'"
    fi
  done
  rm -f "$f.head.$$"
done

# ---------------------------------------------------------------- Agents frontmatter
say "Checking agents frontmatter..."
for f in "$ROOT"/agents/*.md; do
  [[ -f "$f" ]] || continue
  head -n 30 "$f" >"$f.head.$$"
  for key in name version description; do
    if ! grep -qE "^${key}:" "$f.head.$$"; then
      fail "$(realpath --relative-to="$ROOT" "$f" 2>/dev/null || echo "$f"): missing frontmatter key '${key}'"
    fi
  done
  rm -f "$f.head.$$"
done

# ---------------------------------------------------------------- SKILL.md frontmatter (root)
say "Checking root SKILL.md frontmatter..."
for key in name version description entrypoint format runtimes; do
  if ! grep -qE "^${key}:" "$SKILL/SKILL.md"; then
    fail "SKILL.md missing frontmatter key '${key}'"
  fi
done

# ---------------------------------------------------------------- Secret patterns
say "Scanning for secret-like patterns in tracked content..."
# Ignore the template files (they contain shape examples) and prompts library
# Use conservative pattern set: AWS key id, GitHub PAT, generic API key shapes.
PATTERNS=(
  'AKIA[0-9A-Z]{16}'             # AWS access key
  'ghp_[0-9A-Za-z]{36,}'         # GitHub PAT classic
  'github_pat_[0-9A-Za-z_]{70,}' # GitHub fine-grained
  'xoxb-[0-9A-Za-z-]+'           # Slack bot
  '-----BEGIN [A-Z ]*PRIVATE KEY-----'
)
for pat in "${PATTERNS[@]}"; do
  matches="$(grep -RIEl --binary-files=without-match "$pat" "$SKILL" 2>/dev/null || true)"
  if [[ -n "$matches" ]]; then
    fail "secret-like pattern '$pat' matched in:"
    echo "$matches"
  fi
done

# ---------------------------------------------------------------- Version consistency
say "Checking version consistency..."
VER_SKILL="$(awk -F': *' '/^version:/{print $2; exit}' "$SKILL/SKILL.md" | tr -d '[:space:]')"
VER_MANIFEST="$(awk -F'`' '/^- \*\*Version\*\*:/{print $2; exit}' "$SKILL/MANIFEST.md")"

if [[ -z "$VER_SKILL" ]]; then
  fail "could not extract version from SKILL.md"
fi
if [[ -z "$VER_MANIFEST" ]]; then
  fail "could not extract version from MANIFEST.md (check format: - **Version**: \`X.Y.Z\`)"
fi

if [[ -n "$VER_SKILL" && -n "$VER_MANIFEST" && "$VER_SKILL" != "$VER_MANIFEST" ]]; then
  fail "version mismatch: SKILL.md=$VER_SKILL vs MANIFEST.md=$VER_MANIFEST"
else
  ok "version consistent: $VER_SKILL"
fi

# ---------------------------------------------------------------- Summary
echo
if [[ $FAILED -eq 0 ]]; then
  printf "\033[1;32m[validate]\033[0m ALL CHECKS PASSED\n"
  exit 0
else
  printf "\033[1;31m[validate]\033[0m %d CHECK(S) FAILED\n" "$FAILED"
  exit 1
fi
