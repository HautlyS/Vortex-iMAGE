#!/bin/bash
set -e
cd "$(dirname "$0")/.."

# Colors
C='\033[0;36m' M='\033[0;35m' Y='\033[1;33m' G='\033[0;32m' R='\033[0;31m' W='\033[1;37m' D='\033[0;90m' NC='\033[0m'
BG_C='\033[46m' BG_M='\033[45m'

check_gum() {
    command -v gum &>/dev/null && return 0
    echo -e "${Y}Installing gum for interactive TUI...${NC}"
    if command -v brew &>/dev/null; then
        brew install gum
    elif command -v apt &>/dev/null; then
        sudo mkdir -p /etc/apt/keyrings
        curl -fsSL https://repo.charm.sh/apt/gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/charm.gpg
        echo "deb [signed-by=/etc/apt/keyrings/charm.gpg] https://repo.charm.sh/apt/ * *" | sudo tee /etc/apt/sources.list.d/charm.list
        sudo apt update && sudo apt install -y gum
    elif command -v pacman &>/dev/null; then
        sudo pacman -S gum
    elif command -v dnf &>/dev/null; then
        sudo dnf install gum
    else
        echo -e "${R}Please install gum: https://github.com/charmbracelet/gum${NC}"
        exit 1
    fi
}

show_banner() {
    clear
    gum style \
        --foreground 212 --border-foreground 99 --border double \
        --align center --width 60 --margin "1 2" --padding "1 4" \
        '╔═══════════════════════════════════════╗' \
        '║  ██╗   ██╗ ██████╗ ██████╗████████╗  ║' \
        '║  ██║   ██║██╔═══██╗██╔══██╚══██╔══╝  ║' \
        '║  ██║   ██║██║   ██║██████╔╝  ██║     ║' \
        '║  ╚██╗ ██╔╝██║   ██║██╔══██╗  ██║     ║' \
        '║   ╚████╔╝ ╚██████╔╝██║  ██║  ██║     ║' \
        '║    ╚═══╝   ╚═════╝ ╚═╝  ╚═╝  ╚═╝     ║' \
        '╠═══════════════════════════════════════╣' \
        '║     「 B U I L D   S Y S T E M 」     ║' \
        '╚═══════════════════════════════════════╝'
    
    echo ""
    gum style --foreground 141 --italic "  ⟨ $(date '+%Y.%m.%d') ⟩ System Online ⟨ $(uname -s) ⟩"
    echo ""
}

