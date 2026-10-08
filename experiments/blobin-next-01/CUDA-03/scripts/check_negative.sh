#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD=/mnt/workbench/build/blobin-cuda03
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
mkdir -p "$BUILD"
cd "$ROOT"
: > "$BUILD/negative.log"
fail_expected() {
  local label="$1" pattern="$2"
  shift 2
  local output exitcode
  set +e
  output="$("$@" 2>&1)"
  exitcode=$?
  set -e
  if [ "$exitcode" -eq 0 ] || ! grep -q "$pattern" <<<"$output"; then
    printf 'CUDA03_NEGATIVE_RED case=%s exit=%s output=%s\n' "$label" "$exitcode" "$output" >&2
    exit 1
  fi
  test "$exitcode" -eq 1
  printf 'CUDA03_EXPECTED_FAIL case=%s exit=%s\n' "$label" "$exitcode" | tee -a "$BUILD/negative.log"
}
fail_expected required_lib "EXPLICIT CUDA LIBRARY REQUIRED" cargo run --offline --locked --quiet -- --count 16 --dpr 1 --iterations 8
fail_expected absent_lib "LIBRARY UNAVAILABLE" cargo run --offline --locked --quiet -- --count 16 --dpr 1 --iterations 8 --cuda-lib "$BUILD/missing.so"
fail_expected bad_iterations "CUDA03 LAB BUDGET" cargo run --offline --locked --quiet -- --count 16 --dpr 1 --iterations 3 --cuda-lib "$BUILD/libblobin_cuda03.so"
fail_expected bad_count "CUDA03 LAB BUDGET" cargo run --offline --locked --quiet -- --count 257 --dpr 1 --iterations 8 --cuda-lib "$BUILD/libblobin_cuda03.so"
fail_expected missing_vulkan "CUDA03 VULKAN ADAPTER" env VK_ICD_FILENAMES="$BUILD/absent-icd.json" cargo run --offline --locked --quiet -- --count 16 --dpr 1 --iterations 8 --cuda-lib "$BUILD/libblobin_cuda03.so"
echo CUDA03_FIVE_NEGATIVES_PASS
