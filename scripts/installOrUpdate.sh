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

INSTALL_DIR="${XDG_DATA_HOME:-$HOME/.local}/bin"
SHARE_DIR="${XDG_DATA_HOME:-$HOME/.local}/share"

mkdir -p "$INSTALL_DIR" "$SHARE_DIR/icons/hicolor/256x256/apps" "$SHARE_DIR/applications"

echo "Installing tracetui to $INSTALL_DIR/..."
cp "$BINARY" "$INSTALL_DIR/tracetui"
chmod 755 "$INSTALL_DIR/tracetui"

if [ -f "$ICON" ]; then
    echo "Installing icon..."
    cp "$ICON" "$SHARE_DIR/icons/hicolor/256x256/apps/tracetui.png"
fi

if [ -f "$DESKTOP" ]; then
    echo "Installing desktop entry..."
    cp "$DESKTOP" "$SHARE_DIR/applications/tracetui.desktop"
fi

# Ensure INSTALL_DIR is in PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "${HOME}/.bashrc"
       echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "${HOME}/.profile"
       echo "Added $INSTALL_DIR to PATH in ~/.bashrc and ~/.profile"
       echo "Run 'source ~/.bashrc' or restart your terminal." ;;
esac

echo ""
echo "TraceTUI installed successfully! Run 'tracetui' to start."
echo "If 'tracetui' is not found, run: source ~/.bashrc"
echo "Auto-update: future versions will be detected automatically when you launch the app."
