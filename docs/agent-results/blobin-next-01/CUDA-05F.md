# CUDA-05F — Real Rust GPU supervisor under a distinct Linux host principal

**Status:** `10/10 OPERATIONAL LAB GATES PASS; RELEASE_AUTHORITY NOT_QUALIFIED; DEPLOYMENT DENIED`.
**Pre-registered protocol:** `03ba7945fe6740b2cf35dc28656ba866f66c33fb`.
**Shared docs contract:** `GPU-EVIDENCE-CONTRACT-v1.json`, SHA-256 `9af8d1f9222c4be0a3e7febbe6c4318eba069fd540ae2c0de848f8aa766a1761`.
**Device:** host RTX 3070 on an active Wayland desktop. No GPU reset, production service change or permanent accounts.

## Experimental system

A **real `supervise05f` Rust binary** (derived from the previously tested CUDA-05C supervisor) ran as host Linux UID **65534** under a disposable, networkless, read-only-rootfs Debian Docker container with no capabilities and `no-new-privileges`. A narrow temporary bind-mounted directory was owned by service UID65534 / authorized group1000, mode 0710; the server's Unix socket used 0660. The server independently recorded client identities with Linux `SO_PEERCRED`, before each accepted request.

The authorized host UID1000 (distinct from the service principal) could acquire permits and render; UID65533 with group1000 could connect but was denied by the Rust service; UID65532 / group65532 was blocked by the kernel's socket/parent DAC. Host `/proc/<Docker-inspected PID>/status` recorded all four service UID fields as 65534. Client requests cannot provide arbitrary GPU geometry or invoke injected faults.

The service launched the **existing real CUDA-05C worker process**, not a substitute interpreter fixture, with approved local worker and CUDA native-provider digests. The worker computed a genuine 16-primitive scene using CUDA and Vulkan (in separate finite runs), synchronised and compared 49,152 output samples against a CPU oracle. Fault injection is CLI-only in disposable worker children and not available through the user socket. Worker crashes and hangs revoked grants; subsequent reacquisition and GPU rendering succeeded. Corrupt staged worker bytes and CUDA provider bytes at the **same path** were rejected and recovered following restoration.

## Operational gate record

| Gate | Final lab result | Evidence source |
|---|---|---|
| host_service_uid | PASS | Docker host process UID quartet, socket/parent ownership, no userns remap |
| authorized_host_peer | PASS | Rust `CUDA05F_PEER_OBS` UID1000 accepted |
| rejected_connected_peer | PASS | Rust `CUDA05F_PEER_OBS` UID65533 `allowed=false`; user-facing denial |
| dac_rejected_peer | PASS | UID65532:65532 connection error before Rust accept |
| cuda_gpu_parity | PASS | CUDA worker fresh GPU frames, CPU numeric oracle |
| vulkan_gpu_parity | PASS | Vulkan worker fresh GPU frames, CPU numeric oracle |
| worker_crash_recovery | PASS | Injected child SIGKILL in both backends, grant revoked, next fresh worker succeeds |
| worker_timeout_recovery | PASS | Injected sleeping child killed/reaped after bounded deadline; reacquire & parity |
| worker_binary_integrity | PASS | Same-path corrupted worker rejected before launch; restoration requires new permit |
| provider_integrity | PASS (behavioral) | Native provider corrupted only at staged path; rendered worker error, revoke and recovery |
| release_authority | **NOT_QUALIFIED** | No independent human approval or signed release attestation |

**Source and evidence:** `experiments/blobin-next-01/CUDA-05/src/{socket.rs,bin/supervise05f.rs,bin/worker05c.rs,isolation.rs}`; `methodology/{05F-PREREG.md,GPU-EVIDENCE-CONTRACT-v1.json,qualify05f.py,evaluate05f.py,validate05f.py}`; raw/derived/public observation bundles under `methodology/evidence/05F/`.

## Adversarial methodology — preserved negative trials

**Four sequential acquisitions**, not four equivalent statistical repetitions:

1. **Run01 RED (Vulkan):** old Debian/Rust loader staging used ELF RUNPATH; Vulkan adapter unavailable in the integrated service. Standalone loader probes discriminated RPATH versus RUNPATH; `05F-AMENDMENT-01.md` recorded the intervention before the next trial.
2. **Run02 INCOMPLETE (identity evidence):** CUDA and Vulkan worked with transitive RPATH, but the rejected client UID was not retained in Rust server logs. The configured client UID alone could not satisfy the preregistered *server-observed* identity gate. `05F-AMENDMENT-02.md` added kernel-observed peer evidence before re-acquisition.
3. **Run03 RED (IPC lifecycle):** an injected Vulkan worker crash coincided with lost client connection, after which the Rust service terminated on `Broken pipe`. The error does **not** prove GPU driver failure. `05F-AMENDMENT-03.md` made failed socket writes revoke grants and continue, while retaining a bounded 40-second client deadline.
4. **Run04 PASS (operational gates):** eight finite, real-service scenarios under the revised, logged and committed instrument. All ten operational gates qualified; production authorization remains DENIED.

Full, **unmodified** raw observations for runs 1–3 remain separately archived. The final run is a fourth acquisition, not a reinterpretation of old data. The primary experiment uses one completed qualifying scenario of each treatment, not a randomized reliability study. The independent `validate05f.py` parses kernel peer log events, authorization decisions, generation transitions, known error codes, lifecycle exit/cleanup and all evidence hashes without importing the generator; it rejects a forged service host UID. The result is de-identified by a separate projection.

## Limits and threats to validity

- The **rootful Docker daemon** and workstation administrator outrank the service. The service and worker share UID65534; this is not a malicious-worker sandbox, seccomp/cgroup attestation or a process-secrecy guarantee.
- Host `/usr/lib` and Vulkan driver ICD are mounted read-only to compensate for host glibc 2.44 vs Debian trixie glibc 2.41. Altered ELF interpreter/RPATH and test binary digests are lab artifacts; this is not a portable distribution solution.
- Native CUDA provider/worker SHA-256 values are **equality checks against local experimental builds**, not signatures or independent release provenance. A same-UID attacker, in-place race or hostile Docker daemon remains outside this test's security perimeter.
- Provider substitution produces the generic `WORKER_CRASH_OR_ERROR` because child stderr remains suppressed. The isolated manipulation, fail-closed observation, source path and successful restoration support *behavioral refusal*, not detailed native error attribution.
- The host GPU also serves the desktop. **No real driver reset, hardware fault, long-term watchdog event or uninterruptible GPU kernel failure** was induced.
- Neither the original blob.in/Gnosix production runtime nor its permission broker, surface integration, systemd service or public API was modified. No independent external raw-data access or human security review was performed.

## Reproduction and evidence release

```bash
cd experiments/blobin-next-01/CUDA-05
cargo test --offline --locked
cargo fmt -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
cargo build --offline --locked --release --bins
python3 methodology/qualify05f.py       # NEW acquisition; preserve prior data first
python3 methodology/evaluate05f.py
# regenerate evidence/05F/SHA256SUMS.txt for the new observation
python3 methodology/validate05f.py
```

The harness mutates only temporary staged binaries in disposable containers and cleans its own containers/directory. Re-running `qualify05f.py` overwrites the **current run file**; archive it before a replication. Publication is a separate gate: docs-HUB-01G may only ingest the sanitized public projection, retain all NOT_QUALIFIED states and mark the study `draft`.
