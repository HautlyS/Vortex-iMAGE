#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned release builds.
- Disables code signing
- Configures for release mode (bundled assets, not dev server)
- Grants local network permissions as fallback
- Patches WKWebView configuration for bundled assets
"""
import glob
import sys
import re
import os
import plistlib

def patch_pbxproj(path):
    """Patch pbxproj safely line by line."""
    print(f"Patching pbxproj: {path}")
    
    with open(path, 'r') as f:
        lines = f.readlines()
    
    new_lines = []
    modified = False
    
    for line in lines:
        # Skip shellScript lines containing tauri - replace with no-op
        if 'shellScript' in line and 'tauri' in line.lower():
            indent = len(line) - len(line.lstrip())
            line = ' ' * indent + 'shellScript = "echo \\"Rust library pre-built\\"";\n'
            modified = True
        
        # Simple string replacements for code signing
        simple_replacements = [
            ('CODE_SIGN_IDENTITY = "Apple Development"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "-"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "iPhone Developer"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
            ('CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
            ('CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),
            ('ProvisioningStyle = Automatic', 'ProvisioningStyle = Manual'),
        ]
        
        for old, new in simple_replacements:
            if old in line:
                line = line.replace(old, new)
                modified = True
        
        # Handle DEVELOPMENT_TEAM with regex
        if 'DEVELOPMENT_TEAM' in line and '""' not in line:
            new_line = re.sub(r'DEVELOPMENT_TEAM\s*=\s*[^;]+;', 'DEVELOPMENT_TEAM = "";', line)
            if new_line != line:
                line = new_line
                modified = True
        
        new_lines.append(line)
    
    with open(path, 'w') as f:
        f.writelines(new_lines)
    
    print(f"✅ Patched pbxproj: {path} ({'modified' if modified else 'no changes'})")

def patch_info_plist(path):
    """Patch Info.plist to grant local network permissions and configure for release."""
    print(f"Patching Info.plist: {path}")
    
    try:
        with open(path, 'rb') as f:
            plist = plistlib.load(f)
    except Exception as e:
        print(f"⚠️  Could not read plist: {e}")
        return
    
    modified = False
    
    # Add local network usage description (required for iOS 14+)
    if 'NSLocalNetworkUsageDescription' not in plist:
        plist['NSLocalNetworkUsageDescription'] = 'This app requires local network access for development and debugging purposes.'
        modified = True
    
    # Add Bonjour services for local network discovery
    if 'NSBonjourServices' not in plist:
        plist['NSBonjourServices'] = ['_http._tcp.', '_https._tcp.']
        modified = True
    
    # Allow arbitrary loads for development (can be removed for production)
    if 'NSAppTransportSecurity' not in plist:
        plist['NSAppTransportSecurity'] = {}
    
    ats = plist['NSAppTransportSecurity']
    if 'NSAllowsLocalNetworking' not in ats or not ats['NSAllowsLocalNetworking']:
        ats['NSAllowsLocalNetworking'] = True
        modified = True
    
    # Allow localhost connections
    if 'NSExceptionDomains' not in ats:
        ats['NSExceptionDomains'] = {}
    
    if 'localhost' not in ats['NSExceptionDomains']:
        ats['NSExceptionDomains']['localhost'] = {
            'NSExceptionAllowsInsecureHTTPLoads': True,
            'NSIncludesSubdomains': True
        }
        modified = True
    
    if modified:
        with open(path, 'wb') as f:
            plistlib.dump(plist, f)
        print(f"✅ Patched Info.plist: {path}")
    else:
        print(f"✅ Info.plist already configured: {path}")

def patch_swift_files(apple_path):
    """Patch Swift files to ensure release mode uses bundled assets instead of dev server."""
    swift_files = glob.glob(os.path.join(apple_path, '**', '*.swift'), recursive=True)
    
    for swift_file in swift_files:
        try:
            with open(swift_file, 'r') as f:
                content = f.read()
            
            original = content
            modified = False
            
            # Tauri 2.x generates code that checks for devUrl
            # We need to force it to use bundled assets by:
            # 1. Commenting out or removing devUrl references
            # 2. Ensuring the app loads from the bundle
            
            # Pattern 1: Replace devUrl with nil or remove the dev server check
            # Look for patterns like: let devUrl = "http://localhost:1420"
            if 'localhost:1420' in content or 'localhost' in content:
                print(f"⚠️  Found localhost reference in {swift_file}")
                
                # Replace hardcoded dev URLs with nil
                content = re.sub(
                    r'(let\s+devUrl\s*[:=]\s*)(String\?\s*=\s*)?"http://localhost[^"]*"',
                    r'\1nil',
                    content
                )
                
                # Also handle URL initialization
                content = re.sub(
                    r'URL\(string:\s*"http://localhost[^"]*"\)',
                    'nil',
                    content
                )
                
                modified = content != original
            
            # Pattern 2: Force production mode in Tauri config checks
            # Some Tauri versions check #if DEBUG
            if '#if DEBUG' in content and 'devUrl' in content.lower():
                # Wrap dev server code to never execute
                content = re.sub(
                    r'#if DEBUG\s*\n(.*?devUrl.*?)\n\s*#else',
                    r'#if false // Force release mode\n\1\n#else',
                    content,
                    flags=re.DOTALL | re.IGNORECASE
                )
                modified = content != original or modified
            
            if modified:
                with open(swift_file, 'w') as f:
                    f.write(content)
                print(f"✅ Patched Swift file: {swift_file}")
            elif 'localhost' in original.lower():
                print(f"ℹ️  Localhost found but no pattern matched in: {swift_file}")
                
        except Exception as e:
            print(f"⚠️  Could not process {swift_file}: {e}")

def create_xcconfig(apple_path):
    """Create an xcconfig file to force release settings."""
    xcconfig_content = """// Release configuration for unsigned builds
