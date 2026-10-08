# A04 — Interopérabilité potentielle avec gnos.ux

**Objectif:** comparer `in.gnu6` TS/scenic avec les surfaces native Rust/GPUI de Gnosix, sans postuler l'interopérabilité.

**Sources RO:** `in.gnu6/src/**`, `CONTRACT.md`; `/mnt/workbench/workspaces/gnu.in.labs/gnosix/crates/gnos-ux-*/**` en lecture seule, SHA du repo Gnosix enregistré.

**Write-set unique:** `docs/agent-results/par-01/gnosux-compatibility.md`.

**Questions:** identity, lifecycle, scene vs UI intent, geometric projection, threading, serialization/FFI, provider adapters, latency; distinguer `SAME/RELATED/DIFFERENT/UNKNOWN`.

**Gate:** source+test de chaque côté, une hypothèse falsifiable de boundary; aucun code d'interop, SDK, FFI, migration GPU ou décision de language.

**Remise:** carte d'autorités et propositions minimales de bridges réversibles, classées par incertitude et autorisation.
