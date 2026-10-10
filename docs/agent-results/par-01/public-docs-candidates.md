# Public documentation candidates register (A06, batch IN-GNU6-PAR-01)

Status: **PROPOSED_FOR_REVIEW — nothing here is published or authorized for publication.**
Worker: A06. Base: `c9dcdab997f63f67caaa182cb265d6cb9b970863`. Branch `feat/par-01-A06`.
Reviewer: human operator (publication authority is not held by any agent).

## 0. Scope and method

Sources read (read-only, in this worktree at the base SHA above):
`README.md`, `CONTRACT.md`, `QUESTLOG.md`, `docs/provenance/seed-b07be20.md`, directory listing of `docs/evidence/`.
Not read: the public `engineering-corpus` / `docs.gnu6.live` portal (network not used; status `NOT_RUN`), other repositories, other worktrees.

Rule applied: a module or document name found in a private repository gives no right to expose it. Every candidate below has `public_authorized? = NO`, because no source in scope contains an operator decision authorizing public disclosure. `README.md`/`CONTRACT.md` state the opposite: public API UNDECIDED, publication NOT AUTHORIZED, package `private: true`. Conceptual cards therefore stay `PROPOSED_FOR_REVIEW`.

Gate columns: `source+revision | acceptable-public-summary | owner | public_authorized? | evidence | confidence | blocked_reason`.
Revision for all repo files: `c9dcdab` (HEAD of base; last README/CONTRACT/QUESTLOG touch observed via `git log -1`: `d160639`).
Owner is "operator (to assign)" everywhere: no source names a documentation owner (`UNKNOWN`).

## 1. Candidate sentences (gated)

| ID | source+revision | acceptable-public-summary (EN / FR) | owner | public_authorized? | evidence | confidence | blocked_reason |
|---|---|---|---|---|---|---|---|
| C01 | `README.md`@c9dcdab, "in.gnu6" intro | EN: "A headless scene, spatial and topology runtime: it computes the world; renderers project it." / FR: "Un runtime headless de scène, d'espace et de topologie : il calcule le monde, les moteurs de rendu le projettent." | operator (to assign) | NO | README text only (source read, HISTORICAL_RECEIPT of authored intent; no live behavior observed) | high that it matches the source wording; n/a for publication | No public-disclosure decision; repo is private. |
| C02 | `CONTRACT.md` §1, §3, §5 @c9dcdab | EN: "One semantic engine owns scene truth. Renderers are replaceable providers and cannot redefine identity or topology." / FR: "Un seul moteur sémantique détient la vérité de scène. Les moteurs de rendu sont des fournisseurs remplaçables et ne redéfinissent ni l'identité ni la topologie." | operator (to assign) | NO | Contract text (a declared rule, not an observed property of any running system) | high (wording); medium (as an accurate description of code, UNVERIFIED here) | No disclosure decision; "declared" must not be read as "implemented". |
| C03 | `CONTRACT.md` §8 @c9dcdab | EN: "No public API, package publication or compatibility promise exists today." / FR: "Aucune API publique, publication de paquet ni promesse de compatibilité n'existe à ce jour." | operator (to assign) | NO (but this is a non-claim; lowest disclosure risk, suggested first for review) | Contract §8; `package.json` observed `private: true`, version `0.0.0-private` | high | Still needs operator confirmation that stating it publicly is wanted. |
| C04 | `CONTRACT.md` §7 + `QUESTLOG.md` state model @c9dcdab | EN: "Claims carry an explicit truth class (hypothesis, designed, implemented, observed, falsified, ratified); no class implies the next." / FR: "Les affirmations portent une classe de vérité explicite (hypothèse, conçu, implémenté, observé, falsifié, ratifié) ; aucune classe n'implique la suivante." | operator (to assign) | NO | Contract and questlog text | high (wording) | Process description of a private program; disclosure undecided. |
| C05 | `QUESTLOG.md` MQ-00..MQ-15 states @c9dcdab | EN: "The current internal mandate is recorded as complete by the project's own questlog." / FR: "Le mandat interne actuel est consigné comme complet par le questlog du projet." | operator (to assign) | NO | Questlog states `RATIFIED` for MQ-00..MQ-15; receipts exist under `docs/evidence/` (names only listed, contents NOT re-run or re-read by A06) | medium: self-attested status, no fresh measurement | `RATIFIED` is internal; publicly it could read as a product-readiness or support claim. Recommend NOT publishing as a standalone sentence. |
| C06 | `QUESTLOG.md` MQ-12 / `README.md` @c9dcdab | EN: "A browser reference renderer exists on a third-party WebGL library; it is one replaceable provider." / FR: "Un moteur de rendu navigateur de référence existe sur une bibliothèque WebGL tierce ; c'est un fournisseur remplaçable." | operator (to assign) | NO | README/Questlog text; no render re-run by A06 | medium | Naming the vendor and version is a disclosure choice for the operator; summary deliberately omits both. |
| C07 | `docs/provenance/seed-b07be20.md` @c9dcdab | EN: "The engine's first semantic kernel was extracted from an earlier product repository and verified byte-equivalent before any new work." / FR: "Le premier noyau sémantique du moteur a été extrait d'un dépôt produit antérieur et vérifié équivalent octet par octet avant tout nouveau travail." | operator (to assign) | NO | Provenance note states 12/12 byte-equivalence PASS and 18/18 laws PASS (HISTORICAL_RECEIPT; not re-run) | medium | Provenance exposes private repository lineage; also see M-03 in the manifest. |
| C08 | `README.md` "Governing boundary" @c9dcdab | EN: "The engine does not own product state, business logic or window/application semantics; consumers do." / FR: "Le moteur ne possède ni l'état produit, ni la logique métier, ni la sémantique de fenêtres/applications ; les consommateurs les possèdent." | operator (to assign) | NO | README text | high (wording) | References private consumers/products implicitly; generic wording used on purpose. |

