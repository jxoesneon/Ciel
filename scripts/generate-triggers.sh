#!/bin/bash
# Generate activation triggers for a skill
# Usage: ./scripts/generate-triggers.sh <skill-path>

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

echo "Generating triggers for: $SKILL_NAME"

# Extract from SKILL.md frontmatter
extract_frontmatter() {
    local file="$1"
    local field="$2"
    
    if [[ ! -f "$file" ]]; then
        echo ""
        return
    fi
    
    # Extract from frontmatter (between --- markers)
    sed -n '/^---$/,/^---$/p' "$file" | grep "^$field:" | head -1 | sed "s/$field:\s*//" | tr -d '"' | tr -d "'"
}

# Generate direct triggers
generate_direct_triggers() {
    local name="$1"
    local triggers=()
    
    # Base name
    triggers+=("$name")
    
    # Name variants
    local dashed=$(echo "$name" | tr '_' '-')
    local spaced=$(echo "$name" | tr '_' ' ' | tr '-' ' ')
    
    if [[ "$dashed" != "$name" ]]; then
        triggers+=("$dashed")
    fi
    
    triggers+=("$spaced")
    
    # Common prefixes
    triggers+=("use $name")
    triggers+=("run $name")
    triggers+=("call $name")
    
    printf '%s\n' "${triggers[@]}"
}

# Generate functional triggers from description
generate_functional_triggers() {
    local description="$1"
    local triggers=()
    
    # Common action verbs
    local verbs="search find get read write create update delete list analyze check"
    
    # Extract potential capabilities from description
    for verb in $verbs; do
        if echo "$description" | grep -qi "$verb"; then
            # Generate pattern
            triggers+=("$verb.*")
        fi
    done
    
    printf '%s\n' "${triggers[@]}"
}

# Generate from commands directory
generate_command_triggers() {
    local commands_dir="$1"
    local triggers=()
    
    if [[ ! -d "$commands_dir" ]]; then
        return
    fi
    
    for cmd in "$commands_dir"/*.md; do
        if [[ -f "$cmd" ]]; then
            local cmd_name=$(basename "$cmd" .md)
            triggers+=("$cmd_name")
            triggers+=("run $cmd_name")
            triggers+=("$cmd_name command")
        fi
    done
    
    printf '%s\n' "${triggers[@]}"
}

# Generate from function/tool names
generate_tool_triggers() {
    local skill_path="$1"
    local triggers=()
    
    # Scan for common tool definition patterns
    for file in "$skill_path"/*.js "$skill_path"/*.py "$skill_path"/tools/*.json 2>/dev/null; do
        if [[ -f "$file" ]]; then
            # Extract function names (simplified)
            local funcs=$(grep -E "^(function|def|const|export)" "$file" 2>/dev/null | \
                sed -E 's/(function|def|const|export)\s+([a-zA-Z_]+).*/\2/' | \
                head -10)
            
            for func in $funcs; do
                # Convert snake_case to phrase
                local phrase=$(echo "$func" | tr '_' ' ')
                triggers+=("$phrase")
            done
        fi
    done
    
    printf '%s\n' "${triggers[@]}"
}

# Main generation
main() {
    local name=$(extract_frontmatter "$SKILL_MD" "name")
    local description=$(extract_frontmatter "$SKILL_MD" "description")
    
    if [[ -z "$name" ]]; then
        name="$SKILL_NAME"
    fi
    
    echo "Skill: $name"
    echo "Description: $description"
    echo ""
    
    # Collect all triggers
    declare -A all_triggers
    
    # Direct triggers (highest confidence)
    echo "## Direct Triggers (confidence: 0.9-1.0)"
    while IFS= read -r trigger; do
        if [[ -n "$trigger" ]]; then
            all_triggers["$trigger"]=1.0
            echo "- $trigger"
        fi
    done < <(generate_direct_triggers "$name")
    echo ""
    
    # Command triggers
    if [[ -d "$SKILL_PATH/commands" ]]; then
        echo "## Command Triggers (confidence: 0.9)"
        while IFS= read -r trigger; do
            if [[ -n "$trigger" && -z "${all_triggers[$trigger]:-}" ]]; then
                all_triggers["$trigger"]=0.9
                echo "- $trigger"
            fi
        done < <(generate_command_triggers "$SKILL_PATH/commands")
        echo ""
    fi
    
    # Functional triggers
    if [[ -n "$description" ]]; then
        echo "## Functional Triggers (confidence: 0.7-0.8)"
        while IFS= read -r trigger; do
            if [[ -n "$trigger" && -z "${all_triggers[$trigger]:-}" ]]; then
                all_triggers["$trigger"]=0.75
                echo "- $trigger"
            fi
        done < <(generate_functional_triggers "$description")
        echo ""
    fi
    
    # Tool triggers
    echo "## Tool Triggers (confidence: 0.6-0.7)"
    while IFS= read -r trigger; do
        if [[ -n "$trigger" && -z "${all_triggers[$trigger]:-}" ]]; then
            all_triggers["$trigger"]=0.65
            echo "- $trigger"
        fi
    done < <(generate_tool_triggers "$SKILL_PATH")
    echo ""
    
    # Write to triggers file
    local triggers_file="$SKILL_PATH/triggers.yaml"
    
    cat > "$triggers_file" << EOF
# Auto-generated triggers for $name
# Generated: $(date -Iseconds)
generator_version: "1.0.0"

skill: $name
triggers:
EOF
    
    # Sort by confidence and write
    for trigger in "${!all_triggers[@]}"; do
        local confidence=${all_triggers[$trigger]}
        echo "  - pattern: \"$trigger\"" >> "$triggers_file"
        echo "    confidence: $confidence" >> "$triggers_file"
        echo "    type: auto_generated" >> "$triggers_file"
    done
    
    echo "Triggers written to: $triggers_file"
    
    # Update SKILL.md with triggers if not present
    if [[ -f "$SKILL_MD" ]]; then
        if ! grep -q "^triggers:" "$SKILL_MD"; then
            echo ""
            echo "Note: SKILL.md does not have triggers: section in frontmatter"
            echo "Add the following to SKILL.md frontmatter:"
            echo ""
            echo "triggers:"
            for trigger in "${!all_triggers[@]}"; do
                echo "  - \"$trigger\""
            done | head -10
        fi
    fi
}

main
