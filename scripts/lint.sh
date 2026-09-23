#!/usr/bin/env bash
# Local mirror of the CI gates in .github/workflows/ci.yml.
# Runs every gate it can; tools that are not installed are skipped with a
# warning instead of failing. Exits 1 if any executed gate failed.

set -uo pipefail

cd "$(dirname "$0")/.."

FAILED=0

say() { printf "\033[1;36m[lint]\033[0m %s\n" "$*"; }
warn() { printf "\033[1;33m[lint]\033[0m %s\n" "$*" 1>&2; }

gate() {
  local name="$1"
  shift
  say "gate: $name"
  if "$@"; then
    say "pass: $name"
  else
    printf "\033[1;31m[lint]\033[0m FAIL: %s\n" "$name" 1>&2
    FAILED=1
  fi
}

SHELL_FILES=(
  ciel.skill/init/scripts/install.sh
  ciel.skill/init/scripts/verify.sh
  scripts/build-skill.sh
  scripts/validate-spec.sh
  scripts/validate-frontmatter.sh
  scripts/lint.sh
  ciel.skill/init/hooks/devin/*.sh
  ciel.skill/init/hooks/antigravity/*.sh
)

RUFF_PATHS=(scripts/ ciel.skill/init/scripts/ ciel.skill/init/hooks/lib/)

YAML_CONFIG='{extends: default, rules: {line-length: disable, comments: {min-spaces-from-content: 1}, truthy: {check-keys: false}, document-start: disable}}'

gate "validate-spec" ./scripts/validate-spec.sh
gate "validate-frontmatter" ./scripts/validate-frontmatter.sh
gate "python unittest" python3 -m unittest discover tests

if command -v uvx >/dev/null 2>&1; then
  # --isolated: ignore any stray pyproject.toml outside the repo so the
  # effective rule set matches CI's default (E4,E7,E9,F).
  gate "ruff" uvx ruff check --no-cache --isolated "${RUFF_PATHS[@]}"
  gate "yamllint" uvx yamllint -d "$YAML_CONFIG" .github/ ciel.skill/
else
  warn "uvx not found; skipping ruff and yamllint gates"
fi

if command -v shellcheck >/dev/null 2>&1; then
  gate "shellcheck" shellcheck -S warning "${SHELL_FILES[@]}"
else
  warn "shellcheck not found; skipping shellcheck gate"
fi

if command -v shfmt >/dev/null 2>&1; then
  # shellcheck disable=SC2207
  SH_FILES=($(find . -name '*.sh' -not -path './dist/*' -not -path './.git/*'))
  gate "shfmt" shfmt -d -i 2 -ci "${SH_FILES[@]}"
else
  warn "shfmt not found; skipping shfmt gate"
fi

if [ "$FAILED" -ne 0 ]; then
  printf "\033[1;31m[lint]\033[0m one or more gates failed\n" 1>&2
  exit 1
fi
say "all gates passed"
