# UPSTREAM-R&D-01 — License source review

Status: **VERIFIED_SOURCE**, not legal approval for redistribution. Read at the pinned commits on 2026-10-10.

- GPUI Kit `ed27c6293b585acc40c10be7138534243bee5841`: source code [LICENSE-APACHE](https://github.com/longbridge/gpui-kit/blob/ed27c6293b585acc40c10be7138534243bee5841/LICENSE-APACHE) states Apache-2.0. [LICENSE-DOCS.md](https://github.com/longbridge/gpui-kit/blob/ed27c6293b585acc40c10be7138534243bee5841/LICENSE-DOCS.md) additionally offers CC BY 4.0 for specified website documentation prose and illustrations, with exceptions for source, snippets, third-party works. GitHub auto-classifier `NOASSERTION` is not evidence of missing licensing.
- Hyperfine `b33755fe1fca8bcc1f182c773ca3ab040bfe517d`: [LICENSE-APACHE](https://github.com/sharkdp/hyperfine/blob/b33755fe1fca8bcc1f182c773ca3ab040bfe517d/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/sharkdp/hyperfine/blob/b33755fe1fca8bcc1f182c773ca3ab040bfe517d/LICENSE-MIT) both exist at pinned source. GitHub's single SPDX value Apache-2.0 is incomplete as a description of available grant files.

Other four repos: registry-level SPDX metadata captured, transitive dependencies and file-level licenses not independently reviewed. PlxNative remains a GPL compliance risk for code vendoring. No third-party files copied into GNU6.
