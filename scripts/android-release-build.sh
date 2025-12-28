#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# iMAGE - Android Signed Release APK Build Script
# Builds a production-ready, signed APK for distribution
#
# Usage:
#   Interactive:     ./android-release-build.sh build
#   Non-interactive: KEYSTORE_PASSWORD=mypass ./android-release-build.sh build
# ═══════════════════════════════════════════════════════════════════════════

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Project paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ANDROID_PROJECT="$PROJECT_ROOT/src-tauri/gen/android"
KEYSTORE_DIR="$PROJECT_ROOT/.keystore"
KEYSTORE_FILE="$KEYSTORE_DIR/release.keystore"
KEYSTORE_PROPS="$ANDROID_PROJECT/app/keystore.properties"

# Emulator configuration
ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
EMULATOR_DIR="$PROJECT_ROOT/.android-emulator"
CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
SYSTEM_IMAGE="system-images;android-34;google_apis;x86_64"
AVD_NAME="iMAGE_Test_Device"

# Configuration from environment (for non-interactive/CI mode)
KEY_ALIAS="${KEY_ALIAS:-image-release}"
KEYSTORE_VALIDITY="${KEYSTORE_VALIDITY:-10000}"
KEYSTORE_PASSWORD="${KEYSTORE_PASSWORD:-}"
KEY_PASSWORD="${KEY_PASSWORD:-$KEYSTORE_PASSWORD}"
CERT_CN="${CERT_CN:-iMAGE Developer}"
CERT_OU="${CERT_OU:-Development}"
CERT_O="${CERT_O:-iMAGE}"
CERT_L="${CERT_L:-}"
CERT_ST="${CERT_ST:-}"
CERT_C="${CERT_C:-US}"

# Global APK path
BUILT_APK_PATH=""

print_banner() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║        iMAGE - Android Signed Release Build Script            ║"
    echo "║                   Production APK Builder                      ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

is_interactive() {
    [ -t 0 ] && [ -z "$KEYSTORE_PASSWORD" ]
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing=()
    
    command -v keytool >/dev/null 2>&1 || missing+=("keytool (Java JDK)")
    command -v pnpm >/dev/null 2>&1 || missing+=("pnpm")
    
    if [ ! -d "$ANDROID_PROJECT" ]; then
        log_error "Android project not initialized. Run first:"
        echo "  pnpm tauri android init"
        exit 1
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing: ${missing[*]}"
        exit 1
    fi
    
    log_success "Prerequisites OK"
}

