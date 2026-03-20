import os
import re
import sys

def lint_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Split frontmatter and content
    parts = re.split(r'^---\s*$', content, flags=re.MULTILINE)
    if len(parts) < 3:
        print(f"Skipping {file_path}: No frontmatter found")
        return

    frontmatter = parts[1]
    body = "---".join(parts[2:])

    # 1. Quote name and description in frontmatter
    new_frontmatter_lines = []
    for line in frontmatter.strip().split('\n'):
        if line.startswith('name:') or line.startswith('description:'):
            key, value = line.split(':', 1)
            value = value.strip()
            # Remove existing quotes if any to avoid double quoting
            if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
                value = value[1:-1]
            new_frontmatter_lines.append(f'{key}: "{value}"')
        else:
            new_frontmatter_lines.append(line)
    
    new_frontmatter = '\n'.join(new_frontmatter_lines)

    # 2. Ensure exactly one blank line between '---' and first header '#'
    body = body.lstrip()
    # If the first non-whitespace character is '#', ensure one blank line
    if body.startswith('#'):
        body = '\n' + body

    # 3. Ensure files end with exactly one newline
    # 4. Remove any trailing whitespace on any line
    
    final_content = f"---\n{new_frontmatter}\n---\n{body}"
    
    # Process line by line for trailing whitespace
    lines = final_content.splitlines()
    processed_lines = [line.rstrip() for line in lines]
    
    # Ensure exactly one newline at the end
    final_output = '\n'.join(processed_lines).rstrip() + '\n'

    with open(file_path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(final_output)
    print(f"Processed {file_path}")

if __name__ == "__main__":
    files = sys.argv[1:]
    for f in files:
        lint_file(f)