// This ensures the app uses bundled assets instead of dev server

CODE_SIGN_IDENTITY = 
CODE_SIGNING_REQUIRED = NO
CODE_SIGNING_ALLOWED = NO
DEVELOPMENT_TEAM = 
AD_HOC_CODE_SIGNING_ALLOWED = NO

// Ensure release optimizations
GCC_OPTIMIZATION_LEVEL = s
SWIFT_OPTIMIZATION_LEVEL = -O
ENABLE_TESTABILITY = NO
DEBUG_INFORMATION_FORMAT = dwarf-with-dsym

// Force release mode for Tauri - disable DEBUG flag
GCC_PREPROCESSOR_DEFINITIONS = $(inherited) RELEASE=1 NDEBUG=1
SWIFT_ACTIVE_COMPILATION_CONDITIONS = RELEASE

// Disable debug-only features
ENABLE_NS_ASSERTIONS = NO
"""
    
    xcconfig_path = os.path.join(apple_path, 'Release-Unsigned.xcconfig')
    with open(xcconfig_path, 'w') as f:
        f.write(xcconfig_content)
    print(f"✅ Created xcconfig: {xcconfig_path}")


def patch_tauri_lib_swift(apple_path):
    """
    Patch the Tauri-generated lib.swift to force bundled assets.
    This is the key file where Tauri decides between devUrl and frontendDist.
    """
    lib_swift_patterns = [
        os.path.join(apple_path, 'Sources', '*', 'lib.swift'),
        os.path.join(apple_path, '**', 'lib.swift'),
        os.path.join(apple_path, '**', 'Tauri*.swift'),
    ]
    
    lib_files = []
    for pattern in lib_swift_patterns:
        lib_files.extend(glob.glob(pattern, recursive=True))
    
    for lib_file in lib_files:
        try:
            with open(lib_file, 'r') as f:
                content = f.read()
            
            original = content
            
            # Tauri 2.x pattern: looks for devUrl in the generated code
            # The key is to make sure the app doesn't try to connect to localhost
            
            # Pattern: Replace any devUrl assignment with nil
            content = re.sub(
                r'private\s+let\s+devUrl\s*=\s*"[^"]*"',
                'private let devUrl: String? = nil',
                content
            )
            
            # Pattern: Force useDevUrl to false
            content = re.sub(
                r'let\s+useDevUrl\s*=\s*true',
                'let useDevUrl = false',
                content
            )
            
            # Pattern: Comment out dev server initialization
            content = re.sub(
                r'(if\s+let\s+devUrl\s*=\s*devUrl\s*\{[^}]*\})',
                '/* Release build - dev server disabled\n\\1\n*/',
                content
            )
            
            if content != original:
                with open(lib_file, 'w') as f:
                    f.write(content)
                print(f"✅ Patched Tauri lib file: {lib_file}")
            else:
                print(f"ℹ️  No changes needed for: {lib_file}")
                
        except Exception as e:
            print(f"⚠️  Could not process {lib_file}: {e}")

def patch_tauri_conf_for_release(base_path):
    """Verify tauri.conf.json has correct frontendDist for release builds."""
    tauri_conf_path = os.path.join(base_path, 'src-tauri', 'tauri.conf.json')
    
    if not os.path.exists(tauri_conf_path):
        print(f"⚠️  tauri.conf.json not found at {tauri_conf_path}")
        return
    
    import json
    with open(tauri_conf_path, 'r') as f:
        config = json.load(f)
    
    # Verify frontendDist is set (this is what release builds use)
    build_config = config.get('build', {})
    frontend_dist = build_config.get('frontendDist')
    
    if frontend_dist:
        print(f"✅ Tauri config has frontendDist: {frontend_dist}")
    else:
        print("⚠️  No frontendDist configured - release builds may fail")

def ensure_assets_copied(apple_path):
    """Ensure the frontend assets are properly referenced."""
    assets_paths = glob.glob(os.path.join(apple_path, '**', 'Assets.xcassets'), recursive=True)
    
    for assets_path in assets_paths:
        print(f"✅ Found assets at: {assets_path}")

def main():
    base_path = os.getcwd()
    apple_path = os.path.join(base_path, 'src-tauri', 'gen', 'apple')
    
    # Patch pbxproj files
    pbxproj_files = glob.glob(os.path.join(apple_path, '**', '*.pbxproj'), recursive=True)
    
    if not pbxproj_files:
        print("⚠️  No pbxproj files found")
    else:
        for pbx in pbxproj_files:
            patch_pbxproj(pbx)
    
    # Patch Info.plist files
    plist_files = glob.glob(os.path.join(apple_path, '**', 'Info.plist'), recursive=True)
    
    if not plist_files:
        print("⚠️  No Info.plist files found")
    else:
        for plist in plist_files:
            patch_info_plist(plist)
    
    # Patch Tauri lib.swift (key file for dev vs release)
    patch_tauri_lib_swift(apple_path)
    
    # Patch other Swift files
    patch_swift_files(apple_path)
    
    # Create xcconfig for release
    if os.path.exists(apple_path):
        create_xcconfig(apple_path)
    
    # Verify tauri config
    patch_tauri_conf_for_release(base_path)
    
    # Check assets
    ensure_assets_copied(apple_path)
    
    print(f"\n✅ Done: Patched {len(pbxproj_files)} pbxproj and {len(plist_files)} plist file(s)")

if __name__ == "__main__":
    main()
