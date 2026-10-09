#!/usr/bin/env bash
set -euo pipefail
lab="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project="$(cd "$lab/.." && pwd)"
repo="$(git -C "$project" rev-parse --show-toplevel)"
evidence="$lab/fixtures/qualification-20261009-b"
native="/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so"
test -f "$native" || { echo "Missing built CUDA03 provider: $native" >&2; exit 1; }
mkdir -p "$evidence"
cargo fmt --manifest-path "$lab/Cargo.toml" -- --check
cargo clippy --offline --locked --manifest-path "$lab/Cargo.toml" --all-targets -- -D warnings
cargo build --offline --locked --manifest-path "$lab/Cargo.toml" --bin supervise05b
: > "$evidence/cumulative-tests.log"
for phase in F1 F2 F3 F4 F4B F4C CUDA-02 CUDA-03 CUDA-04 CUDA-05; do
  echo "PHASE_BEGIN phase=$phase" >> "$evidence/cumulative-tests.log"
  cargo test --offline --locked --manifest-path "$project/$phase/Cargo.toml" \
    > "$evidence/current-phase-output.tmp" 2>&1 || {
    cat "$evidence/current-phase-output.tmp" >&2
    rm -f "$evidence/current-phase-output.tmp"
    exit 1
  }
  grep '^test result:' "$evidence/current-phase-output.tmp" >> "$evidence/cumulative-tests.log"
  echo "PHASE_PASS phase=$phase" >> "$evidence/cumulative-tests.log"
done
rm -f "$evidence/current-phase-output.tmp"
echo 'CUMULATIVE_ALL_PASS phases=10' >> "$evidence/cumulative-tests.log"
python3 "$lab/scripts/qualify05b.py" > "$evidence/current-qualification-output.tmp"
cat "$evidence/current-qualification-output.tmp"
rm -f "$evidence/current-qualification-output.tmp"
{
  echo "UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "HEAD=$(git -C "$repo" rev-parse HEAD)"
  echo "BRANCH=$(git -C "$repo" branch --show-current)"
  echo "GPU=$(nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader | head -1)"
  echo "RUSTC=$(rustc -V)"
  echo "NVCC=$(nvcc --version | tail -1)"
  echo "SERVICE_UID=$(id -u)"
  echo "NATIVE_SHA256=$(sha256sum "$native" | awk '{print $1}')"
  echo "SAME_UID_NOTE=Service and authorized client ran with the same Linux UID; distinct-principal isolation not demonstrated."
  echo "HEALTH_NOTE=Live GPU dispatch checks are point-in-time; no physical device reset."
  echo "PROVIDER_NOTE=Local SHA256 from local experimental build is not a signature or trustworthy origin."
} > "$evidence/environment.txt"
(cd "$evidence" && sha256sum results.log cumulative-tests.log environment.txt > SHA256SUMS.txt)
python3 "$lab/scripts/validate05b.py"
