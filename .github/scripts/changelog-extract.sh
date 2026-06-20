#!/usr/bin/env bash
set -euo pipefail

# Extract the latest release section from CHANGELOG.md
# Prints lines between "## [<version>]" and the next "## [" or EOF

CHANGELOG="${1:-CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
  echo "CHANGELOG.md not found" >&2
  exit 1
fi

awk '
  /^## \[/ {
    if (found) exit
    found = 1
    next
  }
  found { print }
' "$CHANGELOG"
