#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned builds.
Uses safe string operations that preserve pbxproj format.
"""
import os
import glob
import sys
import subprocess

def run_plistbuddy(pbxproj_path, command):
    """Run PlistBuddy command on pbxproj file."""
    try:
        result = subprocess.run(
            ['/usr/libexec/PlistBuddy', '-c', command, pbxproj_path],
            capture_output=True, text=True
        )
        return result.returncode == 0
    except Exception:
        return False

def patch_pbxproj(path):
    """Patch pbxproj using line-by-line safe replacements."""
    print(f"Patching: {path}")
    
    with open(path, 'r') as f:
        lines = f.readlines()
    
    new_lines = []
    in_shell_script = False
    brace_count = 0
    
    for line in lines:
        modified = line
        
        # Track shell script blocks to replace tauri xcode-script
        if 'shellScript = "' in line and 'tauri' in line.lower():
            # Replace the entire shellScript line
            indent = len(line) - len(line.lstrip())
            modified = ' ' * indent + 'shellScript = "echo \\"Pre-built Rust library\\"";\n'
            new_lines.append(modified)
            continue
        
        # Safe key-value replacements (preserve semicolons)
        replacements = [
            ('CODE_SIGN_IDENTITY = "Apple Development"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "-"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
            ('CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
            ('CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),
            ('ProvisioningStyle = Automatic', 'ProvisioningStyle = Manual'),
            # Switch from debug to release library
            ('debug/libapp.a', 'release/libapp.a'),
        ]
        
        for old, new in replacements:
            if old in modified:
                modified = modified.replace(old, new)
        
        # Clear DEVELOPMENT_TEAM values (preserve format)
        if 'DEVELOPMENT_TEAM = ' in modified and '""' not in modified:
            # Extract just the key part and set empty value
            if ';' in modified:
                indent = len(modified) - len(modified.lstrip())
                modified = ' ' * indent + 'DEVELOPMENT_TEAM = "";\n'
        
        new_lines.append(modified)
    
    with open(path, 'w') as f:
        f.writelines(new_lines)
    
    print(f"✅ Patched: {path}")

def main():
    pbxproj_files = glob.glob('src-tauri/gen/apple/**/*.pbxproj', recursive=True)
    
    if not pbxproj_files:
        print("⚠️  No pbxproj files found - iOS project may not be initialized yet")
        print("   This is OK if running before 'tauri ios init'")
        sys.exit(0)
    
    for pbx in pbxproj_files:
        patch_pbxproj(pbx)
    
    print(f"✅ Patched {len(pbxproj_files)} file(s)")

if __name__ == "__main__":
    main()
