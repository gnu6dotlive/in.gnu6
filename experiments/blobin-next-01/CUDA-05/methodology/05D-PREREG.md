# CUDA-05D — Preregistration: host-principal isolation and public evidence gate

**Protocol ID:** CUDA-05D-P01 · **State:** prospective (frozen before run).
**Baseline:** CUDA-05C `6a37c17239ea35ecc7efb1f6aeb2c6a6a7bbf513`.
**Aim:** Establish whether a client in a rootless user namespace genuinely has a different host-level principal as observed through Linux `SO_PEERCRED`, and whether subordinate UID maps are available *without* privileged changes. No GPU reset, root escalation, account creation, or persistent service.

## Research question and hypotheses

- **Q1:** What host UID does a Linux socket server observe for an ordinary same-user client versus a client launched with `unshare --user --map-root-user`?
- **H1 (null):** namespace-root (`euid=0` inside) will still appear as the same host UID outside. A distinct *namespace* UID is not a distinct *host* principal.
- **Q2:** Can `unshare --user --map-auto` produce a subordinate-UID identity on this host using existing `/etc/subuid` and `/etc/subgid` configuration?
- **H2 (empirical):** record success or failure without assuming either. If it fails, capture the exit code and sanitized error. Never silently replace it with a same-UID test.
- **Q3:** Can a same-host-UID process impersonate another process's authority merely by choosing a namespace? **Operational falsifier:** if `SO_PEERCRED.uid` differs from the initial host UID, investigate a real identity boundary; if not, reject the claim of separate host-UID qualification.

## Test design and raw data contract

- Linux AF_UNIX stream socket bound under fresh private `0700` temporary directory. Socket file permission `0600`.
- Parent Python process accepts two *one-shot* connections: baseline client; rootless user-namespace client. For each, collect observed peer `pid/uid/gid` via `SO_PEERCRED`; separately collect child-process self-reported effective UID, and subprocess exit code. Store no user name or host name.
- Third subprocess probes `unshare --user --map-auto` without network or filesystem writes other than kernel mapping setup.
- All subprocesses have finite timeouts. No privilege escalation, Docker daemon, GPU driver mutation, or host account creation.
- Raw output: `identity-observations.json` (schema version, protocol commit, timestamp UTC, commands, return codes, bounded stderr, peer credentials, mode bits, declared environmental scope).
- **Control:** baseline `SO_PEERCRED.uid` must equal current host effective UID. **Counterfactual:** namespace-root UID inside may differ, but a host-level distinct identity requires outside peer UID to differ.
- **Negative test:** intentionally corrupt one asserted peer UID in a *copy* of the raw record; independent checker must reject tampering.

## Predetermined decision rules

1. **Host distinct-identity gate PASS** only with two truly different `SO_PEERCRED.uid` values, a positive authorized-client path, a denied unauthorized-client path, and evidence that the service identity itself is distinct. No synthetic expected-UID mismatch may satisfy this.
2. **Namespace identity observation PASS** if baseline and userns client complete, identity values are collected and checker verifies against the actual local return codes.
3. **Subordinate mapping** is **AVAILABLE** only if `map-auto` returns 0 and demonstrates a distinct outside UID in a real peer credential test. A zero exit code alone is insufficient.
4. **Publication gate:** only de-identified methodology and scope-bounded findings may enter public docs. Raw local file paths, user names, node names, process identifiers and device UUIDs must not appear on the public site. Public claims must be traceable to a reviewable evidence artifact; if source repo remains private, mark the study **internally replicated / externally not independently reproducible**.
5. **STOP:** any request for password or privileged system mutation, ambiguous UID mapping, missing baseline control, or inability to read peer credentials stops that gate. A partially qualified report is an acceptable scientific outcome.

## Threats to validity / non-goals

User-namespace mappings, DAC socket permissions, `SO_PEERCRED` and `newuidmap` are different mechanisms. Do not conflate them with secure agent isolation. Same-user clients can be indistinguishable at UID granularity. No hardware device fault, worker/cgroup confinement, kernel exploit resistance or production service-identity configuration is tested.

**Freeze convention:** commit this protocol and its instrument before running the acquisition. Later repairs must be a new commit with dated amendment and explicit changes in method.