generate_keystore() {
    log_info "Setting up release keystore..."
    
    mkdir -p "$KEYSTORE_DIR"
    
    if [ -f "$KEYSTORE_FILE" ]; then
        log_info "Keystore already exists at: $KEYSTORE_FILE"
        if is_interactive; then
            read -p "Use existing keystore? [Y/n]: " use_existing
            if [[ "$use_existing" =~ ^[Nn] ]]; then
                rm -f "$KEYSTORE_FILE"
            else
                return 0
            fi
        else
            log_info "Using existing keystore (non-interactive mode)"
            return 0
        fi
    fi
    
    log_info "Creating new release keystore..."
    
    local STORE_PASSWORD="$KEYSTORE_PASSWORD"
    local FINAL_KEY_PASSWORD="$KEY_PASSWORD"
    
    if is_interactive; then
        echo -e "${YELLOW}You'll need to provide the following information:${NC}"
        echo ""
        
        while true; do
            read -sp "Keystore password (min 6 chars): " STORE_PASSWORD
            echo ""
            if [ ${#STORE_PASSWORD} -lt 6 ]; then
                log_error "Password must be at least 6 characters"
                continue
            fi
            read -sp "Confirm password: " STORE_PASSWORD_CONFIRM
            echo ""
            if [ "$STORE_PASSWORD" != "$STORE_PASSWORD_CONFIRM" ]; then
                log_error "Passwords don't match"
                continue
            fi
            break
        done
        
        read -p "Use same password for key? [Y/n]: " same_password
        if [[ "$same_password" =~ ^[Nn] ]]; then
            while true; do
                read -sp "Key password (min 6 chars): " FINAL_KEY_PASSWORD
                echo ""
                if [ ${#FINAL_KEY_PASSWORD} -lt 6 ]; then
                    log_error "Password must be at least 6 characters"
                    continue
                fi
                read -sp "Confirm key password: " KEY_PASSWORD_CONFIRM
                echo ""
                if [ "$FINAL_KEY_PASSWORD" != "$KEY_PASSWORD_CONFIRM" ]; then
                    log_error "Passwords don't match"
                    continue
                fi
                break
            done
        else
            FINAL_KEY_PASSWORD="$STORE_PASSWORD"
        fi
        
        echo ""
        read -p "Your name or organization [$CERT_CN]: " input_cn
        [ -n "$input_cn" ] && CERT_CN="$input_cn"
        read -p "Organization unit [$CERT_OU]: " input_ou
        [ -n "$input_ou" ] && CERT_OU="$input_ou"
        read -p "Organization [$CERT_O]: " input_o
        [ -n "$input_o" ] && CERT_O="$input_o"
        read -p "City [$CERT_L]: " input_l
        [ -n "$input_l" ] && CERT_L="$input_l"
        read -p "State/Province [$CERT_ST]: " input_st
        [ -n "$input_st" ] && CERT_ST="$input_st"
        read -p "Country code (2 letters) [$CERT_C]: " input_c
        [ -n "$input_c" ] && CERT_C="$input_c"
    else
        if [ -z "$STORE_PASSWORD" ]; then
            log_error "KEYSTORE_PASSWORD environment variable required for non-interactive mode"
            exit 1
        fi
        if [ ${#STORE_PASSWORD} -lt 6 ]; then
            log_error "KEYSTORE_PASSWORD must be at least 6 characters"
            exit 1
        fi
        [ -z "$FINAL_KEY_PASSWORD" ] && FINAL_KEY_PASSWORD="$STORE_PASSWORD"
        log_info "Using environment variables for keystore configuration"
    fi
    
    local DNAME="CN=$CERT_CN, OU=$CERT_OU, O=$CERT_O, L=$CERT_L, ST=$CERT_ST, C=$CERT_C"
    
    log_info "Generating keystore..."
    
    keytool -genkeypair \
        -v \
        -keystore "$KEYSTORE_FILE" \
        -alias "$KEY_ALIAS" \
        -keyalg RSA \
        -keysize 2048 \
        -validity "$KEYSTORE_VALIDITY" \
        -storepass "$STORE_PASSWORD" \
        -keypass "$FINAL_KEY_PASSWORD" \
        -dname "$DNAME"
    
    log_success "Keystore created: $KEYSTORE_FILE"
    save_keystore_properties "$STORE_PASSWORD" "$FINAL_KEY_PASSWORD"
}

save_keystore_properties() {
    local store_pass="$1"
    local key_pass="$2"
    
    cat > "$KEYSTORE_PROPS" << EOF
storeFile=$KEYSTORE_FILE
storePassword=$store_pass
keyAlias=$KEY_ALIAS
keyPassword=$key_pass
EOF
    
    chmod 600 "$KEYSTORE_PROPS"
    log_info "Keystore properties saved"
    
    local gitignore="$ANDROID_PROJECT/.gitignore"
    if [ -f "$gitignore" ] && ! grep -q "keystore.properties" "$gitignore" 2>/dev/null; then
        echo "keystore.properties" >> "$gitignore"
    fi
    
    local root_gitignore="$PROJECT_ROOT/.gitignore"
    if [ -f "$root_gitignore" ] && ! grep -q ".keystore/" "$root_gitignore" 2>/dev/null; then
        echo "" >> "$root_gitignore"
        echo "# Android signing keystore" >> "$root_gitignore"
        echo ".keystore/" >> "$root_gitignore"
    fi
}

load_keystore_properties() {
    if [ -f "$KEYSTORE_PROPS" ] && [ -f "$KEYSTORE_FILE" ]; then
        log_info "Loading existing keystore configuration..."
        return 0
    fi
    return 1
}

configure_signing() {
    log_info "Configuring Gradle signing..."
    
    local build_gradle="$ANDROID_PROJECT/app/build.gradle.kts"
    
    if grep -q "signingConfigs" "$build_gradle"; then
        log_info "Signing configuration already present"
        return 0
    fi
    
    cp "$build_gradle" "$build_gradle.backup"
    
    local temp_file=$(mktemp)
    
    awk '
    /^android \{/ {
        print
        print ""
        print "    // Release signing configuration"
        print "    val keystorePropertiesFile = rootProject.file(\"app/keystore.properties\")"
        print "    val keystoreProperties = Properties()"
        print "    if (keystorePropertiesFile.exists()) {"
        print "        keystoreProperties.load(keystorePropertiesFile.inputStream())"
        print "    }"
        print ""
        print "    signingConfigs {"
        print "        create(\"release\") {"
        print "            if (keystorePropertiesFile.exists()) {"
        print "                storeFile = file(keystoreProperties[\"storeFile\"] as String)"
        print "                storePassword = keystoreProperties[\"storePassword\"] as String"
        print "                keyAlias = keystoreProperties[\"keyAlias\"] as String"
        print "                keyPassword = keystoreProperties[\"keyPassword\"] as String"
        print "            }"
        print "        }"
        print "    }"
        print ""
        next
    }
    { print }
    ' "$build_gradle" > "$temp_file"
    
    mv "$temp_file" "$build_gradle"
    sed -i 's/getByName("release") {/getByName("release") {\n            signingConfig = signingConfigs.getByName("release")/' "$build_gradle"
    
    log_success "Gradle signing configured"
}

build_release() {
    log_info "Building signed release APK..."
    
    log_info "Cleaning previous builds..."
    rm -rf "$ANDROID_PROJECT/app/build/outputs/apk/release" 2>/dev/null || true
    rm -rf "$ANDROID_PROJECT/app/build/outputs/apk/universal/release" 2>/dev/null || true
    
    log_info "Building frontend..."
    pnpm --dir "$PROJECT_ROOT" run build
    
    log_info "Building Android release APK (this may take a few minutes)..."
    pnpm --dir "$PROJECT_ROOT" tauri android build
    
    log_success "Release build complete!"
    
    # Find and store APK path
    local apk_dir="$ANDROID_PROJECT/app/build/outputs/apk"
    BUILT_APK_PATH=$(find "$apk_dir" -name "*.apk" -path "*/release/*" 2>/dev/null | head -1)
}

verify_apk() {
    log_info "Verifying APK signature..."
    
    local apk_dir="$ANDROID_PROJECT/app/build/outputs/apk"
    local apk_file=$(find "$apk_dir" -name "*.apk" -path "*/release/*" 2>/dev/null | head -1)
    
    if [ -z "$apk_file" ]; then
        log_warning "No release APK found to verify"
        return 0
    fi
    
    # Try apksigner from Android SDK first (preferred)
    local apksigner="$ANDROID_HOME/build-tools/34.0.0/apksigner"
    if [ ! -f "$apksigner" ]; then
        # Try to find any apksigner
        apksigner=$(find "$ANDROID_HOME/build-tools" -name "apksigner" 2>/dev/null | head -1)
    fi
    
    if [ -n "$apksigner" ] && [ -f "$apksigner" ]; then
        if "$apksigner" verify --verbose "$apk_file" 2>&1 | grep -q "Verifies"; then
            log_success "APK signature verified (APK Signature Scheme v2)"
        else
            log_warning "APK signature verification failed"
            return 1
        fi
    elif command -v apksigner >/dev/null 2>&1; then
        if apksigner verify --verbose "$apk_file" 2>&1 | grep -q "Verifies"; then
            log_success "APK signature verified"
        else
            log_warning "APK signature verification failed"
            return 1
        fi
    else
        log_warning "apksigner not found, skipping verification"
    fi
}

show_output() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}                    BUILD SUCCESSFUL!                          ${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local apk_dir="$ANDROID_PROJECT/app/build/outputs/apk"
    local universal_dir="$apk_dir/universal/release"
    local release_dir="$apk_dir/release"
    
    echo -e "${CYAN}APK Output Locations:${NC}"
    
    if [ -d "$universal_dir" ]; then
        echo ""
        echo "Universal APK (recommended for distribution):"
        find "$universal_dir" -name "*.apk" -exec ls -lh {} \; 2>/dev/null
    fi
    
    if [ -d "$release_dir" ]; then
        echo ""
        echo "Architecture-specific APKs:"
        find "$release_dir" -name "*.apk" -exec ls -lh {} \; 2>/dev/null
    fi
    
    echo ""
    echo -e "${YELLOW}Important:${NC}"
    echo "  • Keep your keystore file safe: $KEYSTORE_FILE"
    echo "  • Back up keystore.properties securely"
    echo "  • You'll need the same keystore for future updates"
    echo ""
}


# ═══════════════════════════════════════════════════════════════════════════
# POST-BUILD OPTIONS: ADB Install & Emulator
# ═══════════════════════════════════════════════════════════════════════════

setup_android_sdk() {
    log_info "Setting up Android SDK for emulator..."
    
    mkdir -p "$ANDROID_HOME/cmdline-tools"
    
    if [ ! -d "$ANDROID_HOME/cmdline-tools/latest" ]; then
        log_info "Downloading Android command line tools..."
        local temp_zip="/tmp/cmdline-tools.zip"
        curl -L "$CMDLINE_TOOLS_URL" -o "$temp_zip"
        unzip -q "$temp_zip" -d "$ANDROID_HOME/cmdline-tools"
        mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        rm "$temp_zip"
        log_success "Command line tools installed"
    fi
    
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    # Accept licenses
    log_info "Accepting Android SDK licenses..."
    yes | sdkmanager --licenses >/dev/null 2>&1 || true
    
    # Install required components for emulator
    log_info "Installing emulator components (this may take a while)..."
    sdkmanager --install \
        "platform-tools" \
        "emulator" \
        "platforms;android-34" \
        "$SYSTEM_IMAGE" 2>&1 | grep -E "(Downloading|Installing|done)" || true
    
    log_success "Android SDK setup complete"
}

create_avd() {
    log_info "Creating Android Virtual Device: $AVD_NAME"
    
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    # Check if AVD already exists
    if avdmanager list avd 2>/dev/null | grep -q "$AVD_NAME"; then
        log_info "AVD '$AVD_NAME' already exists"
        return 0
    fi
    
    # Create AVD
    echo "no" | avdmanager create avd \
        -n "$AVD_NAME" \
        -k "$SYSTEM_IMAGE" \
        -d "pixel_6" \
        --force 2>/dev/null
    
    log_success "AVD created: $AVD_NAME"
}

start_emulator() {
    log_info "Starting Android emulator..."
    
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    export ANDROID_EMULATOR_HOME="$HOME/.android"
    
    # Check if emulator is already running
    if adb devices 2>/dev/null | grep -q "emulator"; then
        log_info "Emulator already running"
        return 0
    fi
    
    # Start emulator in background
    log_info "Launching emulator (this may take a minute)..."
    nohup "$ANDROID_HOME/emulator/emulator" -avd "$AVD_NAME" -gpu auto -no-snapshot-save > /tmp/emulator.log 2>&1 &
    
    # Wait for emulator to boot
    log_info "Waiting for emulator to boot..."
    local timeout=120
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        if adb shell getprop sys.boot_completed 2>/dev/null | grep -q "1"; then
            log_success "Emulator booted successfully"
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
        echo -ne "\r  Waiting... ${elapsed}s / ${timeout}s"
    done
    
    echo ""
    log_error "Emulator boot timeout. Check /tmp/emulator.log for details"
    return 1
}

install_to_device() {
    local apk_file="$1"
    
    if [ -z "$apk_file" ] || [ ! -f "$apk_file" ]; then
        log_error "APK file not found"
        return 1
    fi
    
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    # Check for connected devices
    local devices=$(adb devices 2>/dev/null | grep -E "device$|emulator" | wc -l)
    
    if [ "$devices" -eq 0 ]; then
        log_error "No Android device or emulator connected"
        echo "  Connect a device via USB or start an emulator first"
        return 1
    fi
    
    log_info "Installing APK to device..."
    adb install -r "$apk_file"
    
    log_success "APK installed successfully!"
    
    # Launch the app
    log_info "Launching iMAGE..."
    adb shell am start -n "com.vortex.image/.MainActivity" 2>/dev/null || \
    adb shell monkey -p com.vortex.image -c android.intent.category.LAUNCHER 1 2>/dev/null
    
    log_success "App launched!"
}

post_build_menu() {
    if ! is_interactive; then
        return 0
    fi
    
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}                    What would you like to do?                 ${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  1) Install to connected device (via ADB)"
    echo "  2) Run in Android emulator (auto-setup if needed)"
    echo "  3) Exit (done)"
    echo ""
    
    read -p "Choose an option [1-3]: " choice
    
    case "$choice" in
        1)
            install_to_connected_device
            ;;
        2)
            run_in_emulator
            ;;
        3|"")
            log_info "Done! Your APK is ready for distribution."
            ;;
        *)
            log_warning "Invalid option. Exiting."
            ;;
    esac
}

install_to_connected_device() {
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    # Check if adb is available
    if ! command -v adb >/dev/null 2>&1; then
        log_warning "ADB not found. Setting up Android SDK..."
        setup_android_sdk
    fi
    
    # Check for connected devices
    local devices=$(adb devices 2>/dev/null | grep -E "device$" | grep -v "emulator" | wc -l)
    
    if [ "$devices" -eq 0 ]; then
        log_error "No physical device connected"
        echo ""
        echo "To connect a device:"
        echo "  1. Enable Developer Options on your Android device"
        echo "  2. Enable USB Debugging in Developer Options"
        echo "  3. Connect device via USB"
        echo "  4. Accept the debugging prompt on your device"
        echo ""
        read -p "Press Enter when device is connected, or 'q' to quit: " retry
        
        if [ "$retry" = "q" ]; then
            return 1
        fi
        
        devices=$(adb devices 2>/dev/null | grep -E "device$" | grep -v "emulator" | wc -l)
        if [ "$devices" -eq 0 ]; then
            log_error "Still no device detected"
            return 1
        fi
    fi
    
    install_to_device "$BUILT_APK_PATH"
}

run_in_emulator() {
    echo ""
    log_info "Setting up Android emulator environment..."
    
    # Check if Java is installed
    if ! command -v java >/dev/null 2>&1; then
        log_error "Java is required for the emulator"
        echo "Install with: sudo apt install openjdk-17-jdk"
        return 1
    fi
    
    # Setup SDK if needed
    if [ ! -d "$ANDROID_HOME/emulator" ]; then
        echo ""
        log_warning "Android emulator not found. This will download ~2GB of files."
        read -p "Continue with automatic setup? [Y/n]: " confirm
        
        if [[ "$confirm" =~ ^[Nn] ]]; then
            log_info "Skipping emulator setup"
            return 0
        fi
        
        setup_android_sdk
    fi
    
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    # Create AVD if needed
    create_avd
    
    # Start emulator
    start_emulator
    
    # Install and launch app
    if [ $? -eq 0 ]; then
        sleep 2  # Give emulator a moment to stabilize
        install_to_device "$BUILT_APK_PATH"
    fi
}

show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build       Build signed release APK (default)"
    echo "  keystore    Generate/manage keystore only"
    echo "  verify      Verify APK signature"
    echo "  install     Install latest APK to connected device"
    echo "  emulator    Setup and run emulator with the app"
    echo "  clean       Clean build artifacts"
    echo "  help        Show this help"
    echo ""
    echo "Environment Variables (for non-interactive/CI mode):"
    echo "  KEYSTORE_PASSWORD   Keystore password (required for non-interactive)"
    echo "  KEY_PASSWORD        Key password (defaults to KEYSTORE_PASSWORD)"
    echo "  KEY_ALIAS           Key alias name (default: image-release)"
    echo "  KEYSTORE_VALIDITY   Validity in days (default: 10000)"
    echo "  CERT_CN             Certificate CN (default: iMAGE Developer)"
    echo "  CERT_OU             Organization unit (default: Development)"
    echo "  CERT_O              Organization (default: iMAGE)"
    echo "  CERT_L              City/Locality"
    echo "  CERT_ST             State/Province"
    echo "  CERT_C              Country code (default: US)"
    echo "  ANDROID_HOME        Android SDK location (default: ~/Android/Sdk)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Interactive build"
    echo "  KEYSTORE_PASSWORD=secret $0 build    # Non-interactive build"
    echo "  $0 install                           # Install to device"
    echo "  $0 emulator                          # Run in emulator"
}

