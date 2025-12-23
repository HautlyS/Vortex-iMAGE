#!/bin/bash
set -e
# Configuration
ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
SYSTEM_IMAGE="system-images;android-34;google_apis;x86_64"
AVD_NAME="iMAGE_Emulator"

echo "Checking if AVD '$AVD_NAME' exists..."
if avdmanager list avd 2>/dev/null | grep -q "$AVD_NAME"; then
    echo "AVD '$AVD_NAME' already exists."
else
    echo "Creating AVD '$AVD_NAME' using image '$SYSTEM_IMAGE'..."
    # 'no' answers "Do you wish to create a custom hardware profile?"
    echo "no" | avdmanager create avd -n "$AVD_NAME" -k "$SYSTEM_IMAGE" --force
    echo "AVD created successfully."
fi
