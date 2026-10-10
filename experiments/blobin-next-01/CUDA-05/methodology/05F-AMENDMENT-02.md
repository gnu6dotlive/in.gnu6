# CUDA-05F — Amendment 02 (peer-identity observability)

**Protocol:** CUDA-05F-P01 at `03ba7945fe6740b2cf35dc28656ba866f66c33fb`.
**Status:** post-acquisition instrumentation amendment, prior to third run.

## Discrepancy discovered by reviewing run two

The second complete trial demonstrated real CUDA + Vulkan parity and fault recovery after the ELF RPATH repair. However, the Rust service's unauthorized-UID branch used `socket::authorize` and emitted only `CUDA05C_PEER_UNAUTHORIZED`. The separate rejected client fixture *configured* UID 65533, but the supervisor's log did **not** preserve the kernel-observed UID of that rejected peer. Therefore this run could not independently establish the `rejected_connected_peer` observational predicate as originally specified, even though the implementation exercised SO_PEERCRED.

The second trial is preserved unchanged as `evidence/05F/05F-run02-rpath.json`, not silently counted as a full integrated identity gate.

## Narrow repair

Before each request, `supervise05f` now calls `socket::credentials` once, emits an internal-only `CUDA05F_PEER_OBS index=... peer_uid=... allowed=...` line derived from `SO_PEERCRED`, and then compares the observed UID with the configured allowed UID. The command set, privileges, GPU workload, authorization predicate, fault injections, resource bounds and preregistered gates are unchanged. Private raw logs retain numeric UIDs; public projection must remove them.

**Decision:** perform a **new third acquisition** and independently check observed rejected peer UID. Keep all earlier data and errors. A code improvement made after observation does not retroactively prove the old measurement.
