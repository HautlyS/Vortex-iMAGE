#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned release builds.

Key fixes:
1. Disable code signing for CI builds
2. Keep the Rust build script intact (don't break it!)
3. Update library search paths to include both debug and release
4. Configure for Release mode
"""
import glob
import sys
import re
import os
import plistlib
import shutil

def patch_pbxproj(path):
    """Patch pbxproj for unsigned release builds."""
    print(f"Patching pbxproj: {path}")
    
    with open(path, 'r') as f:
        content = f.read()
    
    original = content
    
    # 1. Disable code signing (but DON'T touch the shell scripts!)
    signing_replacements = [
        ('CODE_SIGN_IDENTITY = "Apple Development"', 'CODE_SIGN_IDENTITY = ""'),
        ('CODE_SIGN_IDENTITY = "-"', 'CODE_SIGN_IDENTITY = ""'),
        ('CODE_SIGN_IDENTITY = "iPhone Developer"', 'CODE_SIGN_IDENTITY = ""'),
        ('CODE_SIGN_IDENTITY = "iPhone Distribution"', 'CODE_SIGN_IDENTITY = ""'),
        ('CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
        ('CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
        ('CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),
        ('ProvisioningStyle = Automatic', 'ProvisioningStyle = Manual'),
    ]
    
    for old, new in signing_replacements:
        content = content.replace(old, new)
    
    # Clear DEVELOPMENT_TEAM values
    content = re.sub(r'DEVELOPMENT_TEAM\s*=\s*[A-Z0-9]+;', 'DEVELOPMENT_TEAM = "";', content)
    
    # 2. Ensure LIBRARY_SEARCH_PATHS includes both debug and release
    # This is critical for the linker to find libapp.a
    # Pattern: Add release path if only debug exists
    if 'Externals/arm64/debug' in content and 'Externals/arm64/release' not in content:
        content = content.replace(
            'Externals/arm64/debug',
            'Externals/arm64/debug",\n\t\t\t\t\t"$(SRCROOT)/Externals/arm64/release'
        )
        print("  Added release path to LIBRARY_SEARCH_PATHS")
    
    # Also handle $(CURRENT_ARCH) variant
    if 'Externals/$(CURRENT_ARCH)/debug' in content:
        content = content.replace(
            'Externals/$(CURRENT_ARCH)/debug',
            'Externals/$(CURRENT_ARCH)/debug",\n\t\t\t\t\t"$(SRCROOT)/Externals/$(CURRENT_ARCH)/release'
        )
    
    if content != original:
        with open(path, 'w') as f:
            f.write(content)
        print(f"  Patched: {path}")
        return True
    else:
        print(f"  No changes needed: {path}")
        return False

def patch_info_plist(path):
    """Patch Info.plist - minimal changes for release."""
    print(f"Checking Info.plist: {path}")
    
    try:
        with open(path, 'rb') as f:
            plist = plistlib.load(f)
    except Exception as e:
        print(f"  Could not read plist: {e}")
        return False
    
    modified = False
    
    if 'NSAppTransportSecurity' not in plist:
        plist['NSAppTransportSecurity'] = {}
        modified = True
    
    ats = plist['NSAppTransportSecurity']
    if not ats.get('NSAllowsLocalNetworking'):
        ats['NSAllowsLocalNetworking'] = True
        modified = True
    
    if modified:
        with open(path, 'wb') as f:
            plistlib.dump(plist, f)
        print(f"  Patched: {path}")
    else:
        print(f"  Already configured: {path}")
    
    return modified

def update_xcscheme_for_release(apple_path):
    """Update xcscheme files to use Release configuration."""
    scheme_files = glob.glob(os.path.join(apple_path, '**', '*.xcscheme'), recursive=True)
    
    for scheme_file in scheme_files:
        try:
            with open(scheme_file, 'r') as f:
                content = f.read()
            
            original = content
            
            content = re.sub(
                r'(<ArchiveAction[^>]*buildConfiguration\s*=\s*)"Debug"',
                r'\1"Release"',
                content
            )
            
            if content != original:
                with open(scheme_file, 'w') as f:
                    f.write(content)
                print(f"  Updated scheme: {os.path.basename(scheme_file)}")
        except Exception as e:
            print(f"  Could not process scheme: {e}")

def ensure_library_in_both_locations(apple_path):
    """Ensure libapp.a exists in both debug and release directories."""
    externals_base = os.path.join(apple_path, 'Externals', 'arm64')
    release_lib = os.path.join(externals_base, 'release', 'libapp.a')
    debug_lib = os.path.join(externals_base, 'debug', 'libapp.a')
    
    source_lib = None
    if os.path.exists(release_lib):
        source_lib = release_lib
        print(f"  Found library in release: {release_lib}")
    elif os.path.exists(debug_lib):
        source_lib = debug_lib
        print(f"  Found library in debug: {debug_lib}")
    
    if source_lib:
        for target_dir in [os.path.dirname(release_lib), os.path.dirname(debug_lib)]:
            os.makedirs(target_dir, exist_ok=True)
            target_lib = os.path.join(target_dir, 'libapp.a')
            if not os.path.exists(target_lib):
                shutil.copy2(source_lib, target_lib)
                print(f"  Copied library to: {target_lib}")
    else:
        print("  WARNING: No libapp.a found in Externals")

def main():
    base_path = os.getcwd()
    apple_path = os.path.join(base_path, 'src-tauri', 'gen', 'apple')
    
    if not os.path.exists(apple_path):
        print(f"Apple path not found: {apple_path}")
        print("Run 'pnpm tauri ios init' first")
        sys.exit(1)
    
    print(f"Patching iOS project at: {apple_path}")
    print("=" * 50)
    
    pbxproj_files = glob.glob(os.path.join(apple_path, '**', '*.pbxproj'), recursive=True)
    print(f"\nFound {len(pbxproj_files)} pbxproj file(s)")
    for pbx in pbxproj_files:
        patch_pbxproj(pbx)
    
    plist_files = glob.glob(os.path.join(apple_path, '**', 'Info.plist'), recursive=True)
    print(f"\nFound {len(plist_files)} Info.plist file(s)")
    for plist in plist_files:
        patch_info_plist(plist)
    
    print("\nUpdating xcscheme files...")
    update_xcscheme_for_release(apple_path)
    
    print("\nChecking library locations...")
    ensure_library_in_both_locations(apple_path)
    
    print("\n" + "=" * 50)
    print("Patching complete!")

if __name__ == "__main__":
    main()
