# W14 — MXC × Deno authorization-to-containment boundary

Contract: `00-BASE.md` + `12-MXC-SECURITY-01.md` + `13-DENO-PERMISSION-01.md`.
Status: **DESIGNED / NOT_EXECUTED**. No authority to modify the Gnosix Judgment Plane, desktop MCP, in.gnu6 core, Coolify, secrets or production.

## Question
Can a Deno-specified permission request be narrowed into a kernel-enforced execution boundary, with fail-closed refusal when the requested scope is unenforceable?

## Adapters and independent authorities
- Deno broker: policy decision and contextual audit; the observed broker override precedes ordinary `PermissionState` checking, so define caller, issuer and allowed override scopes explicitly.
- Judgment Plane: separately owned typed `ScopedCapability`/`ExecutionGrant`, non-forgeability and TTL. **No new grant authority is implied.**
- MXC SDK: process lifecycle and OS-sandbox enforcement. Its Bubblewrap backend must confirm exact policy support and report errors instead of silent fallback.
- Kernel: actual syscall-level evidence; neither policy logs nor source-string checks alone establish this.

## Planned falsification matrix
1. Allow in Deno + allow MXC → permitted read of disposable allowed file only.
2. Allow in Deno + deny MXC → denied host file read/write, durable process exit and audit.
3. Deny in Deno + allow MXC → denied without privileged spawn, if intended policy.
4. Unknown broker answer or response ID → fail closed, no fallback to direct permission state.
5. Unsupported host policy → reject before launching; no downgraded permissive backend.
6. Broker absent vs broken → separately defined and tested; no assumption of equivalence.
7. Host identity/TTL/resource scope mismatch → non-amplification gate RED.

Inputs must be temporary fixtures and exact source pins; tests require **actual** Deno and MXC runtime execution under isolated bounded lab conditions. No compilation, integration or product release is authorized simply by writing this contract.
