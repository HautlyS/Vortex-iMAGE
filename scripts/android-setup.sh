#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# iMAGE - Android Development Environment Setup Script
# Automatically downloads and configures Android SDK, NDK, and builds the app
# ═══════════════════════════════════════════════════════════════════════════

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ANDROID_SDK_VERSION="11076708"  # Command line tools version
ANDROID_BUILD_TOOLS="34.0.0"
ANDROID_PLATFORM="34"
NDK_VERSION="26.1.10909125"
JAVA_VERSION="17"

# Paths
ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_VERSION}_latest.zip"

print_banner() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║           iMAGE - Android Setup & Build Script                ║"
    echo "║                   Tauri 2.x Mobile                            ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking system dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    command -v curl >/dev/null 2>&1 || missing_deps+=("curl")
    command -v unzip >/dev/null 2>&1 || missing_deps+=("unzip")
    command -v rustc >/dev/null 2>&1 || missing_deps+=("rust")
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    command -v node >/dev/null 2>&1 || missing_deps+=("nodejs")
    command -v pnpm >/dev/null 2>&1 || missing_deps+=("pnpm")
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        echo ""
        echo "Please install the missing dependencies:"
        echo "  - curl, unzip: sudo apt install curl unzip"
        echo "  - rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "  - nodejs: https://nodejs.org/"
        echo "  - pnpm: npm install -g pnpm"
        exit 1
    fi
    
    log_success "All system dependencies found"
}

check_java() {
    log_info "Checking Java installation..."
    
    if command -v java >/dev/null 2>&1; then
        JAVA_VER=$(java -version 2>&1 | head -n 1 | cut -d'"' -f2 | cut -d'.' -f1)
        if [ "$JAVA_VER" -ge "$JAVA_VERSION" ]; then
            log_success "Java $JAVA_VER found"
            return 0
        fi
    fi
    
    log_warning "Java $JAVA_VERSION+ not found. Installing OpenJDK..."
    
    if command -v apt >/dev/null 2>&1; then
        sudo apt update && sudo apt install -y openjdk-${JAVA_VERSION}-jdk
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y java-${JAVA_VERSION}-openjdk-devel
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --noconfirm jdk${JAVA_VERSION}-openjdk
    else
        log_error "Could not install Java. Please install OpenJDK $JAVA_VERSION manually."
        exit 1
    fi
    
    log_success "Java installed successfully"
}

setup_android_sdk() {
    log_info "Setting up Android SDK..."
    
    # Create SDK directory
    mkdir -p "$ANDROID_HOME/cmdline-tools"
    
    # Download command line tools if not present
    if [ ! -d "$ANDROID_HOME/cmdline-tools/latest" ]; then
        log_info "Downloading Android command line tools..."
        
        TEMP_ZIP="/tmp/cmdline-tools.zip"
        curl -L "$CMDLINE_TOOLS_URL" -o "$TEMP_ZIP"
        
        unzip -q "$TEMP_ZIP" -d "$ANDROID_HOME/cmdline-tools"
        mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        rm "$TEMP_ZIP"
        
        log_success "Command line tools downloaded"
    else
        log_info "Command line tools already present"
    fi
    
    # Set up PATH
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
    
    # Accept licenses
    log_info "Accepting Android SDK licenses..."
    yes | sdkmanager --licenses >/dev/null 2>&1 || true
    
    # Install required SDK components
    log_info "Installing Android SDK components..."
    sdkmanager --install \
        "platform-tools" \
        "platforms;android-${ANDROID_PLATFORM}" \
        "build-tools;${ANDROID_BUILD_TOOLS}" \
        "ndk;${NDK_VERSION}" \
        "emulator" \
        "system-images;android-${ANDROID_PLATFORM};google_apis;x86_64"
    
    log_success "Android SDK setup complete"
}

setup_rust_targets() {
    log_info "Setting up Rust Android targets..."
    
    rustup target add aarch64-linux-android
    rustup target add armv7-linux-androideabi
    rustup target add i686-linux-android
    rustup target add x86_64-linux-android
    
    log_success "Rust Android targets installed"
}

