import os
import re

def fix_markdown(content):
    if not content.strip():
        return content
    
    # MD012: No multiple blank lines
    content = re.sub(r'\n{3,}', '\n\n', content)
    
    # MD009: No trailing spaces
    lines = [line.rstrip() for line in content.splitlines()]
    
    # MD022: Blanks around headings
    # MD026: No trailing punctuation in heading
    temp_lines = []
    for i, line in enumerate(lines):
        if re.match(r'^#+ ', line):
            # MD026: Remove trailing colon, semicolon, etc. from headings
            line = re.sub(r'[:;,.]$', '', line.strip())
            
            if i > 0 and temp_lines and temp_lines[-1].strip() != '':
                temp_lines.append('')
            temp_lines.append(line)
            if i < len(lines) - 1 and lines[i+1].strip() != '' and not re.match(r'^#+ ', lines[i+1]):
                 temp_lines.append('')
        else:
            temp_lines.append(line)
    
    # MD031: Blanks around fences
    # MD040: Fenced code blocks language
    lines = temp_lines
    temp_lines = []
    in_code_block = False
    for i, line in enumerate(lines):
        if line.startswith('```'):
            if not in_code_block:
                # Opening fence
                if i > 0 and temp_lines and temp_lines[-1].strip() != '':
                    temp_lines.append('')
                # MD040: Default language if missing
                if line.strip() == '```':
                    line = '```text'
                temp_lines.append(line)
                in_code_block = True
            else:
                # Closing fence
                temp_lines.append(line)
                in_code_block = False
                if i < len(lines) - 1 and lines[i+1].strip() != '':
                    temp_lines.append('')
        else:
            temp_lines.append(line)

    # MD032: Blanks around lists
    # MD030: Spaces after list markers
    lines = temp_lines
    final_lines = []
    list_pattern = r'^(\s*)(\d+\.|[\*\-\+])(\s+)'
    
    for i, line in enumerate(lines):
        m = re.match(list_pattern, line)
        if m:
            # MD030: Ensure exactly one space after marker
            indent = m.group(1)
            marker = m.group(2)
            rest = line[m.end():].lstrip()
            line = f"{indent}{marker} {rest}"
            
            # MD032: Blank line above list
            prev_is_list = i > 0 and re.match(list_pattern, lines[i-1])
            if not prev_is_list and i > 0 and final_lines and final_lines[-1].strip() != '':
                final_lines.append('')
            final_lines.append(line)
            
            # MD032: Blank line below list
            if i < len(lines) - 1:
                next_is_list = re.match(list_pattern, lines[i+1])
                if not next_is_list and lines[i+1].strip() != '' and not lines[i+1].startswith('```'):
                    final_lines.append('')
        else:
            final_lines.append(line)
            
    fixed = '\n'.join(final_lines)
    # MD047: Single trailing newline
    fixed = fixed.strip() + '\n'
    return fixed

if __name__ == "__main__":
    root_dir = r'C:\Users\Eduardo\Ciel'
    exclude_dirs = {'.git', 'dist', 'node_modules', '.ciel', '.attic', 'archive'}

    fixed_count = 0
    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        for f in files:
            if f.endswith('.md'):
                path = os.path.join(root, f)
                try:
                    with open(path, 'r', encoding='utf-8') as file:
                        content = file.read()
                    fixed = fix_markdown(content)
                    if content != fixed:
                        with open(path, 'w', encoding='utf-8') as file:
                            file.write(fixed)
                        print(f"Fixed {os.path.relpath(path, root_dir)}")
                        fixed_count += 1
                except Exception as e:
                    print(f"Error fixing {path}: {e}")

    print(f"\nTotal files fixed: {fixed_count}")
