#!/bin/bash
# Analyze a skill candidate for core integration
# Usage: ./scripts/analyze-skill.sh <skill-path>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SKILL_PATH="${1:-}"

if [[ -z "$SKILL_PATH" ]]; then
    echo "Usage: $0 <skill-path>"
    exit 1
fi

if [[ ! -d "$SKILL_PATH" ]]; then
    echo "Error: Skill path does not exist: $SKILL_PATH"
    exit 1
fi

SKILL_NAME=$(basename "$SKILL_PATH")
SKILL_MD="$SKILL_PATH/SKILL.md"

echo "=== Skill Analysis: $SKILL_NAME ==="
echo ""

# Check for SKILL.md
if [[ ! -f "$SKILL_MD" ]]; then
    echo "ERROR: No SKILL.md found"
    exit 1
fi

# Extract metadata
echo "## Metadata"
echo ""

NAME=$(grep "^name:" "$SKILL_MD" | head -1 | sed 's/name:\s*//' | tr -d '"' | tr -d "'" | xargs || echo "")
DESCRIPTION=$(grep "^description:" "$SKILL_MD" | head -1 | sed 's/description:\s*//' | tr -d '"' | tr -d "'" | xargs || echo "")

echo "- **Path**: $SKILL_PATH"
echo "- **Name**: ${NAME:-$SKILL_NAME}"
echo "- **Description**: ${DESCRIPTION:-"(none)"}"

# Detect format
if grep -q "format: skill/1.0" "$SKILL_MD" 2>/dev/null; then
    echo "- **Format**: Ciel-native (already integrated)"
    echo ""
    echo "Status: ALREADY INTEGRATED"
    exit 0
else
    echo "- **Format**: ECC Simple (requires harmonization)"
fi

echo ""

# File structure
echo "## File Structure"
echo ""
FILE_COUNT=$(find "$SKILL_PATH" -type f | wc -l)
echo "- **Total files**: $FILE_COUNT"

# List markdown files
MD_FILES=$(find "$SKILL_PATH" -name "*.md" -type f | head -10)
if [[ -n "$MD_FILES" ]]; then
    echo "- **Documentation files**:"
    echo "$MD_FILES" | while read f; do
        echo "  - $(basename "$f") ($(wc -l < "$f" | xargs) lines)"
    done
fi

echo ""

# Capability analysis
echo "## Capability Analysis"
echo ""

# Extract key capabilities from description
if [[ -n "$DESCRIPTION" ]]; then
    echo "- **Stated capabilities**:"
    
    # Look for action verbs
    for verb in "search" "find" "analyze" "check" "audit" "build" "create" "manage" "orchestrate" "automate" "review" "test" "deploy" "design" "write" "read" "process" "extract" "convert" "generate" "validate"; do
        if echo "$DESCRIPTION" | grep -qi "$verb"; then
            echo "  - $verb"
        fi
    done
fi

# Check for commands directory
if [[ -d "$SKILL_PATH/commands" ]]; then
    CMD_COUNT=$(ls "$SKILL_PATH/commands/" 2>/dev/null | wc -l)
    echo "- **Commands**: $CMD_COUNT"
fi

# Check for hooks
if [[ -d "$SKILL_PATH/hooks" ]] || [[ -d "$SKILL_PATH/.claude/hooks" ]]; then
    echo "- **Hooks**: Yes"
fi

# Check for MCP/tools
if [[ -d "$SKILL_PATH/mcp" ]] || [[ -d "$SKILL_PATH/tools" ]]; then
    echo "- **MCP/Tools**: Yes"
fi

echo ""

# Domain classification
echo "## Domain Classification"
echo ""

DOMAINS=""
for domain in "frontend" "backend" "api" "database" "testing" "security" "devops" "design" "mobile" "web" "cloud" "ai" "ml" "git" "github" "docker" "kubernetes" "aws" "testing" "documentation" "writing" "research" "analysis" "orchestration" "workflow"; do
    if echo "$NAME $DESCRIPTION" | grep -qi "$domain"; then
        DOMAINS="$DOMAINS $domain"
    fi
done

if [[ -n "$DOMAINS" ]]; then
    echo "- **Detected domains**:$DOMAINS"
else
    echo "- **Detected domains**: (general)"
fi

echo ""

# Council preparation
echo "## Council Presentation Summary"
echo ""
echo "```yaml"
echo "skill_candidate:"
echo "  name: ${NAME:-$SKILL_NAME}"
echo "  source: $SKILL_PATH"
echo "  format: ecc-simple"
echo "  harmonization_required: true"
echo "  domains: [$DOMAINS ]"
echo "  file_count: $FILE_COUNT"
echo "  documentation_lines: $(wc -l < "$SKILL_MD" | xargs)"
echo "```"

echo ""
echo "## Recommended Council Members"
echo ""

# Suggest which council members should focus
if echo "$DOMAINS" | grep -qi "security"; then
    echo "- **Safety** (primary): Security-related skill requires thorough risk assessment"
fi
if echo "$DOMAINS" | grep -qi "orchestration\|workflow"; then
    echo "- **Coherence** (primary): Orchestration pattern alignment check"
fi
if [[ $FILE_COUNT -gt 20 ]]; then
    echo "- **Efficiency** (primary): Large skill - composability review"
fi
echo "- **Capability**: Gap analysis vs existing registry"
echo "- **Evolution**: Self-improvement hooks assessment"

echo ""
echo "=== End Analysis ==="
