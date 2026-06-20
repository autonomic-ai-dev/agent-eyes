#!/usr/bin/env bash
set -euo pipefail

# Build agent-eyes for macOS (x86_64 + arm64)

NAME="agent-eyes"
VERSION="${1:-$(git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")}"

echo "Building $NAME v$Version for macOS..."

cargo build --release -p "$NAME" 2>&1

BINARY="target/release/$NAME"
if [ -f "$BINARY" ]; then
  echo "Build complete: $BINARY"
  ls -lh "$BINARY"
else
  echo "ERROR: Build failed — $BINARY not found" >&2
  exit 1
fi