check_deps() {
    local missing=()
    command -v cargo &>/dev/null || missing+=("cargo")
    command -v node &>/dev/null || missing+=("node")
    
    if [ ${#missing[@]} -gt 0 ]; then
        gum style --foreground 196 "✗ Missing: ${missing[*]}"
        exit 1
    fi
    
    if ! cargo tauri --version &>/dev/null 2>&1; then
        gum spin --spinner dot --title "Installing tauri-cli..." -- cargo install tauri-cli
    fi
    
    if [ -f "pnpm-lock.yaml" ]; then
        PKG_MGR="pnpm"
        command -v pnpm &>/dev/null || npm i -g pnpm
    elif [ -f "yarn.lock" ]; then
        PKG_MGR="yarn"
    else
        PKG_MGR="npm"
    fi
    
    [ -d "node_modules" ] || gum spin --spinner dot --title "Installing dependencies..." -- $PKG_MGR install
}

setup_android() {
    if [ -z "$ANDROID_HOME" ]; then
        for dir in "$HOME/Android/Sdk" "$HOME/.android/sdk" "$HOME/Library/Android/sdk"; do
            [ -d "$dir" ] && { export ANDROID_HOME="$dir"; break; }
        done
    fi
    
    if [ -z "$ANDROID_HOME" ]; then
        gum style --foreground 214 "⚠ Android SDK not found"
        if gum confirm "Download Android SDK automatically?"; then
            export ANDROID_HOME="$HOME/Android/Sdk"
            mkdir -p "$ANDROID_HOME/cmdline-tools"
            
            local url="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
            [[ "$OSTYPE" == "darwin"* ]] && url="https://dl.google.com/android/repository/commandlinetools-mac-11076708_latest.zip"
            
            gum spin --spinner dot --title "Downloading SDK..." -- curl -sL "$url" -o /tmp/sdk.zip
            unzip -q /tmp/sdk.zip -d /tmp/ && mv /tmp/cmdline-tools "$ANDROID_HOME/cmdline-tools/latest"
            rm /tmp/sdk.zip
        else
            return 1
        fi
    fi
    
    local sdkm="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
    if [ -f "$sdkm" ]; then
        yes | "$sdkm" --licenses &>/dev/null || true
        gum spin --spinner dot --title "Installing NDK & tools..." -- \
            "$sdkm" "ndk;25.2.9519653" "platform-tools" "platforms;android-34" "build-tools;34.0.0"
        export NDK_HOME="$ANDROID_HOME/ndk/25.2.9519653"
    fi
    
    cargo tauri android init 2>/dev/null || true
    return 0
}

build_targets() {
    local targets=("$@")
    
    for target in "${targets[@]}"; do
        echo ""
        gum style --foreground 99 --bold "► Building: $target"
        
        case "$target" in
            "Linux » AppImage")
                gum spin --spinner dot --title "Compiling AppImage..." --show-output -- cargo tauri build --bundles appimage
                ;;
            "Linux » DEB")
                gum spin --spinner dot --title "Compiling DEB..." --show-output -- cargo tauri build --bundles deb
                ;;
            "Linux » RPM")
                gum spin --spinner dot --title "Compiling RPM..." --show-output -- cargo tauri build --bundles rpm
                ;;
            "macOS » DMG")
                gum spin --spinner dot --title "Compiling DMG..." --show-output -- cargo tauri build --bundles dmg
                ;;
            "macOS » App Bundle")
                gum spin --spinner dot --title "Compiling App..." --show-output -- cargo tauri build --bundles app
                ;;
            "Android » APK")
                setup_android && gum spin --spinner dot --title "Compiling APK..." --show-output -- cargo tauri android build --release
                ;;
            "Android » AAB")
                setup_android && gum spin --spinner dot --title "Compiling AAB..." --show-output -- cargo tauri android build --release --aab
                ;;
            "iOS » IPA")
                cargo tauri ios init 2>/dev/null || true
                gum spin --spinner dot --title "Compiling iOS..." --show-output -- cargo tauri ios build --release
                ;;
        esac
    done
    
    echo ""
    gum style --foreground 82 --bold "✓ Build complete!"
    gum style --foreground 245 "  Output: ./src-tauri/target/release/bundle/"
}

main() {
    check_gum
    show_banner
    check_deps
    
    # OS-specific targets
    local targets=()
    case "$OSTYPE" in
        linux*)
            targets=("Linux » AppImage" "Linux » DEB" "Linux » RPM" "Android » APK" "Android » AAB")
            ;;
        darwin*)
            targets=("macOS » DMG" "macOS » App Bundle" "iOS » IPA" "Android » APK" "Android » AAB")
            ;;
    esac
    
    gum style --foreground 141 "  ┌─────────────────────────────────────┐"
    gum style --foreground 141 "  │  Select build targets (space/enter) │"
    gum style --foreground 141 "  └─────────────────────────────────────┘"
    echo ""
    
    selected=$(gum choose --no-limit --cursor.foreground 212 --selected.foreground 82 \
        --cursor "▸ " --header "  ⟨ Available Targets ⟩" "${targets[@]}")
    
    if [ -z "$selected" ]; then
        gum style --foreground 214 "No targets selected. Exiting."
        exit 0
    fi
    
    echo ""
    gum style --foreground 99 "Selected targets:"
    echo "$selected" | while read -r t; do
        gum style --foreground 82 "  ✓ $t"
    done
    echo ""
    
    if gum confirm --affirmative "Build" --negative "Cancel" "Start build process?"; then
        mapfile -t sel_array <<< "$selected"
        build_targets "${sel_array[@]}"
    fi
    
    echo ""
    if gum confirm "Build more targets?"; then
        main
    fi
}

main
