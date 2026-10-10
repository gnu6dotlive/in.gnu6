# W01A — MXC Linux security boundary

Source: `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f` (immutable checkout). Extends 00-BASE.md, 01-MXC.md and TRONC_COMMUN.md.
Boundary: do not edit the MXC checkout, product repos or any existing worker worktrees. No production authority, package installation or source vendor.
Exclusive new write-set: `tools/upstream-rd-01/qualify_mxc_security.py`, `tools/upstream-rd-01/probe_loopback.py`, `evidence/upstream-rd-01/mxc-security-01/`, `docs/agent-results/upstream-rd-01/mxc-security-01.md` and this contract.
Oracles: seven causally removed ordering guards must all turn source oracle RED; containerized loopback with/without isolated network namespace must separate outcomes and network namespace IDs.
Build gate: offline Cargo dependencies only and bounded process, no implicit downloads. A missing crate means `BLOCKED_DEPENDENCY`, not MXC runtime FAIL.
Disposition: `SOURCE_GATE_PASS` + `HOST_BWRAP_GATE_PASS` does not establish MXC SDK execution; that gate remains NOT_RUN.
