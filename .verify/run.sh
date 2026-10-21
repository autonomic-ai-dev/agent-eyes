#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/.." && pwd)"

echo "==> [agent-eyes] Starting verification suite..."
cd "$REPO_ROOT"

echo "==> Running cargo test -p agent-eyes..."
cargo test -p agent-eyes

echo "==> Running cargo clippy -p agent-eyes (warnings are errors)..."
cargo clippy -p agent-eyes --all-targets -- -D warnings

echo "==> [agent-eyes] Verification passed cleanly (exit 0)."
