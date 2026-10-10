#!/usr/bin/env bash
# Opt-in BLOB-02C resident GPU preview. NEVER modifies the live systemd service.
set -euo pipefail
SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${XDG_RUNTIME_DIR:?BLOBIN_REQUIRES_USER_RUNTIME_DIR}"
TARGET_DIR="/mnt/workbench/build/blobin-resident-02-target"
SOCKET="$XDG_RUNTIME_DIR/blobin-showcase-resident-02/render.sock"
LIB="${BLOBIN_CUDA_NATIVE_LIBRARY:-/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so}"
if [[ ! -f "$LIB" ]]; then echo "CUDA_PROVIDER_NOT_FOUND: $LIB" >&2; exit 2; fi
CARGO_TARGET_DIR="$TARGET_DIR" cargo build --offline --release --manifest-path "$SELF/Cargo.toml" --bins
for n in blobin_renderd blobin_frame_worker blobin_resident_worker worker05c; do
  test -x "$TARGET_DIR/release/$n" || { echo "BINARY_NOT_EXECUTABLE: $n" >&2; exit 2; }
done
test ! -e "$SOCKET" || { echo "REFUSE_EXISTING_SOCKET: $SOCKET" >&2; exit 2; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BLOBIN_RESIDENT_PREVIEW_SOCKET=$SOCKET"
echo "LIVE_SYSTEMD_SERVICE_REMAINS_UNTOUCHED"
exec "$TARGET_DIR/release/blobin_renderd" \
 --socket "$SOCKET" \
 --frame-worker "$TARGET_DIR/release/blobin_frame_worker" \
 --frame-sha256 "$(sha "$TARGET_DIR/release/blobin_frame_worker")" \
 --gate-worker "$TARGET_DIR/release/worker05c" \
 --gate-sha256 "$(sha "$TARGET_DIR/release/worker05c")" \
 --resident-worker "$TARGET_DIR/release/blobin_resident_worker" \
 --resident-sha256 "$(sha "$TARGET_DIR/release/blobin_resident_worker")" \
 --resident-ttl-ms 30000 --resident-frame-budget 120 \
 --cuda-lib "$LIB" --cuda-sha256 "$(sha "$LIB")" \
 --timeout-ms 15000
