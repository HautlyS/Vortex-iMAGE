#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned builds.
Uses safe regex operations that preserve pbxproj format.
"""
import os
import glob
import sys
import re

def patch_pbxproj(path):
    """Patch pbxproj using regex-safe replacements."""
    print(f"Patching: {path}")
    
    with open(path, 'r') as f:
        content = f.read()
    
    original = content
    
    # Replace tauri xcode-script shellScript (handles multiline)
    content = re.sub(
        r'(shellScript\s*=\s*")[^"]*tauri[^"]*(")',
        r'\1echo "Pre-built Rust library"\2',
        content,
        flags=re.IGNORECASE
    )
    
    # Safe key-value replacements
    replacements = [
        (r'CODE_SIGN_IDENTITY\s*=\s*"[^"]*"', 'CODE_SIGN_IDENTITY = ""'),
        (r'CODE_SIGNING_REQUIRED\s*=\s*YES', 'CODE_SIGNING_REQUIRED = NO'),
        (r'CODE_SIGNING_ALLOWED\s*=\s*YES', 'CODE_SIGNING_ALLOWED = NO'),
        (r'CODE_SIGN_STYLE\s*=\s*Automatic', 'CODE_SIGN_STYLE = Manual'),
        (r'ProvisioningStyle\s*=\s*Automatic', 'ProvisioningStyle = Manual'),
        # Clear DEVELOPMENT_TEAM (handles quoted and unquoted values)
        (r'DEVELOPMENT_TEAM\s*=\s*"[^"]*"', 'DEVELOPMENT_TEAM = ""'),
        (r'DEVELOPMENT_TEAM\s*=\s*[A-Z0-9]+;', 'DEVELOPMENT_TEAM = "";'),
        # Switch from debug to release library
        (r'debug/libapp\.a', 'release/libapp.a'),
    ]
    
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)
    
    # Validate: check for obvious syntax issues
    if content.count('{') != content.count('}'):
        print(f"⚠️  Warning: Brace mismatch in {path}")
    
    with open(path, 'w') as f:
        f.write(content)
    
    changes = len(original) != len(content) or original != content
    print(f"✅ Patched: {path} ({'modified' if changes else 'no changes needed'})")

def main():
    pbxproj_files = glob.glob('src-tauri/gen/apple/**/*.pbxproj', recursive=True)
    
    if not pbxproj_files:
        print("⚠️  No pbxproj files found - iOS project may not be initialized yet")
        sys.exit(0)
    
    for pbx in pbxproj_files:
        patch_pbxproj(pbx)
    
    print(f"✅ Patched {len(pbxproj_files)} file(s)")

if __name__ == "__main__":
    main()
