#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORE_DIR="$SCRIPT_DIR/genesis-lab-core"
WEB_DIR="$SCRIPT_DIR/web"

echo "Building genesis-lab-core with wasm-pack..."
wasm-pack build "$CORE_DIR" --target web --out-dir "$WEB_DIR/pkg" --release

echo ""
echo "Build complete. Serve the web/ directory, e.g.:"
echo "  python3 -m http.server -d $WEB_DIR 8080"
