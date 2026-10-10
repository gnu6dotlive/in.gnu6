# CUDA-05D — Scientific qualification report: host identity vs user namespaces

**Date of observation:** 2026-10-09 22:51 UTC.
**Status:** `OBSERVED_WITH_CONTROLS / DISTINCT_HOST_PRINCIPAL_NOT_QUALIFIED / PUBLICATION_CANDIDATE_WITH_LIMITS`
**Protocol:** CUDA-05D-P01, committed **before acquisition** at `ee0ea36bae0950c04e13100c7f53cf7e643c82c4`.
**Post-acquisition software amendment:** `05D-AMENDMENT-01.md`, provenance pin only.
**Population:** one Linux workstation, one execution of each of three trials, no machine-wide security assertions.

## Research question and decisive observation

Does running a client as UID 0 inside a rootless user namespace establish that a host-side Unix-socket supervisor observes it as a different host principal?

**No in this observed configuration.** The baseline client reported self UID 1000 and was observed by `SO_PEERCRED` as host UID 1000. The rootless namespace client reported **UID 0 internally** but was observed by the same server as **host UID 1000**. These are two distinct measurements: a self-report inside the namespace and a kernel peer-credential observation in the receiving namespace. The server's result is the relevant identity boundary for socket authorization.

The subordinate mapping trial exited with code 1 and diagnostic `newuidmap: Could not set caps`; no peer connection was established. This confirms that mapping did not complete **in this trial**. It does not determine the global availability or root cause of subordinate-UID mapping on other machines.

| Case | Child exit code | Child-reported EUID | Server-observed UID | Conclusion |
|---|---:|---:|---:|---|
| Baseline | 0 | 1000 | 1000 | Baseline oracle/control passed. |
| `--user --map-root-user` | 0 | 0 | 1000 | A namespace root is not a second host principal here. |
| `--user --map-auto` | 1 | not observed | not observed | Mapping unavailable in this run; failure diagnostic preserved. |

Host-account UID 1000 and raw process PIDs are **private experimental metadata only** and deliberately excluded from the public snapshot.

## Claims ledger (one claim ≠ one broad conclusion)

| Claim ID | Claim | Evidence class | Evidence | Verdict / boundary |
|---|---|---|---|---|
| C05D-01 | Baseline `SO_PEERCRED` UID matches host EUID | E4 internal observation | `identity-observations.json` baseline | Supported for this trial. |
| C05D-02 | Namespace-local UID 0 mapped to the baseline host principal | E4 internal observation | `identity-observations.json` namespace_root | Supported for this trial only. |
| C05D-03 | Auto subordinate mapping completed | E4 attempted observation | nonzero process exit, bounded stderr | **Not supported** in this trial. |
| C05D-04 | Supervisor service runs as genuinely different host account | E0 requirement, not E5 | no separate service UID was started | **NOT_QUALIFIED**. |
| C05D-05 | Agent socket authorization isolates agents with the same UID | E0 untested | SO_PEERCRED identifies UID, not agent identity | **NOT_QUALIFIED**. |
| C05D-06 | GPU context survives actual device loss | untested | CUDA-05C injected child faults, not GPU driver loss | **NOT_QUALIFIED**. |
| C05D-07 | Public page is independently reproduced from accessible source | E1 document draft, partial E4 evidence | de-identified snapshot; original repo not public | **Not externally verifiable** at present. |

The corpus E0–E6 classes are local methodological vocabulary, **not accreditation** and not a numeric scale.

## Method and controls

A Python Linux AF_UNIX server creates a temporary directory with permissions `0700` and socket `0600`. Each child connects and sends its namespace-local identity. The server independently reads Linux `SO_PEERCRED` and records the peer PID, UID, GID. Three commands, subprocess exit codes, bounded stderr, Python/kernel version and UTC timestamp are recorded. The baseline is an explicit positive control. The subordinate mapping is an attempted counterfactual, **not** treated as a positive even though UID-mapping configuration exists.

**Limitations:** three configurations tried once, no confidence interval, no independent replication, no second service host user, and no GPU reset. Self-reported client identities are forgeable and are not used for the host-authorization conclusion. `SO_PEERCRED` gives a credential snapshot, not cryptographic agent identity or device liveness. No load testing or response-time claims were made. Diagnostic text suggests a `newuidmap` capability blocker but a causal root-cause analysis is not yet supported.

## Evidence custody and falsifiability

- `05D-PREREG.md`: original prospective protocol; bytes at the frozen commit are checked.
- `probe_identity05d.py`: the preregistered instrument.
- `evidence/identity-observations.json`: complete internal observations with return codes and namespace/client/host identities.
- `evidence/interpretation.json`: derived verdict, references the frozen protocol commit and raw SHA-256.
- `evidence/public-results.json`: explicit de-identified projection for the documentation portal.
- `evidence/SHA256SUMS.txt`: raw, derived, public, frozen protocol and instrument checksums (six entries).
- `validate05d.py`: independent recomputation of material claims; rejects deliberately altered baseline peer UID. It also verifies that the public projection contains no raw PID/UID fields.
- `05D-AMENDMENT-01.md`: transparent repair of moving-HEAD provenance in the post-processing script. No preregistered decision rules or raw observations were modified.

### How to reproduce locally (requires access to the experiment source)

```bash
cd experiments/blobin-next-01/CUDA-05/methodology
python3 probe_identity05d.py
python3 evaluate05d.py
(cd evidence && sha256sum identity-observations.json interpretation.json public-results.json ../05D-PREREG.md ../probe_identity05d.py ../evaluate05d.py > SHA256SUMS.txt)
python3 validate05d.py
```

**Each rerun is a new observation**, not proof of a timeless invariant; keep the previous raw bundle before overwriting. There is no safe shortcut from a checksum to accessible full evidence.

## Publication gate for docs.gnu6.live

**Allowed draft claims:** “Within one rootless namespace trial, UID 0 inside remained the same host UID outside”; “automatic subordinate mapping failed in this trial”; “distinct host-service authorization remains unqualified.” Include both FR/EN, original method commit, observational date, E4 scope, downloadable sanitized JSON and explicit missing source access.

**Disallowed public claims:** agent identity isolated, Linux rootless generally insecure, production access approved, GPU-driver failure recovered, statistically robust reliability, signed native provider or independent external replication. A reviewer must assess the public prose for privacy leaks and source accessibility before publishing.

**Next experiment:** sacrificial VM or container runtime with **two real host principals**; start service under dedicated UID, test explicitly allowed client plus denied client, independently record `SO_PEERCRED` in the server, retain both positive and negative evidence, and add worker cgroups/seccomp and lifecycle tests without touching the graphical GPU.
