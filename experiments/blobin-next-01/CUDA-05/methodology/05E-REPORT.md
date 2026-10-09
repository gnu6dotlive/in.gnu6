# CUDA-05E — Distinct host principals in a disposable Linux socket laboratory

**Research protocol:** CUDA-05E-P01, frozen as commit `84f82a786d80981edd824a2d9c81f4bbfd105c71` before the experiment.
**Amendment:** `fbd752d322a148f6d83279bef5cf04b4f9436374`, describing the failed first setup and a narrower setup-order repair before re-acquisition.
**Population:** one successful experiment with one ephemeral server and four client attempts. No statistical estimates or portability claim.

## Question, method and observations

A server ran inside a rootful-Docker-managed, networkless Debian container as UID **65534** / GID **1000**, with all Linux capabilities dropped, `no-new-privileges`, read-only root filesystem, bounded process/memory limits, and a *single narrow writable bind-mounted temporary directory*. A separate one-shot setup container with only `CAP_CHOWN` assigned the lab directory to service UID 65534. Its mode was 0710; the server-owned socket mode was 0660.

The observer confirmed the server's **host process UID** directly through `/proc/<docker-inspected-pid>/status` (all four UID columns 65534). Docker inspection showed container `User=65534:1000`, no configured user namespace remapping, and network mode `none`. The server read client credentials using Linux `SO_PEERCRED`. Client UID1000 (different host principal) was allowed. A different container client UID65533:GID1000 **connected**, but was denied based on its **actual peer UID**. UID65532:GID65532 could not connect because of directory/socket DAC; the server saw no event. The authorized UID1000 succeeded again after the denial.

| Claim | Evidence | Verdict |
|---|---|---|
| Host UID of server differs from permitted host UID of client | `/proc/<PID>/status`, `docker inspect`, socket peer-credentials | **Observed** in this trial |
| A client with same socket-access group but unapproved UID is denied by application | second accepted `SERVICE_OBS` event shows peer UID65533 and DENIED | **Observed** |
| A client without permission is blocked before the application | client returned exit code 13 / Permission denied, no fourth socket accept event | **Observed** |
| The server continues accepting a permitted client after denial | third accepted connection, ALLOWED; clean service exit | **Observed** |
| Docker daemon/root compromise is prevented | not tested; daemon remains privileged | **NOT_QUALIFIED** |
| Production GPU supervisor has distinct principals and hardened sandbox | stand-alone scripted Unix socket demonstration only | **NOT_QUALIFIED** |
| Rootless subordinate UID setup works | CUDA-05D separately failed; not repaired by this experiment | **NOT_QUALIFIED** |

All operational stages succeeded in the completed trial. Exactly **one** completed experimental run was performed; earlier acquisition attempt failed **before any peer sample** because `CAP_CHOWN` alone could not also `chmod` the newly re-owned directory. That failure is preserved in `05E-AMENDMENT-01.md`, with the directory subsequently restored/cleaned. The repair moved `chmod` to the original host owner **before** `CAP_CHOWN` rather than granting more caps.

## Causal separation and threats to validity

The experiment isolates **three** checks: different actual host account IDs (host `/proc` and container user configuration), socket DAC (directory 0710 and socket 0660), and application UID allowlist (same-group rejected client). They must **not** be conflated. The recorded server UID is **65534**, with client 1000 and rejected client 65533. A peer PID of zero in the container service logs is attributable to its separate PID namespace and is **not** an identity control; the UID field was the decision input.

A rootful Docker daemon has greater authority than any tested client. The test service is a short-lived Perl fixture, not the Rust Gnosix/blob.in production supervisor; no CUDA native provider is loaded here and no GPU failure is induced. The absence of network traffic is enforced through Docker `--network none`, but supply-chain guarantees for the local image and kernel hardening were not audited. The tests establish bounded behavior, not a secure system against a malicious host administrator.

## Evidence custody and method

The preregistration is an immutable Git object. Complete local raw data: `evidence/05E/05E-raw.json`; derived decisions: `05E-derived.json`; sanitized public projection: `05E-public.json`; checksums: `SHA256SUMS.txt`. `evaluate05e.py` produces the report data and `validate05e.py` independently recomputes the principal/allow/deny/DAC/cleanup gates, checks source hashes and protocol provenance, and rejects a deliberately falsified service UID.

**Public release boundary:** share only sanitized results, method, caveats, protocol/amendment commit IDs, and the fact that private raw data cannot be independently inspected externally. Do not publish UIDs, PIDs, user names or paths in the site's sanitized JSON. The descriptive report remains internally available.

## Next experiment and integration gate

Integrate the distinct-UID socket pattern with the **real Rust CUDA-05C supervisor** in a disposable runtime, then test policy confinement and a genuine UID-denied connection end-to-end. Only then consider supervised GPU worker confinement in a sacrificial machine; this test alone does not authorize production deployment, merge or a public security claim.
