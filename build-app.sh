#!/bin/bash
# Builds Wingman.app (type check, engine tests, release build) and installs it in /Applications, relaunching it if
# it was running. Installing matters when the source sits on an external drive: an app run from there makes macOS
# ask for removable-volume access at every rebuild.
#   ./build-app.sh                build and install
#   ./build-app.sh --no-install   build only (the app is in src-tauri/target/release/bundle/macos/)
set -euo pipefail
cd "$(dirname "$0")"
[ -d /opt/homebrew/opt/rustup/bin ] && export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
command -v cargo >/dev/null || { echo "Rust is missing: https://rustup.rs"; exit 1; }
[ -d node_modules ] || npm ci
# Keep local paths (home folder, checkout location) out of the binary.
# (CARGO_ENCODED_RUSTFLAGS: flags separated by 0x1f, so paths with spaces work.)
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=wingman"$'\x1f'"--remap-path-prefix=$HOME=~"
SRC="src-tauri/target/release/bundle/macos/Wingman.app"
LOG="${TMPDIR:-/tmp}/wingman-build.log"
npx svelte-check --threshold error >/dev/null || { npx svelte-check --threshold error; exit 1; }
TESTS=$( (cd src-tauri && cargo test --quiet 2>&1) ) || { echo "$TESTS" | tail -30; echo "tests failed"; exit 1; }
echo "$TESTS" | grep -E "test result: ok. [1-9]" || true
rm -rf "$SRC"   # never install a bundle left over from an earlier build
code=0; npx tauri build >"$LOG" 2>&1 || code=$?
grep -E "^(error|warning)|Finished 1 bundle" "$LOG" || true
[ "$code" = 0 ] && [ -d "$SRC" ] || { echo "build failed (exit $code), full log: $LOG"; exit 1; }
[ "${1:-}" = "--no-install" ] && { echo "built $SRC"; exit 0; }
WAS_RUNNING=0; pgrep -f "Wingman.app/Contents/MacOS/wingman" >/dev/null && WAS_RUNNING=1
pkill -f "Wingman.app/Contents/MacOS/wingman" || true; sleep 1
rm -rf /Applications/Wingman.app && cp -R "$SRC" /Applications/Wingman.app
echo "installed /Applications/Wingman.app"
[ "$WAS_RUNNING" = 1 ] && open /Applications/Wingman.app || true
