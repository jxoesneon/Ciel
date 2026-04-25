import os
import re

def fix_markdown(content):
    # MD022: Blanks around headings
    # Ensure there is exactly one blank line before and after # headings
    lines = content.splitlines()
    new_lines = []
    for i, line in enumerate(lines):
        if re.match(r'^#+ ', line):
            if i > 0 and new_lines and new_lines[-1].strip() != '':
                new_lines.append('')
            new_lines.append(line)
            if i < len(lines) - 1 and lines[i+1].strip() != '':
                new_lines.append('')
        else:
            new_lines.append(line)
    
    # MD032: Blanks around lists
    # Ensure there is a blank line before and after lists
    content = '\n'.join(new_lines)
    lines = content.splitlines()
    final_lines = []
    list_pattern = r'^(\d+\.|[\*\-\+]) '
    for i, line in enumerate(lines):
        is_list = re.match(list_pattern, line)
        prev_is_list = i > 0 and re.match(list_pattern, lines[i-1])
        next_is_list = i < len(lines) - 1 and re.match(list_pattern, lines[i+1])
        
        if is_list:
            if not prev_is_list and i > 0 and final_lines and final_lines[-1].strip() != '':
                final_lines.append('')
            final_lines.append(line)
            if not next_is_list and i < len(lines) - 1 and lines[i+1].strip() != '':
                final_lines.append('')
        else:
            final_lines.append(line)
            
    return '\n'.join(final_lines) + ('\n' if content.endswith('\n') else '')

agents_dir = r'C:\Users\Eduardo\Ciel\agents'
files_to_fix = [os.path.join(agents_dir, f) for f in os.listdir(agents_dir) if f.endswith('.md')]
files_to_fix.append(r'C:\Users\Eduardo\Ciel\README.md')
files_to_fix.append(r'C:\Users\Eduardo\Ciel\ciel.skill\SKILL.md')
files_to_fix.append(r'C:\Users\Eduardo\Ciel\ciel.skill\MANIFEST.md')
files_to_fix.append(r'C:\Users\Eduardo\Ciel\ciel.skill\CHANGELOG.md')

for path in files_to_fix:
    if os.path.exists(path):
        with open(path, 'r', encoding='utf-8') as file:
            content = file.read()
        fixed = fix_markdown(content)
        with open(path, 'w', encoding='utf-8') as file:
            file.write(fixed)
        print(f"Fixed {os.path.basename(path)}")