setup_environment() {
    log_info "Setting up environment variables..."
    
    # Create/update shell profile
    PROFILE_FILE="$HOME/.bashrc"
    [ -f "$HOME/.zshrc" ] && PROFILE_FILE="$HOME/.zshrc"
    
    # Check if already configured
    if ! grep -q "ANDROID_HOME" "$PROFILE_FILE" 2>/dev/null; then
        cat >> "$PROFILE_FILE" << EOF

# Android SDK (added by iMAGE setup script)
export ANDROID_HOME="$ANDROID_HOME"
export NDK_HOME="\$ANDROID_HOME/ndk/${NDK_VERSION}"
export PATH="\$ANDROID_HOME/cmdline-tools/latest/bin:\$ANDROID_HOME/platform-tools:\$PATH"
EOF
        log_success "Environment variables added to $PROFILE_FILE"
    else
        log_info "Environment variables already configured"
    fi
    
    # Export for current session
    export ANDROID_HOME="$ANDROID_HOME"
    export NDK_HOME="$ANDROID_HOME/ndk/${NDK_VERSION}"
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
}

init_tauri_android() {
    log_info "Initializing Tauri Android project..."
    
    # Navigate to project root
    cd "$(dirname "$0")/.."
    
    # Install npm dependencies
    log_info "Installing npm dependencies..."
    pnpm install
    
    # Initialize Android
    log_info "Running tauri android init..."
    pnpm tauri android init
    
    log_success "Tauri Android project initialized"
}

build_android() {
    local BUILD_TYPE="${1:-debug}"
    
    log_info "Building Android app ($BUILD_TYPE)..."
    
    cd "$(dirname "$0")/.."
    
    if [ "$BUILD_TYPE" = "release" ]; then
        pnpm tauri android build --release
    else
        pnpm tauri android build
    fi
    
    log_success "Android build complete!"
    
    # Show output location
    echo ""
    log_info "APK location:"
    if [ "$BUILD_TYPE" = "release" ]; then
        echo "  src-tauri/gen/android/app/build/outputs/apk/release/"
    else
        echo "  src-tauri/gen/android/app/build/outputs/apk/debug/"
    fi
}

run_android() {
    log_info "Running on Android device/emulator..."
    
    # Ensure tools are in PATH
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
    
    cd "$(dirname "$0")/.."
    
    # Check for connected devices
    if ! adb devices | grep -q "device$"; then
        log_warning "No Android device connected. Starting emulator..."
        
        # List available AVDs
        AVD_NAME=$(emulator -list-avds | head -n 1)
        
        if [ -z "$AVD_NAME" ]; then
            log_error "No Android Virtual Device found. Please create one using Android Studio or:"
            echo "  avdmanager create avd -n test_device -k 'system-images;android-${ANDROID_PLATFORM};google_apis;x86_64'"
            exit 1
        fi
        
        # Start emulator in background
        emulator -avd "$AVD_NAME" &
        
        log_info "Waiting for emulator to boot..."
        adb wait-for-device
        sleep 10
    fi
    
    pnpm tauri android dev
}

show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  setup       Full setup (SDK, NDK, Rust targets, Tauri init)"
    echo "  build       Build debug APK"
    echo "  release     Build release APK"
    echo "  run         Run on connected device/emulator"
    echo "  dev         Start development mode"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup    # First time setup"
    echo "  $0 build    # Build debug APK"
    echo "  $0 release  # Build release APK"
    echo "  $0 run      # Run on device"
}

# Main execution
print_banner

case "${1:-setup}" in
    setup)
        check_dependencies
        check_java
        setup_android_sdk
        setup_rust_targets
        setup_environment
        init_tauri_android
        
        echo ""
        log_success "Setup complete! You can now build with:"
        echo "  ./scripts/android-setup.sh build"
        echo ""
        log_warning "Please restart your terminal or run:"
        echo "  source ~/.bashrc  # or ~/.zshrc"
        ;;
    build)
        build_android "debug"
        ;;
    release)
        build_android "release"
        ;;
    run|dev)
        run_android
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
