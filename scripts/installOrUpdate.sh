#!/usr/bin/env bash
# Install or update TraceTUI on Linux.
# Run from the directory where tracetui binary is located.
# If already installed, running tracetui will check for updates automatically.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/tracetui"
ICON="$SCRIPT_DIR/tracetui.png"
DESKTOP="$SCRIPT_DIR/tracetui.desktop"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: tracetui binary not found next to this script."
    echo "Extract the entire tarball and run this script from the same directory."
    exit 1
fi

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo &>/dev/null; then
        SUDO="sudo"
    else
        echo "This script must be run as root or have sudo installed."
        exit 1
    fi
fi

echo "Installing tracetui to /usr/local/bin/..."
$SUDO cp "$BINARY" /usr/local/bin/tracetui
$SUDO chmod 755 /usr/local/bin/tracetui

if [ -f "$ICON" ]; then
    echo "Installing icon..."
    $SUDO mkdir -p /usr/local/share/icons/hicolor/256x256/apps
    $SUDO cp "$ICON" /usr/local/share/icons/hicolor/256x256/apps/tracetui.png
fi

if [ -f "$DESKTOP" ]; then
    echo "Installing desktop entry..."
    $SUDO mkdir -p /usr/local/share/applications
    $SUDO cp "$DESKTOP" /usr/local/share/applications/tracetui.desktop
fi

echo ""
echo "TraceTUI installed successfully! Run 'tracetui' to start."
echo "Auto-update: future versions will be detected automatically when you launch the app."