All eight: claim status `HYPOTHESIS` as to publication fitness; wording faithfulness `VERIFIED_NOW` against the source text only.

## 2. Redaction manifest

Terms and classes found in sources that are NOT carried into any candidate above, with the handling proposed.

| ID | Item (class) | Seen in | Handling | Reason |
|---|---|---|---|---|
| M-01 | Absolute local filesystem paths (workstation/worktree layout) | `docs/provenance/seed-b07be20.md` | REDACT | Internal layout; no value to readers. |
| M-02 | Commit SHAs of private repos (`b07be20...`, bootstrap commits) | provenance note, questlog | OPERATOR DECISION | SHAs of a private repo are inert but reveal lineage; withhold until decided. |
| M-03 | Names of private consumer products / donor projects (Docklab, jewelry product line, blob.in, gnu6.live app, Gnosix) | README, CONTRACT, QUESTLOG | WITHHOLD; use "consumer" / "earlier product repository" | Product naming and donor attribution are authority-bearing and undecided. Also: Gnosix identifiers must not carry donor names (project memory rule). |
| M-04 | Third-party renderer name and version (PixiJS 8.22.0) | README, QUESTLOG | OPERATOR DECISION | Vendor/version disclosure is a choice; summary C06 omits it. |
| M-05 | Internal quest identifiers (MQ-xx, SQ-xx, EXPERIENCE-IN-xx) and receipt filenames | QUESTLOG, `docs/evidence/` | WITHHOLD | Private program structure; names give no right to expose. |
| M-06 | Internal endpoints, hosts, deploy/Coolify details | none found in files read; not searched beyond the listed sources | NOT_OBSERVED, not "absent" | A06 did not scan the whole repo; a secret/endpoint scan is `NOT_RUN`. |
| M-07 | Secrets, tokens, authority material | none observed in files read | NOT_OBSERVED | Same limit as M-06. |
| M-08 | Operator identity / emails | none carried | n/a | |
| M-09 | Benchmark numbers and test counts (e.g. "29 files / 119 tests") | QUESTLOG | WITHHOLD | Point-in-time internal receipts; stale risk, no fresh measurement. |
| M-10 | "RATIFIED COMPLETE" wording | README, QUESTLOG | WITHHOLD (see C05) | Could be read as a readiness promise contradicting `CONTRACT.md` §8. |

## 3. Integration proposal (documentation only; no pseudo-SDK)

Proposal for the human reviewer; nothing is to be written into `engineering-corpus` by agents.

1. Placement: at most one short conceptual page ("Scene runtime: concepts and boundaries") under an existing conceptual section of the public portal, if the operator authorizes any page. No API reference, no install instructions, no code samples, no type signatures. There is no public API to document (C03).
2. Content order if approved: C01, C08, C02, C04, C03. C05, C06, C07 only after separate operator decisions on M-03, M-04, M-10.
3. Each published sentence must carry its own review record: reviewer, date, decision, and the source+revision from the table, kept in the private repo, not on the public page.
4. Required before any publication: (a) explicit operator authorization naming the page; (b) a repository-wide secret and endpoint scan (`NOT_RUN` here); (c) a read of the current public portal to avoid contradicting or duplicating it (`NOT_RUN` here); (d) a re-read of the source at the then-current SHA, since this register pins `c9dcdab` only.
5. Explicitly out of the proposal: any statement of supported versions, compatibility, performance, roadmap, or production readiness.

## 4. Claim statuses

- Wording of C01-C08 matches the cited source text at `c9dcdab`: `VERIFIED_NOW` (text comparison only).
- Truth of the underlying engine statements (laws pass, renderer works): `HISTORICAL_RECEIPT` / not re-run by A06.
- Fitness for public release of any candidate: `HYPOTHESIS`; authority `BLOCKED_AUTHORITY`.
- Public portal state, repo-wide secret scan, owner assignment: `NOT_RUN` / `UNKNOWN`.

## 5. Non-claims

This register does not establish that any sentence may be published, that the engine is production-ready, that any API exists or is planned, or that the portal contents are known. It does not authorize or perform publication, push, Coolify triggers, or writes outside its write set.

## 6. Next decision (operator)

Decide: (1) whether any public conceptual page about this runtime is wanted at all; (2) dispositions for M-02, M-03, M-04, M-10; (3) assign a documentation owner; (4) authorize the `NOT_RUN` checks in section 3.4 before any further step.
