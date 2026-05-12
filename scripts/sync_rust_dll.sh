#!/usr/bin/env bash
# Build the Rust audio crate on FalconX (Windows DLL) and copy the resulting
# .dll into the UE5 plugin's ThirdParty dir.
#
# Usage:
#   ./scripts/sync_rust_dll.sh
#
# Prereqs on FalconX (one-time):
#   - Rust toolchain installed (rustup, default `stable-x86_64-pc-windows-msvc`)
#   - VS 2022 Build Tools (already installed for UE5 engine build)
#
# This script:
#   1. Tars the rust/ source dir, scp's it to FalconX
#   2. Runs `cargo build --release` on FalconX
#   3. Copies eyecandy_audio.dll + .dll.lib back to ThirdParty/EyeCandyAudio/Win64/

set -euo pipefail

HOST="Alexander@192.168.0.145"
KEY="$HOME/.ssh/falconx_ed25519"
SSH_FLAGS=(-i "$KEY" -o BatchMode=yes -o StrictHostKeyChecking=no -o ConnectTimeout=15)
HERE="$(cd "$(dirname "$0")/.." && pwd)"

RUST_DIR="$HERE/Source/EyeCandyAudio/rust"
PLUGIN_THIRDPARTY_LOCAL="$HERE/Plugins/EyeCandyAudio/ThirdParty/EyeCandyAudio/Win64"
mkdir -p "$PLUGIN_THIRDPARTY_LOCAL"

REMOTE_RUST='C:\Users\Alexander\eyecandy-src\Source\EyeCandyAudio\rust'
REMOTE_OUT="$REMOTE_RUST"'\target\release'

echo "[1/4] Packing rust/ source..."
pushd "$HERE/Source/EyeCandyAudio" >/dev/null
tar --exclude='target' -czf /tmp/eca-rust.tar.gz rust/
popd >/dev/null

echo "[2/4] Uploading to FalconX..."
scp "${SSH_FLAGS[@]}" /tmp/eca-rust.tar.gz "$HOST:/Users/Alexander/Documents/eca-rust.tar.gz" >/dev/null
ssh "${SSH_FLAGS[@]}" "$HOST" 'powershell -Command "if (-not (Test-Path C:\Users\Alexander\eyecandy-src\Source\EyeCandyAudio)) { New-Item -ItemType Directory -Path C:\Users\Alexander\eyecandy-src\Source\EyeCandyAudio -Force | Out-Null }; tar -xzf C:\Users\Alexander\Documents\eca-rust.tar.gz -C C:\Users\Alexander\eyecandy-src\Source\EyeCandyAudio"'
rm -f /tmp/eca-rust.tar.gz

echo "[3/4] cargo build --release on FalconX (this takes 30-90s on first build)..."
ssh "${SSH_FLAGS[@]}" "$HOST" "cd /d $REMOTE_RUST && cargo build --release --lib" 2>&1 | tail -10

echo "[4/4] Pulling DLL back..."
scp "${SSH_FLAGS[@]}" "$HOST:/Users/Alexander/eyecandy-src/Source/EyeCandyAudio/rust/target/release/eyecandy_audio.dll" "$PLUGIN_THIRDPARTY_LOCAL/eyecandy_audio.dll"
scp "${SSH_FLAGS[@]}" "$HOST:/Users/Alexander/eyecandy-src/Source/EyeCandyAudio/rust/target/release/eyecandy_audio.dll.lib" "$PLUGIN_THIRDPARTY_LOCAL/eyecandy_audio.dll.lib"

# Header copy (keeps include path consistent)
cp "$HERE/Source/EyeCandyAudio/cpp/eyecandy_audio.h" "$PLUGIN_THIRDPARTY_LOCAL/eyecandy_audio.h"

ls -la "$PLUGIN_THIRDPARTY_LOCAL/"
echo "Done. DLL ready at $PLUGIN_THIRDPARTY_LOCAL"
