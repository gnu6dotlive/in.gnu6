# B01 - blob.in product research (batch IN-GNU6-PAR-01)

Worker: B01. Base: `c9dcdab997f6`. Branch: `feat/par-01-B01`. Date: 2026-10-08.
Contract: `contracts/agent-batches/2026-10-08/07-BLOBIN-RESEARCH.md`.
Scope: qualify blob.in as a product (organic-shape engine), separate from its donor role for `in.gnu6`.
SQ-003 is not reopened. No source snapshot byte was modified.

Status vocabulary: `VERIFIED_NOW`, `HISTORICAL_RECEIPT`, `HYPOTHESIS`, `FALSIFIED`, `NOT_RUN`, `BLOCKED_AUTHORITY`, `UNKNOWN`.
(The contract's `REPRODUCED_NOW / HISTORICAL_ONLY / BLOCKED / FALSE` map to `VERIFIED_NOW / HISTORICAL_RECEIPT / BLOCKED* / FALSIFIED`.)

## 1. Sources and identity

- Snapshot (read-only): `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`
- Receipt read: `docs/provenance/donor-blobin-appstack-b3-4603aed.md`; QUESTLOG SQ-003.
- Inventory: 95 files. Manifest digest recomputed, command:
  `find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum` run in the snapshot root
  = `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`. **VERIFIED_NOW**: matches the receipt (the receipt's exact
  recipe was not stated; this recipe, with `./`-relative paths and NUL-sorted order, reproduces it).
  Re-run after all work: identical (pre/post unchanged). Other recipes (hash-of-hashes only, sorted/unsorted) do not match,
  so the digest is recipe-dependent; the recipe above should be recorded by the integrator.
- Suffix `4603aed`: the sibling build dir `/mnt/workbench/build/` holds `verify-4603aedde4d189d2e4bdd3e65872faff237aaf7d.log`
  (content: a single line, a "Good git signature" for an ED25519 key). This gives a **candidate full SHA**
  `4603aedde4d189d2e4bdd3e65872faff237aaf7d` for the suffix. `git cat-file -t` for it failed in every git repo under
  `/mnt/workbench/workspaces/gnu.in.labs/*/` (checked 25). The log does not name the object it verified and does not bind the
  snapshot bytes to that commit. Status: full SHA is a **HYPOTHESIS**; resolution as a live Git object remains **UNKNOWN / not resolved**.
- Snapshot has no `.git`. Declared repo `https://github.com/gnu-in-labs/blob.in`: not queried (no access authorization given
  for this run; public/private status not assumed). **NOT_RUN**.
- Version/license metadata (declared, not legal validation): `Cargo.toml` workspace 0.14.0, `GPL-3.0-or-later`.

## 2. Component topology (source reading + fresh commands)

Workspace members (`Cargo.toml`): `blobin-core`, `blobin-gen`, `blobin-golden`, `blobin-bench`, `blobin-cxxqt`, `blobin-gpui`; plus
non-member C++ `blobin-qt` (CMake/Corrosion), `tokens.json`, `gen/`, `tools/`.
Dependency edges: `bench/golden/cxxqt/gpui -> blobin-core`; `blobin-gen` has no engine dependency (serde only);
`blobin-gpui -> gnosis-gpui-theme (path ../../gnosis-gpui-theme) + gpui (zed git rev 00bd72e7...)`.
`blobin-core` deps: lyon 1.0, glam 0.27, cxx 1.0, optional rayon (feature `threaded-solver`, default on); crate-type staticlib+rlib.
`blobin-core/src` modules (line counts from `wc -l`): path 120, blob 389, group 106, tessellate 182, solve 179, state 208,
engine 183, bloom 417, pool 181, material 137, raster 202, diff 142, tokens 61 (generated), backend 79, ffi 387.

| Component | What it is | Fresh evidence (this run) | Historical only | Gap |
|---|---|---|---|---|
| blobin-core geometry/tessellation/solver/state/scheduler | Rust engine | VERIFIED_NOW: 47 tests (41 unit + 3 lifecycle + 3 scene) pass in default, `--no-default-features` and `--features threaded-solver` | R3 sign-off | no fresh GPU/perf measurement |
| blobin-gen (tokens.json -> Rust/C++/GPUI) | codegen | VERIFIED_NOW: 7 tests pass; regenerated `tokens.rs` byte-identical to committed (R1) | - | none found |
| blobin-golden | CPU raster oracle + diff | VERIFIED_NOW: 4 material-parity tests pass | R8 | tests CPU-vs-CPU goldens only; see GPU row |
| blobin-cxxqt | Q_PROPERTY surface | VERIFIED_NOW: 1 test (Rust side only) | - | Qt-side behavior not exercised |
| blobin-qt (C++ shim, shaders) | QQuickItem/QSG shim | VERIFIED_NOW (static): `loc-guard.sh` 282/300; `no-unsafe.sh` OK. Qt/CMake build: NOT_RUN today | receipt reports build PASS in reconstructed copy | no fresh Qt build or render in this run |
| blobin-gpui | GPUI consumer | NOT_RUN (cannot build: missing path dep + git dep needs network) | - | **BLOCKED**: `gnosis-gpui-theme` absent from snapshot |
| blobin-bench | Criterion frame budget | NOT_RUN (excluded; benchmark requires a question + reference box) | R7: ~82 us for 16 engines + bloom(12), i7-12700F | no fresh number |
| GPU shader == raster | hardware parity | NOT_RUN. No GPU test performed; no new GPU claim made | R8: llvmpipe 4/4 exact; RTX 3070 numeric pass; operator qualitative pass; `sha256sum -c checksums.sha256` on the evidence dir: all 7 files OK now (integrity only) | the 2026-06-11 run itself not repeated |

Pristine workspace self-containment: in an isolated copy (outside the snapshot), unmodified `cargo test -p blobin-core --offline`
fails at manifest load: `failed to read .../gnosis-gpui-theme/Cargo.toml: No such file or directory`.
Receipt claim "pristine exported workspace is not self-contained" is **VERIFIED_NOW**. (Note: the actual path is
`../../gnosis-gpui-theme` relative to `blobin-gpui`, i.e. `engine/gnosis-gpui-theme`; the receipt wrote `../gnosis-gpui-theme`.
Minor wording imprecision. `find` shows no `gpui-theme` directory anywhere within depth 4 of the baseline root.)
Copy with only the `"blobin-gpui"` line removed from `members`: `cargo test --workspace --exclude blobin-bench --offline` passes
(41+3+3, 1, 7, 0 + 4 parity, doc 0). **VERIFIED_NOW**. This proves the core/gen/golden/cxxqt crates build without GPUI, not that the
original workspace was complete.

## 3. Authority questions (Qt / GPUI / tokens)

- Token authority: `tokens.json` (version 0.20.0) is the only motion/colour source; `blobin-gen` emits `tokens.rs`, `gnu_theme.rs`,
  `gnu_theme.h`, `gnu_theme.gpui.rs`. Regeneration of `tokens.rs` matches committed bytes. **VERIFIED_NOW** for `tokens.rs` only
  (the other three outputs were not diffed by me).
- `TOKEN_AUTHORITY_REPORT.md` says its own history: "Unifie sur tokens.version = 0.20.0" and later "v0.20.0 ... current runtime source",
  while the Cargo workspace and readiness.md say engine 0.14.0 (see C4).
- Rendering authority: C++ shim consumes flat vertex/index/material buffers from Rust; the shim is described as having no geometry.
  Static check: `blobin-qt/src` contains no geometry computation beyond upload; I did not audit every line. **HYPOTHESIS** that the
  shim holds no semantic state (consistent with `loc-guard`, which only bounds size).
- GPUI is a second consumer adapter (canvas.rs 58, orb.rs 62 lines) over `blobin-core`; unbuildable here.

## 4. Contradictions inspected (gate: at least one required; four found)

C1. **`BLOBIN_BACKEND=cpp` is not a working fallback.** FALSIFIED as an implemented dual engine.
 - `backend.rs` doc: "blobin-qt reads it once at item construction and routes accordingly" and "`cpp` - the legacy C++ shape engine".
 - Counter-proof: `grep -rni backend blobin-qt blobin-cxxqt blobin-gpui` returns only two shader comments about the OpenGL backend.
   `ffi.rs` declares `backend_from_env()` (lines ~103, 317) but no C++ or other Rust caller exists in the snapshot
   (`grep -rn 'backend_from_env\|Backend::' --include='*.rs' --include='*.cpp' --include='*.h'` hits only `backend.rs` and the ffi decl/impl).
   README "Not yet" itself says the cpp path is "the selection + the documented fallback contract; the Rust path is what's implemented".
 - Consequence: R9 (signed 2026-06-12, "BLOBIN_BACKEND=cpp on the Qt-6 reference build") was a human sign-off of a configuration whose
   switch, in this source, is read by nothing. The sign-off is a HISTORICAL_RECEIPT; what it exercised is **UNKNOWN**. The shipped QML/
   shell paths outside this snapshot may implement the legacy path; that is not observable here.
C2. **Default backend disagrees across files.** `backend.rs` (and README) default = `rust`; `readiness.md` "default stays `cpp` in shipped
 channels"; `components.toml` `backend_default = "cpp"`, `backend_opt_in = "BLOBIN_BACKEND=rust"`. Both statements exist in the
 snapshot (VERIFIED_NOW by reading). Which one governs is an operator/product decision, not derivable from source. Given C1, the
 distinction is currently inert in this source.
C3. **README C++ budget is stale.** README: "Current: 205 LoC (item 137 + material 68)". `tools/loc-guard.sh` now counts six files and
 prints 282/300 (VERIFIED_NOW, identical to receipt). `ENGINEERING-NOTES.md` carries the newer table (110/51/45 + headers). README tree
 also omits `blobin-gpui` and `blobin-bench` from its layout diagram. README historical accuracy: partially FALSIFIED (numbers).
C4. **Version and license identity are inconsistent.** Workspace 0.14.0 vs `tokens.json` 0.20.0 (README/TOKEN report call 0.20.0 the
 unified/current); license: crates say `GPL-3.0-or-later` while `components.toml` lists `"AGPL-3.0"` for this engine and the baseline
 root `LICENSE` is AGPL-3.0 text. No `LICENSE` file exists inside `blob.in/`. Not a legal conclusion; flag for operator.

## 5. Sign-off state R7 -> R10 (from sign-off JSONs, read only)

- R1-R6 auto gates: R1, R2/R3 (47x3), R4, R5, R6 reproduced today as above; `just ci` / `tools/readiness.sh` not run as a whole.
- R7: pass, 2026-06-10, commit `d07d161`, reviewer OP. HISTORICAL_RECEIPT.
- R8: software pass (`664c277`), hardware numeric-pass (`339a0ca`, RTX 3070, idle max_delta 4, over 7.794%, ssim 0.99535), qualitative pass
  (`02835e9`). HISTORICAL_RECEIPT; evidence file integrity VERIFIED_NOW.
- R9: scope re-scoped by OP from Qt 5.15 host to cpp-backend-on-Qt6 (`02835e9`); functional and visual pass. HISTORICAL_RECEIPT; see C1.
- R10: `tools/r10-sign-off.json` has one entry, `result: "pending"`, null dates. 14-day dogfood: **not done** (VERIFIED_NOW by reading).
  Readiness.md says R10 is not an early-phase blocker but is required for a rust-default flip.
- Commit hashes cited by the sign-offs (`d07d161`, `664c277`, `339a0ca`, `02835e9`, `a53af06`) are not resolvable here (no `.git`): UNKNOWN.

## 6. Non-marketing product status

blob.in is a real, tested Rust organic-shape and spring/solver core (47 tests green today) with a thin Qt Quick shim and a GPUI adapter, a
token codegen loop, and a CPU raster oracle. It is not a self-contained deliverable as exported (missing sibling crate), has no resolvable
revision identity, has an unimplemented advertised fallback backend, an unfinished default-flip gate (R10 pending), and inconsistent
version/license/default-backend statements. Performance and GPU parity rest on 2026-06 historical receipts on one reference machine.

## 7. Recommendations

For B02 (product dossier): state the status in section 6; list C1-C4 as open contradictions rather than harmonizing them; cite the digest
recipe; mark every number sourced from R7/R8 as historical; do not describe `cpp` as a working fallback; do not state a license beyond
"declared GPL-3.0-or-later in crate metadata, AGPL-3.0 in components.toml/LICENSE at baseline root".
For the integrator: (a) record the digest recipe and the candidate full SHA hypothesis; (b) decide whether to ask the operator to
resolve default backend/license/version (product decisions); (c) do not edit SQ-003, but consider whether the C1 finding is a requalification
note for the receipt line "BLOBIN_BACKEND=cpp does not prove a C++ engine" (the receipt is already compatible; C1 adds the source evidence
that no consumer reads the switch); (d) correct the receipt's dependency path wording only if the operator wants receipt edits.

## 8. Commands actually run (all read-only on the snapshot)

- file/manifest digest (pre and post): `find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum`
- `cat`/`sed`/`grep` over Cargo.toml files, README, readiness.md, ENGINEERING-NOTES.md, TOKEN_AUTHORITY_REPORT.md, backend.rs,
  sign-off JSONs, components.toml, justfile, `tools/loc-guard.sh`
- `tools/loc-guard.sh 300`, `tools/no-unsafe.sh` (run in the snapshot; both only read)
- `sha256sum -c checksums.sha256` in `blobin-golden/r8-hardware/20260611-nvidia-rtx3070`
- `git cat-file -t 4603aedde4d1...` across neighbour repos
- Isolated copy (cp -a to a unique dir under `~/.cache`, since `/tmp` tmpfs was full; removed afterwards):
  `cargo test -p blobin-core --offline` (pristine; fails as above);
  after removing only the `blobin-gpui` member line: `cargo test --workspace --exclude blobin-bench --offline`;
  `cargo test --offline` in `blobin-core` with default, `--no-default-features`, `--features threaded-solver`;
  `cargo run -p blobin-gen -- tokens.json --rust <out>` then `diff` against `blobin-core/src/tokens.rs`.

## 9. Non-claims

No GPU run, benchmark, Qt/CMake build, or render was performed. No claim about the live Gnosis shell, QML paths outside this snapshot,
the GitHub repository, legal licence status, product-default ratification, or publication. SQ-003 not reopened or edited. Passing tests
show code executes as the tests assert, not that the product is correct or shippable.

## 10. Next decision (operator)

1. Is the live `cpp` backend path (outside this snapshot) real? If not, should R9 be requalified?
2. Which default backend, version (0.14.0 vs 0.20.0) and license (GPL vs AGPL) are authoritative for blob.in as a product?
3. Is access to the declared GitHub repo authorized for metadata-only inspection, to try to resolve the `4603aed` revision?
