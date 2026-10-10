# CUDA-05E amendment 01 — bounded setup failure and ordering repair

**Date:** 2026-10-09. **Frozen base:** `84f82a786d80981edd824a2d9c81f4bbfd105c71`.
**Status:** prerequisite failure before any socket identity observation; no hypothesis was tested and no result was silently discarded.

## What happened

The first acquisition invocation stopped at directory setup: in a `debian:trixie-slim` one-shot setup container, `CAP_CHOWN` allowed `chown 65534:<client-group> /lab`, then `chmod 0710 /lab` failed with "Operation not permitted". After the ownership transition, the process lacked the privileges to modify the directory mode. The Python TemporaryDirectory cleanup also could not remove the temporarily re-owned directory. A separate, narrowly scoped one-shot Docker cleanup restored the original host user's ownership, and the temporary directory was deleted. **Zero peer-credential samples were obtained.**

## Amendment before a new acquisition

The host owner now applies `chmod 0710` **before** launching the setup container, and the setup container performs **only** `chown` with `CAP_CHOWN`. No extra capability was granted. This changes the setup order, not the hypotheses, identity cases, allowed/denied socket policy or decision rules. The next successful run must have a new UTC timestamp and fresh raw evidence. Any further deviation requires another explicitly recorded amendment.

**Audit status:** initial precondition failure retained here as a first-run negative operational result; the next trial is independent, not an overwrite of collected identity data.
