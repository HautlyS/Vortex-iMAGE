import os
import re
import glob
import sys

def patch_pbxproj(path):
    print(f"Modifying: {path}")
    
    with open(path, 'r') as f:
        content = f.read()

    # 1. Safer handling of TargetAttributes
    # Instead of trying to delete the whole block (which fails on nested braces),
    # we specifically target ProvisioningStyle to ensure it is Manual.
    content = re.sub(r'ProvisioningStyle = Automatic;', 'ProvisioningStyle = Manual;', content)
    
    # 2. Remove SystemCapabilities (triggers signing requirements)
    # This regex is safer because SystemCapabilities usually don't have deeply nested braces
    content = re.sub(r'SystemCapabilities = \{.*?\};', 'SystemCapabilities = {};', content, flags=re.DOTALL)

    replacements = [
        # Remove Signing Identity
        (r'CODE_SIGN_IDENTITY = "[^"]*"', 'CODE_SIGN_IDENTITY = ""'),
        (r'"CODE_SIGN_IDENTITY\[sdk=iphoneos\*\]" = "[^"]*"', '"CODE_SIGN_IDENTITY[sdk=iphoneos*]" = ""'),
        
        # Remove Development Team
        (r'DEVELOPMENT_TEAM = [A-Z0-9]+', 'DEVELOPMENT_TEAM = ""'),
        (r'DEVELOPMENT_TEAM = "[^"]*"', 'DEVELOPMENT_TEAM = ""'),
        
        # Remove Provisioning Profiles
        (r'PROVISIONING_PROFILE_SPECIFIER = "[^"]*"', 'PROVISIONING_PROFILE_SPECIFIER = ""'),
        (r'"PROVISIONING_PROFILE_SPECIFIER\[sdk=iphoneos\*\]" = "[^"]*"', '"PROVISIONING_PROFILE_SPECIFIER[sdk=iphoneos*]" = ""'),
        
        # Disable Code Signing Requirements
        (r'CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
        (r'CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
        
        # FIX: Disable Ad-Hoc signing for unsigned builds
        (r'AD_HOC_CODE_SIGNING_ALLOWED = YES', 'AD_HOC_CODE_SIGNING_ALLOWED = NO'),
        
        # Force Manual Signing Style (handles cases outside TargetAttributes too)
        (r'CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),

        # Disable Provisioning Profile Requirement
        (r'PROVISIONING_PROFILE_REQUIRED = YES', 'PROVISIONING_PROFILE_REQUIRED = NO'),

        # FIX: Force Tauri script to run in Release mode (prevents "missing addr file" panic)
        # Matches --configuration ${CONFIGURATION}, "${CONFIGURATION}", \"${CONFIGURATION}\", ${CONFIGURATION:?} etc.
        (r'--configuration\s*(?:[\\"]*)\$\{CONFIGURATION[^}]*\}(?:[\\"]*)', '--configuration Release'),




    ]

    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)

    # Force inject settings if missing
    if 'CODE_SIGNING_REQUIRED' not in content:
        content = content.replace('buildSettings = {', 'buildSettings = {\n\t\t\t\tCODE_SIGNING_REQUIRED = NO;')
    if 'CODE_SIGNING_ALLOWED' not in content:
        content = content.replace('buildSettings = {', 'buildSettings = {\n\t\t\t\tCODE_SIGNING_ALLOWED = NO;')
        
    with open(path, 'w') as f:
        f.write(content)
    print("Successfully patched pbxproj")

def main():
    pbxproj_files = glob.glob('src-tauri/gen/apple/**/*.pbxproj', recursive=True)
    if not pbxproj_files:
        print("ERROR: No pbxproj found")
        sys.exit(1)
        
    for pbx in pbxproj_files:
        patch_pbxproj(pbx)

if __name__ == "__main__":
    main()