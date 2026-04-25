import os
import re

def fix_markdown(content):
    if not content.strip():
        return content
    
    # MD022: Blanks around headings
    lines = content.splitlines()
    new_lines = []
    for i, line in enumerate(lines):
        if re.match(r'^#+ ', line):
            if i > 0 and new_lines and new_lines[-1].strip() != '':
                new_lines.append('')
            new_lines.append(line)
            # Look ahead for MD022 below
            if i < len(lines) - 1 and lines[i+1].strip() != '' and not re.match(r'^#+ ', lines[i+1]):
                 new_lines.append('')
        else:
            new_lines.append(line)
    
    # MD032: Blanks around lists
    content = '\n'.join(new_lines)
    lines = content.splitlines()
    final_lines = []
    # Match list items: starting with *, -, +, or 1.
    list_pattern = r'^(\s*)(\d+\.|[\*\-\+])\s+'
    
    for i, line in enumerate(lines):
        is_list = re.match(list_pattern, line)
        
        if is_list:
            prev_is_list = i > 0 and re.match(list_pattern, lines[i-1])
            if not prev_is_list and i > 0 and final_lines and final_lines[-1].strip() != '':
                final_lines.append('')
            final_lines.append(line)
            
            # Look ahead
            if i < len(lines) - 1:
                next_is_list = re.match(list_pattern, lines[i+1])
                if not next_is_list and lines[i+1].strip() != '':
                    final_lines.append('')
        else:
            final_lines.append(line)
            
    fixed = '\n'.join(final_lines)
    # MD047: Single trailing newline
    fixed = fixed.strip() + '\n'
    return fixed

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
