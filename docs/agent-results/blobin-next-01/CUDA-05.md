# CUDA-05 — Trusted Runtime Ingress & Device Lifecycle (partial)

Status: `CUDA05_CONTROLLED_INGRESS_PROTOTYPE / DEVICE_PROBE_TRUST_NOT_PROVEN / PRODUCTION_NOT_PROVEN`

Parent: CUDA-04D commit `728f5cc9c0ee926ad67e6fe32a6ae2d1cd7c30ce`.
Branch: `feat/blobin-cuda-05-ingress`.

## Delivered

- Isolated Rust crate under `experiments/blobin-next-01/CUDA-05/`.
- Managed GPU ingress with explicit CUDA absolute pathname and expected SHA-256 digest.
- Process-local non-constructible permit, private fields, device and generation binding.
- Revocation increments the managed session generation; previously issued permits cannot dispatch.
- CUDA binary content is checked against the approved hash before every evaluation. CUDA-04 also repeats its provider check during dispatch.
- Backend output remains tagged with CUDA-04 health and provider provenance.

## Critical trust boundary

The public `Probe` trait is an injectable testing/supervisor boundary, NOT a verified device-health source. Arbitrary callers can implement it and assert Ready. An operator-controlled Rust object can call `acquire` and revoke permits. No OS principal or trusted runtime service authorizes calls. Therefore the term *trusted* in the phase name denotes the intended future ingress, not a security claim for this prototype.

Actual hardware device loss, safe device reset, watchdog timeouts, permission brokerage, native-code sandboxing, filesystem trust/TOCTOU closure and production integration are unqualified. No destructive GPU reset is performed on a Wayland display machine. The earlier CUDA-04D native loader limits still apply.

## Reproduction

```sh
cd experiments/blobin-next-01/CUDA-05
cargo test --offline --locked
cargo fmt -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
```

No merge, push or deployment. Keep CUDA-05 open pending trusted supervisor integration and genuine device-health acquisition.
