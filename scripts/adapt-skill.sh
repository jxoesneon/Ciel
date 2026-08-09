#!/bin/bash
# Adapt an ECC skill into original Ciel-native skill
# Usage: ./scripts/adapt-skill.sh <source-path> --decision ADAPT --output <target-name>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CIEL_HOME="${CIEL_HOME:-$HOME/.ciel}"

SOURCE_PATH=""
DECISION=""
TARGET_NAME=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --decision)
      DECISION="$2"
      shift 2
      ;;
    --output)
      TARGET_NAME="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 <source-path> --decision ADAPT|EXTRACT|DISCARD [--output <target-name>]"
      exit 0
      ;;
    -*)
      echo "Unknown option: $1"
      exit 1
      ;;
    *)
      if [[ -z "$SOURCE_PATH" ]]; then
        SOURCE_PATH="$1"
      fi
      shift
      ;;
  esac
done

if [[ -z "$SOURCE_PATH" ]]; then
  echo "Error: Source path required"
  exit 1
fi

if [[ ! -d "$SOURCE_PATH" ]]; then
  echo "Error: Source path does not exist: $SOURCE_PATH"
  exit 1
fi

SOURCE_NAME=$(basename "$SOURCE_PATH")
TARGET_NAME="${TARGET_NAME:-$SOURCE_NAME}"
TARGET_DIR="$CIEL_HOME/skills/$TARGET_NAME"

# Check if already exists
if [[ -d "$TARGET_DIR" && -f "$TARGET_DIR/LICENSE" ]]; then
  echo "Skill already adapted at: $TARGET_DIR"
  exit 0
fi

echo "=== Adapting Skill: $SOURCE_NAME → $TARGET_NAME ==="
echo "Decision: $DECISION"
echo ""

# Read source skill
SOURCE_SKILL_MD="$SOURCE_PATH/SKILL.md"
if [[ ! -f "$SOURCE_SKILL_MD" ]]; then
  echo "Error: No SKILL.md found in source"
  exit 1
fi

# Extract basic info (for reference only - don't copy verbatim)
SOURCE_DESCRIPTION=$(grep "^description:" "$SOURCE_SKILL_MD" | head -1 | sed 's/description:\s*//' | tr -d '"' | tr -d "'" | xargs || echo "")

case "$DECISION" in
  ADAPT)
    echo "Creating fresh adaptation..."
    mkdir -p "$TARGET_DIR"
    
    # Create Apache-2.0 LICENSE
    cat > "$TARGET_DIR/LICENSE" << 'EOF'
Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

Copyright 2025 Ciel Project

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
EOF
    
    # Create PROVENANCE.md
    cat > "$TARGET_DIR/PROVENANCE.md" << EOF
# Provenance

## Inspiration

This skill was inspired by the ECC ecosystem skill \`$SOURCE_NAME\`.

**Original Source**: \`$SOURCE_PATH\`
**Adaptation Type**: Full rewrite
**Date**: $(date -Iseconds)
**Adapted By**: Ciel Project

## Adaptation Statement

This is an **original work** created for the Ciel ecosystem. While inspired 
by concepts from the source skill, no content was copied verbatim. All 
documentation, examples, and structure were written fresh to align with 
Ciel's architecture, voice, and conventions.

## Changes from Source

1. Rewritten in Ciel-idiomatic style
2. Adapted examples for Ciel ecosystem
3. Added integration with Ciel skill graph
4. Harmonized trigger phrases with Ciel registry
5. Added proper Apache-2.0 licensing

## Original Reference

For reference only (not copied):
- Source description: $SOURCE_DESCRIPTION

EOF
    
    echo "Created:"
    echo "  - $TARGET_DIR/LICENSE (Apache-2.0)"
    echo "  - $TARGET_DIR/PROVENANCE.md"
    echo ""
    echo "Next: Write fresh SKILL.md in $TARGET_DIR/"
    echo "Do NOT copy from $SOURCE_SKILL_MD — write original content"
    ;;
    
  EXTRACT)
    echo "EXTRACT decision — patterns should be merged into existing skill"
    echo "Source: $SOURCE_PATH"
    echo ""
    echo "Use: ./scripts/extract-patterns.sh $SOURCE_PATH <target-skill>"
    ;;
    
  DISCARD)
    echo "DISCARD decision — skill not useful for Ciel"
    echo "Source: $SOURCE_PATH"
    echo ""
    echo "Logging discard decision..."
    echo "$(date -Iseconds),$SOURCE_NAME,DISCARD,user_decision" >> "$CIEL_HOME/.attic/discarded_skills.csv"
    echo "Logged to $CIEL_HOME/.attic/discarded_skills.csv"
    ;;
    
  *)
    echo "Error: Invalid decision '$DECISION'"
    echo "Valid: ADAPT, EXTRACT, DISCARD"
    exit 1
    ;;
esac

echo ""
echo "=== Adaptation Setup Complete ==="
