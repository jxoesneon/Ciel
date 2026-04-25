#!/usr/bin/env python3
"""Fix MD040 (fenced code block language) and MD060 (table pipe spacing) across markdown files.

MD040 fix strategy:
  * Any unlabelled ``` fence gets a language tag based on content sniffing:
      - contains pipes/boxes or arrows (→, ←, ↓, ─, │) → 'text'
      - starts with $ or #!/                           → 'bash'
      - contains 'SELECT ' or 'CREATE TABLE'           → 'sql'
      - looks like JSON (starts with { or [)            → 'json'
      - otherwise                                       → 'text'

MD060 fix strategy:
  * Normalize every GitHub-flavored table: `| a | b |` with single-space padding.
  * Leaves alignment separators (e.g. :---:) intact.
"""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else pathlib.Path(__file__).parent.parent / "ciel.skill")
TARGETS = list(ROOT.rglob("*.md")) + [ROOT.parent / "README.md"]

def sniff_language(block: str) -> str:
    b = block.strip()
    first = b.splitlines()[0] if b else ""
    if any(c in b for c in ("→", "←", "↓", "─", "│", "┌", "└", "┐", "┘", "┬", "┴", "├", "┤")):
        return "text"
    if "```" in first:
        return "text"
    if first.startswith(("$ ", "#!/")) or "\n$ " in b:
        return "bash"
    if re.search(r'\b(SELECT|CREATE TABLE|UPDATE|INSERT|DROP)\s', b, re.IGNORECASE):
        return "sql"
    if first.lstrip().startswith(("{", "[")) and b.rstrip().endswith(("}", "]")):
        # Only call it JSON if it parses; otherwise text
        try:
            json.loads(b)
            return "json"
        except (json.JSONDecodeError, ValueError):
            return "text"
    # YAML heuristic: at least 2 top-level block mapping keys AND no prose
    # markers like long sentences with periods.
    yaml_keys = len(re.findall(r'^[A-Za-z_][A-Za-z0-9_]*:\s', b, re.MULTILINE))
    prose_sentences = len(re.findall(r'[.?!]\s+[A-Z]', b))
    if yaml_keys >= 2 and prose_sentences == 0:
        return "yaml"
    return "text"


def fix_md040(text: str) -> str:
    """Add language tag to bare ``` opening fences."""
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # Detect opening fence (```$ or ``` with just whitespace)
        m = re.match(r'^(\s*)```\s*$', line)
        if m:
            # collect block content until closing fence
            indent = m.group(1)
            body_lines = []
            j = i + 1
            while j < len(lines):
                if re.match(r'^(\s*)```\s*$', lines[j]):
                    break
                body_lines.append(lines[j])
                j += 1
            body = "\n".join(body_lines)
            lang = sniff_language(body)
            out.append(f"{indent}```{lang}")
            out.extend(body_lines)
            if j < len(lines):
                out.append(lines[j])
            i = j + 1
            continue
        # Detect existing language tag (already has one) — pass through
        m2 = re.match(r'^(\s*)```\s*\S+', line)
        if m2:
            # pass the open fence
            out.append(line)
            i += 1
            while i < len(lines):
                out.append(lines[i])
                if re.match(r'^(\s*)```\s*$', lines[i]):
                    i += 1
                    break
                i += 1
            continue
        out.append(line)
        i += 1
    return "\n".join(out)


def fix_md060(text: str) -> str:
    """Normalize markdown tables to `| cell | cell |` style."""
    lines = text.split("\n")
    out = []
    i = 0
    in_code = False
    code_fence_re = re.compile(r'^\s*```')
    while i < len(lines):
        line = lines[i]
        if code_fence_re.match(line):
            in_code = not in_code
            out.append(line)
            i += 1
            continue
        if in_code:
            out.append(line)
            i += 1
            continue

        # Detect a potential table row: starts with optional whitespace then |
        if re.match(r'^\s*\|', line) and line.count("|") >= 2:
            # Collect contiguous table block
            block = []
            while i < len(lines) and re.match(r'^\s*\|', lines[i]) and lines[i].count("|") >= 2:
                block.append(lines[i])
                i += 1
            # Normalize each row
            # Preserve leading whitespace of the first line for indentation alignment
            leading_ws = re.match(r'^(\s*)', block[0]).group(1)
            norm_rows = []
            for row in block:
                # Strip leading/trailing whitespace and outer pipes safely
                stripped = row.strip()
                if not stripped.startswith("|"):
                    stripped = "|" + stripped
                if not stripped.endswith("|"):
                    stripped = stripped + "|"
                # Split on | but keep inner escapes
                # GitHub tables don't require escaping; split plainly
                cells = stripped.split("|")
                # First and last are empty
                inner = cells[1:-1]
                # Trim each cell; add space padding on both sides
                norm_cells = []
                for c in inner:
                    c = c.strip()
                    # Alignment separator rows: contain only -, :, whitespace
                    if re.fullmatch(r'[\s:\-]+', c):
                        # Normalize: keep leading/trailing colon, center with hyphens
                        left = c.startswith(":")
                        right = c.endswith(":")
                        core = re.sub(r'[^-]', '', c.replace(":", ""))
                        if len(core) < 3:
                            core = "---"
                        sep = ("" if not left else ":") + core + ("" if not right else ":")
                        norm_cells.append(sep)
                    else:
                        norm_cells.append(c)
                # Use padded style consistently, including separator rows.
                norm_rows.append(leading_ws + "| " + " | ".join(norm_cells) + " |")
            out.extend(norm_rows)
            continue
        out.append(line)
        i += 1
    return "\n".join(out)


def main():
    fixed = 0
    for path in TARGETS:
        if not path.exists():
            continue
        original = path.read_text(encoding="utf-8")
        new = fix_md040(original)
        new = fix_md060(new)
        if new != original:
            path.write_text(new, encoding="utf-8")
            fixed += 1
    print(f"fixed {fixed} files")

if __name__ == "__main__":
    main()