clean_build() {
    log_info "Cleaning build artifacts..."
    
    rm -rf "$ANDROID_PROJECT/app/build" 2>/dev/null || true
    rm -rf "$ANDROID_PROJECT/build" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/dist" 2>/dev/null || true
    
    log_success "Build artifacts cleaned"
}

# ═══════════════════════════════════════════════════════════════════════════
# MAIN EXECUTION
# ═══════════════════════════════════════════════════════════════════════════

print_banner

case "${1:-build}" in
    build)
        check_prerequisites
        
        if ! load_keystore_properties; then
            generate_keystore
        fi
        
        configure_signing
        build_release
        verify_apk
        show_output
        post_build_menu
        ;;
    keystore)
        check_prerequisites
        generate_keystore
        ;;
    verify)
        verify_apk
        ;;
    install)
        # Find latest APK
        BUILT_APK_PATH=$(find "$ANDROID_PROJECT/app/build/outputs/apk" -name "*.apk" -path "*/release/*" 2>/dev/null | head -1)
        if [ -z "$BUILT_APK_PATH" ]; then
            log_error "No release APK found. Run '$0 build' first."
            exit 1
        fi
        install_to_connected_device
        ;;
    emulator)
        # Find latest APK
        BUILT_APK_PATH=$(find "$ANDROID_PROJECT/app/build/outputs/apk" -name "*.apk" -path "*/release/*" 2>/dev/null | head -1)
        if [ -z "$BUILT_APK_PATH" ]; then
            log_error "No release APK found. Run '$0 build' first."
            exit 1
        fi
        run_in_emulator
        ;;
    clean)
        clean_build
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
