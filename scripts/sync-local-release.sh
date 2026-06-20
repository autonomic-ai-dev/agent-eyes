#!/usr/bin/env bash
set -euo pipefail

# Sync the locally built agent-eyes binary to ~/.local/bin

NAME="agent-eyes"
BINARY="target/release/$NAME"
INSTALL_DIR="${HOME}/.local/bin"

if [ ! -f "$BINARY" ]; then
  echo "ERROR: $BINARY not found. Run build first." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/$NAME"
echo "Installed $BINARY -> $INSTALL_DIR/$NAME"
ls -lh "$INSTALL_DIR/$NAME"
