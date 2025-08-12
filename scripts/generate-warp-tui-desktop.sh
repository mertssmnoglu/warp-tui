#!/usr/bin/env bash
# Generates a desktop entry for warp-tui

APP_NAME="Warp TUI"
EXEC_PATH="$HOME/.cargo/bin/warp-tui"
ICON_PATH=""
DESKTOP_FILE="$HOME/.local/share/applications/warp-tui.desktop"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=$APP_NAME
Comment=Warp Terminal UI
Exec=$EXEC_PATH
Icon=${ICON_PATH}
Terminal=true
Type=Application
Categories=Utility;TerminalEmulator;
EOF

chmod +x "$DESKTOP_FILE"

if command -v update-desktop-database >/dev/null; then
    update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1
fi

echo "Desktop entry created at: $DESKTOP_FILE"
