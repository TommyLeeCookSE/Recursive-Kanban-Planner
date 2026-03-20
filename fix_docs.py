import os
import re
import unicodedata
import sys

def normalize_to_ascii(text):
    # Replace em-dash and en-dash with ASCII hyphen
    text = text.replace('\u2014', '-').replace('\u2013', '-')
    # Normalize other characters
    nfkd_form = unicodedata.normalize('NFKD', text)
    return "".join([c for c in nfkd_form if not unicodedata.combining(c)])

def fix_frontmatter(frontmatter):
    lines = frontmatter.strip().split('\n')
    new_lines = []
    for line in lines:
        if ':' in line:
            key, value = line.split(':', 1)
            key = key.strip().lower()
            value = value.strip()
            
            # Remove existing quotes to analyze content
            unquoted_value = value
            if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
                unquoted_value = value[1:-1]
            
            # Unescape existing double quotes to avoid double-escaping
            unquoted_value = re.sub(r'\\+"', '"', unquoted_value)
            
            # Rule: NO quotes unless special characters like '&', ':', or '"'
            special_chars = ['&', ':', '"']
            needs_quotes = any(char in unquoted_value for char in special_chars)
            
            if needs_quotes:
                # Ensure it has double quotes and escape internal double quotes if any
                escaped_value = unquoted_value.replace('"', '\\"')
                new_lines.append(f'{key}: "{escaped_value}"')
            else:
                new_lines.append(f'{key}: {unquoted_value}')
        else:
            new_lines.append(line)
    return '---\n' + '\n'.join(new_lines) + '\n---'

def is_list_item(line):
    # Matches '- ', '* ', '+ ', '1. ', etc.
    return bool(re.match(r'^\s*([-*+]|\d+\.)\s+', line))

def fix_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Character normalization (Rule 3)
    content = normalize_to_ascii(content)

    # Split frontmatter
    frontmatter = ""
    body = content
    if content.startswith('---'):
        parts = re.split(r'^---\s*$', content, flags=re.MULTILINE)
        if len(parts) >= 3:
            frontmatter = fix_frontmatter(parts[1])
            body = "---".join(parts[2:])

    # Initial body cleanup: remove trailing whitespace (Rule 5)
    lines = [line.rstrip() for line in body.splitlines()]

    # Process lines for headers (Rule 2) and lists (Rule 4)
    new_lines = []
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Header processing
        if line.lstrip().startswith('#'):
            # Ensure space after #
            line = re.sub(r'^(#+)([^#\s])', r'\1 \2', line)
            
            # Ensure blank line BEFORE (except if it's the very first content after frontmatter)
            if new_lines and new_lines[-1] != "":
                new_lines.append("")
            
            new_lines.append(line)
            
            # Ensure blank line AFTER
            if i + 1 < len(lines) and lines[i+1] != "":
                new_lines.append("")
            
            i += 1
            continue

        # List processing
        if is_list_item(line):
            # Ensure blank line BEFORE the entire list
            if new_lines and new_lines[-1] != "" and not is_list_item(new_lines[-1]):
                new_lines.append("")
            
            # Add list items without blank lines between them
            while i < len(lines) and (is_list_item(lines[i]) or (lines[i].strip() == "" and i + 1 < len(lines) and is_list_item(lines[i+1]))):
                if is_list_item(lines[i]):
                    new_lines.append(lines[i])
                i += 1
            
            # Ensure blank line AFTER the entire list
            if i < len(lines) and lines[i] != "":
                new_lines.append("")
            continue

        # Regular lines
        if line == "" and new_lines and new_lines[-1] == "":
            # Skip redundant empty lines
            pass
        else:
            new_lines.append(line)
        i += 1

    # Final assembly
    body_text = '\n'.join(new_lines).strip()
    
    # Rule 1: EXACTLY one blank line between '---' and first header
    final_content = frontmatter + '\n\n' + body_text + '\n'
    
    # Final check for trailing whitespace on every line and exactly one newline at end
    final_lines = [line.rstrip() for line in final_content.splitlines()]
    final_output = '\n'.join(final_lines).rstrip() + '\n'

    if content != final_output:
        with open(file_path, 'w', encoding='utf-8', newline='\n') as f:
            f.write(final_output)
        return True
    return False

if __name__ == "__main__":
    import glob
    files = glob.glob('.agents/skills/**/SKILL.md', recursive=True) + glob.glob('.agents/workflows/*.md')
    print(f"Found {len(files)} files: {files}")
    modified = []
    for f in files:
        if fix_file(f):
            modified.append(f)
    
    if modified:
        for m in sorted(modified):
            print(m)
