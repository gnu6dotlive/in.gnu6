# W03A — Deno Rust permission broker

Source: `denoland/deno@a18ce33715e30cd2b0d99c7e322ef11a65490e4d`. Extends 00-BASE.md, 03-DENO.md and TRONC_COMMUN.md.
Boundary: read specific pinned GitHub files. No Deno download, host runtime install, daemon mutation or change to Gnosix authorization.
Exclusive new write-set: `tools/upstream-rd-01/qualify_deno_broker.py`, `evidence/upstream-rd-01/deno-permission-01/`, `docs/agent-results/upstream-rd-01/deno-permission-01.md` and this contract.
Oracles: source tests verify broker response correlation, explicit allow/deny, invalid variant errors, broker IO failure exits and ordering of broker arbitration before ordinary permission state; mutation each invariant turns RED.
No executable Deno runtime available on host: this is a **static source contract with causal mutations**, not a live permission denial, isolation or complete policy audit. An absent runtime is `NOT_RUN`, not `PASS`.
