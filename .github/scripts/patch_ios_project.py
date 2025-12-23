#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned builds.
Uses line-by-line processing to preserve pbxproj format.
"""
import glob
import sys
import re

def patch_pbxproj(path):
    """Patch pbxproj safely line by line."""
    print(f"Patching: {path}")
    
    with open(path, 'r') as f:
        lines = f.readlines()
    
    new_lines = []
    modified = False
    
    for line in lines:
        original_line = line
        
        # Skip shellScript lines containing tauri - replace with no-op
        if 'shellScript' in line and 'tauri' in line.lower():
            # Preserve indentation, replace content
            indent = len(line) - len(line.lstrip())
            line = ' ' * indent + 'shellScript = "echo \\"Rust library pre-built\\"";\n'
            modified = True
        
        # Simple string replacements (exact matches)
        simple_replacements = [
            ('CODE_SIGN_IDENTITY = "Apple Development"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "-"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "iPhone Developer"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
            ('CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
            ('CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),
            ('ProvisioningStyle = Automatic', 'ProvisioningStyle = Manual'),
            ('debug/libapp.a', 'release/libapp.a'),
        ]
        
        for old, new in simple_replacements:
            if old in line:
                line = line.replace(old, new)
                modified = True
        
        # Handle DEVELOPMENT_TEAM with regex (various formats)
        if 'DEVELOPMENT_TEAM' in line and '""' not in line:
            new_line = re.sub(r'DEVELOPMENT_TEAM\s*=\s*[^;]+;', 'DEVELOPMENT_TEAM = "";', line)
            if new_line != line:
                line = new_line
                modified = True
        
        new_lines.append(line)
    
    with open(path, 'w') as f:
        f.writelines(new_lines)
    
    print(f"✅ Patched: {path} ({'modified' if modified else 'no changes'})")

def main():
    pbxproj_files = glob.glob('src-tauri/gen/apple/**/*.pbxproj', recursive=True)
    
    if not pbxproj_files:
        print("⚠️  No pbxproj files found")
        sys.exit(0)
    
    for pbx in pbxproj_files:
        patch_pbxproj(pbx)
    
    print(f"✅ Done: {len(pbxproj_files)} file(s)")

if __name__ == "__main__":
    main()
