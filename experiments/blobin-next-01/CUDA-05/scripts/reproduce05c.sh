#!/usr/bin/env bash
set -euo pipefail
lab="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project="$(cd "$lab/.." && pwd)"
repo="$(git -C "$lab" rev-parse --show-toplevel)"
evidence="$lab/fixtures/qualification-20261009-c"
native="/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so"
test -f "$native" || { echo "CUDA-03 native library absent: $native" >&2; exit 1; }
mkdir -p "$evidence"
cargo fmt --manifest-path "$lab/Cargo.toml" -- --check
cargo clippy --offline --locked --manifest-path "$lab/Cargo.toml" --all-targets -- -D warnings
cargo build --offline --locked --release --manifest-path "$lab/Cargo.toml" --bins
: > "$evidence/cumulative-tests.log"
for phase in F1 F2 F3 F4 F4B F4C CUDA-02 CUDA-03 CUDA-04 CUDA-05; do
    echo "PHASE_BEGIN phase=$phase" >> "$evidence/cumulative-tests.log"
    cargo test --offline --locked --manifest-path "$project/$phase/Cargo.toml" > "$evidence/current-cargo-output.tmp" 2>&1 || {
        cat "$evidence/current-cargo-output.tmp" >&2
        rm -f "$evidence/current-cargo-output.tmp"
        exit 1
    }
    grep '^test result:' "$evidence/current-cargo-output.tmp" >> "$evidence/cumulative-tests.log"
    echo "PHASE_PASS phase=$phase" >> "$evidence/cumulative-tests.log"
done
rm -f "$evidence/current-cargo-output.tmp"
echo 'CUMULATIVE_ALL_PASS phases=10' >> "$evidence/cumulative-tests.log"
python3 "$lab/scripts/qualify05c.py"
{
    echo "DATE_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "PARENT=$(git -C "$repo" rev-parse HEAD)"
    echo "BRANCH=$(git -C "$repo" branch --show-current)"
    echo "GPU=$(nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader | head -1)"
    echo "RUST=$(rustc -V)"
    echo "CUDA=$(nvcc --version | tail -1)"
    echo "SERVICE_UID=$(id -u)"
    echo "WORKER_SHA256=$(sha256sum /mnt/workbench/target/release/worker05c | awk '{print $1}')"
    echo "CUDA_NATIVE_SHA256=$(sha256sum "$native" | awk '{print $1}')"
    echo 'IDENTITY_STATUS=unshare --user --map-auto unsuccessful: newuidmap Could not set caps; separate host UID not qualified'
    echo 'OPERATION=each child GPU fresh context, CUDA-02 CPU parity, bounded timeout and child reaping'
    echo 'FAULT_SCOPE=child SIGKILL and sleep(30) in worker only; no physical GPU reset'
    echo 'CLIENT_SCOPE=client and server share host UID; incorrect allowed UID test only'
    echo 'PROVIDER_SCOPE=locally built SHA256 does not prove signed origin; native loader not sandboxed'
} > "$evidence/environment.txt"
(cd "$evidence" && sha256sum results.log cumulative-tests.log environment.txt > SHA256SUMS.txt)
python3 "$lab/scripts/validate05c.py"
