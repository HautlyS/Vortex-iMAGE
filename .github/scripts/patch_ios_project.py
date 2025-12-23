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
        # Regex explanation:
        # --configuration\s+   : Matches "--configuration" followed by whitespace
        # (?:[\\"]*)           : Non-capturing group for optional quotes/escaped quotes
        # (?:\$)?              : Optional "$" (for $CONFIGURATION case)
        # (?:\{)?              : Optional "{"
        # CONFIGURATION        : The literal string "CONFIGURATION"
        # .*?                  : Any suffixes like ":?"
        # (?:\}|(?=\s)|(?=[\\"])) : End of variable (closing brace, or lookahead for space/quote)
        (r'--configuration\s+(?:[\\"]*)(?:\$)?(?:\{)?CONFIGURATION.*?(?:\}|(?=\s)|(?=[\\"]))', '--configuration Release'),

        # FIX: Remove DEBUG=1 from GCC_PREPROCESSOR_DEFINITIONS passed to the script
        # This prevents tauri-cli from thinking it's a debug build even if configuration is Release.
        # Matches --gcc-preprocessor-definitions ... ${GCC_PREPROCESSOR_DEFINITIONS...} ...
        # NOTE: We must use escaped quotes (\\"\\") because this string is inserted into a quoted shellScript in pbxproj.
        (r'--gcc-preprocessor-definitions\s+(?:[\\"]*)(?:\$)?(?:\{)?GCC_PREPROCESSOR_DEFINITIONS.*?(?:\}|(?=\s)|(?=[\\"]))', r'--gcc-preprocessor-definitions \\"\\"'),


    ]

    # Apply regex replacements
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)

    # FIX 2: Hardcoded Path Replacement
    # Some generated projects have hardcoded references to "debug/libapp.a" even in Release schemes.
    # We forcefully redirect them to the release artifacts we just built.
    print("Patching hardcoded debug paths...")
    content = content.replace('debug/libapp.a', 'release/libapp.a')
    content = content.replace('debug/libvortex_image_lib.a', 'release/libvortex_image_lib.a')


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