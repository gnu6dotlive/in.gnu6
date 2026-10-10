# CUDA-05F — Amendment 01 (after first full acquisition)

**Date:** 2026-10-09 local / 2026-10-10 UTC.
**Preregistered protocol commit:** `03ba7945fe6740b2cf35dc28656ba866f66c33fb`.
**Disposition:** **First complete run remains recorded as negative for Vulkan.** No original hypotheses or gates changed.

## Unexpected observation

In eight real Rust-service scenarios, CUDA normal/fault/tamper cases succeeded. Vulkan normal, crash and hang services started under distinct host UID and exercised Unix IPC, but *real GPU acquisition* failed with `CUDA05C_WORKER_CRASH_OR_ERROR`. The original Rust parent suppressed the worker's stderr by design; source-level reason was **not** concluded from that summary.

## Discriminating follow-up

A direct Docker child under UID65534 reproduced the failure with the *staged patchelf-modified binary* and emitted `CUDA03 VULKAN ADAPTER No suitable graphics adapter found`. The original preflight's explicit `ld-linux --library-path /hostlib:/usr/lib/x86_64-linux-gnu` had worked. A controlled ABI-loader probe using `patchelf --force-rpath --set-rpath /hostlib:/usr/lib/x86_64-linux-gnu` succeeded on exactly the same binary/backend. This identifies ELF loader search semantics (RUNPATH vs transitive RPATH) as the discriminating factor **in this environment**, not a generic Vulkan defect.

## Changed instrument, unchanged test

Only staging of the disposable worker and service ELF binaries changes: force transitive RPATH to `/hostlib:/usr/lib/x86_64-linux-gnu`. Hash approvals are calculated **after** the patch as before. Container privileges, GPU backend, permit logic, false-UID clients, crash and hang injections, source/oracle, and 11 required gates are unchanged. Rerun under new UTC timestamp as a **second** experiment; never reinterpret the first Vulkan failures as PASS.

**Original evidence custody:** `evidence/05F/05F-run01-negative.json` plus SHA-256 in the manifest. New `05F-raw.json` is the subsequent run. Clean host and no production claims.
