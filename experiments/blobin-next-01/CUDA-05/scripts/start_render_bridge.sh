#!/usr/bin/env bash
# BLOB-SHOWCASE-01 — separate owner-owned local runtime only; does NOT launch Claude's viewer.
set -euo pipefail
SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v cargo >/dev/null || { echo 'cargo is required' >&2; exit 2; }
: "${XDG_RUNTIME_DIR:?XDG_RUNTIME_DIR is required for the private local Unix socket}"
if [[ ! -d "$XDG_RUNTIME_DIR" || ! -O "$XDG_RUNTIME_DIR" ]]; then
  echo 'XDG_RUNTIME_DIR must be owned by the active local user' >&2; exit 2
fi
SOCKET="$XDG_RUNTIME_DIR/blobin-showcase/render.sock"
TARGET_DIR="${CARGO_TARGET_DIR:-/mnt/workbench/target}"
CARGO_TARGET_DIR="$TARGET_DIR" cargo build --offline --release --manifest-path "$SELF/Cargo.toml" --bins
FRAME="$TARGET_DIR/release/blobin_frame_worker"
GATE="$TARGET_DIR/release/worker05c"
DAEMON="$TARGET_DIR/release/blobin_renderd"
for f in "$FRAME" "$GATE" "$DAEMON"; do
  [[ -f "$f" && -x "$f" ]] || { echo "Missing runtime executable: $f" >&2; exit 2; }
done
get_sha() { sha256sum -- "$1" | awk '{print $1}'; }
args=(--socket "$SOCKET" --frame-worker "$FRAME" --frame-sha256 "$(get_sha "$FRAME")" --gate-worker "$GATE" --gate-sha256 "$(get_sha "$GATE")" --timeout-ms 15000)
# CUDA-03 resident library is the actual native provider for the CUDA-04/05 runtime.
# No CPU fallback is implied if CUDA is not configured.
CUDA_LIB="${BLOBIN_CUDA_NATIVE_LIBRARY:-/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so}"
if [[ -f "$CUDA_LIB" ]]; then
  args+=(--cuda-lib "$CUDA_LIB" --cuda-sha256 "$(get_sha "$CUDA_LIB")")
else
  echo "NOTE: CUDA library not found ($CUDA_LIB). CPU/Vulkan still available; CUDA returns RENDER_ERROR." >&2
fi
if [[ -e "$SOCKET" ]]; then
  echo "Socket already exists: $SOCKET. Check whether renderd is still running; refuse to overwrite an unknown socket." >&2
  exit 2
fi
printf 'BLOB-FRAME-v1 socket=%s\n' "$SOCKET"
exec "$DAEMON" "${args[@]}"
