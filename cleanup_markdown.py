import re
import os
import sys

def cleanup_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # 1. YAML Frontmatter
    def yaml_replacer(match):
        yaml_content = match.group(1)
        lines = yaml_content.split('\n')
        new_lines = []
        for line in lines:
            if line.startswith('name:') or line.startswith('description:'):
                key, value = line.split(':', 1)
                value = value.strip()
                if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
                    new_lines.append(f"{key}: {value}")
                else:
                    # Escape double quotes inside if necessary, but here we just wrap
                    escaped_value = value.replace('"', '\\"')
                    new_lines.append(f'{key}: "{escaped_value}"')
            else:
                new_lines.append(line)
        return '---\n' + '\n'.join(new_lines) + '\n---'

    content = re.sub(r'^---\n(.*?)\n---', yaml_replacer, content, flags=re.DOTALL)
    
    # Ensure exactly one blank line after YAML frontmatter
    content = re.sub(r'^---\n(.*?)\n---\n*', lambda m: m.group(0).split('---')[-1].strip() + '\n\n' if False else m.group(0).strip() + '\n\n', content, flags=re.DOTALL)
    # Wait, the above regex is a bit complex. Let's simplify.
    content = re.sub(r'^(---.*?---\n)\s*', r'\1\n', content, flags=re.DOTALL)

    # 3. Encoding & Characters
    content = content.replace('â€”', '--')

    # 4. Whitespace (Trailing)
    lines = content.split('\n')
    lines = [line.rstrip() for line in lines]
    
    # 2. Headers & 5. Lists
    new_lines = []
    def is_list_item(line):
        return re.match(r'^\s*([-*]|\d+\.) ', line)

    for i, line in enumerate(lines):
        # Header check
        if line.startswith('#'):
            # Ensure blank line BEFORE (except first line)
            if i > 0 and new_lines and new_lines[-1] != "":
                new_lines.append("")
            new_lines.append(line)
            # Ensure blank line AFTER is handled by the next check
            continue
        
        # If previous line was a header and this one isn't empty, insert empty
        if i > 0 and lines[i-1].startswith('#') and line != "":
            new_lines.append("")
            
        # List check: Ensure a blank line before the start of a list if it follows a paragraph.
        if is_list_item(line):
            # If it's the start of a list (previous line is not a list item and not empty and not a header)
            if i > 0 and lines[i-1] != "" and not is_list_item(lines[i-1]) and not lines[i-1].startswith('#'):
                new_lines.append("")
        
        new_lines.append(line)

    # Remove redundant multiple blank lines
    final_lines = []
    for i, line in enumerate(new_lines):
        if line == "" and i > 0 and final_lines[-1] == "":
            continue
        final_lines.append(line)
    
    content = '\n'.join(final_lines)
    
    # 4. Whitespace (Final newline)
    content = content.strip() + '\n'
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

if __name__ == "__main__":
    for path in sys.argv[1:]:
        if os.path.exists(path):
            cleanup_file(path)
            print(f"Cleaned up {path}")
        else:
            print(f"File not found: {path}")
