#!/usr/bin/env bash
set -euo pipefail

# Sign the macOS agent-eyes binary for distribution
# Requires: APPLE_IDENTITY environment variable or first argument

NAME="agent-eyes"
BINARY="target/release/$NAME"

if [ ! -f "$BINARY" ]; then
  echo "ERROR: $BINARY not found. Run build first." >&2
  exit 1
fi

IDENTITY="${1:-${APPLE_IDENTITY:-}}"

if [ -z "$IDENTITY" ]; then
  echo "Usage: $0 <apple-identity>"
  echo "Set APPLE_IDENTITY env var or pass as first argument."
  echo "Example: $0 'Developer ID Application: Your Name (TEAMID)'"
  exit 1
fi

echo "Signing $BINARY with identity: $IDENTITY"
codesign --force --options runtime --sign "$IDENTITY" "$BINARY"
echo "Verifying signature..."
codesign -dv --verbose=4 "$BINARY"
echo "Signing complete."
