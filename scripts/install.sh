#!/usr/bin/env bash
set -euo pipefail

# agent-eyes install script
# Usage: curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-eyes/master/scripts/install.sh | bash

NAME="agent-eyes"
REPO="autonomic-ai-dev/agent-eyes"
BINARY_DIR="${HOME}/.local/bin"
VERSION="${1:-latest}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_RAW="apple-darwin" ;;
  Linux)  OS_RAW="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64) ARCH_RAW="x86_64" ;;
  aarch64|arm64) ARCH_RAW="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
fi

TARGET="${ARCH_RAW}-${OS_RAW}"
ARCHIVE_URL="https://github.com/$REPO/releases/download/$VERSION/${NAME}-${TARGET}"
CHECKSUMS_URL="https://github.com/$REPO/releases/download/$VERSION/checksums.txt"

echo "Installing $NAME $VERSION ($TARGET)..."

mkdir -p "$BINARY_DIR"

# Download binary
echo "Downloading $ARCHIVE_URL ..."
curl -fsSL "$ARCHIVE_URL" -o "$BINARY_DIR/$NAME"
chmod +x "$BINARY_DIR/$NAME"

echo ""
echo "Installed to $BINARY_DIR/$NAME"
echo ""
echo "Ensure $BINARY_DIR is in your PATH:"
echo "  export PATH=\"\$PATH:$BINARY_DIR\""
echo ""
echo "Verify installation:"
echo "  $NAME status"
