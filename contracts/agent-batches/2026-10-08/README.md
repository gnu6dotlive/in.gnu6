# IN-GNU6-PAR-01 — Programme d'agents locaux parallèles

Statut: **CONTRACTS_AUTHORED / WORKERS_NOT_STARTED**. 2026-10-08. Dépôt `in.gnu6` **privé**, baseline locale `d160639e804c244a84197d8dc4906ffa2df313e8`. À l'intake, `main` était en avance de 53 commits sur `origin/main`: **ne pas pousser ou réconcilier le distant par automatisme**. Le worktree SQ-011 contient des fichiers non suivis, à préserver.

## Vérité commune

- MQ-00→MQ-15 **RATIFIED** pour le mandat actuel. Aucune nouvelle Main Quest obligatoire. Aucun worker ne peut promouvoir un side quest à lui seul.
- `EXPERIENCE-IN-01/02` **CLOSED**; consumer `experience-v6.8.0` qualifié localement, aucune autorisation de publication.
- `SQ-007` WebGPU = LATER; `SQ-008` coûts d'observation = UNSET.
- `SQ-003` provenance **donor** blob.in = CLOSED, mais **l'étude du produit blob.in** est un travail indépendant non clos par ce donor receipt.
- API publique `in.gnu6` = UNDECIDED; licence = UNDECIDED; npm `private:true`, `UNLICENSED`. Les contrats sont internes et ne peuvent pas autoriser eux-mêmes la publication.

## Huit missions parallèles

| ID | Contrat spécialisé | Write-set exclusif (depuis repo root) |
|---|---|---|
| A01 | [01-ONTOLOGY.md](01-ONTOLOGY.md) | `docs/agent-results/par-01/engine-ontology.md` |
| A02 | [02-RENDERERS.md](02-RENDERERS.md) | `docs/agent-results/par-01/renderer-envelope.md` |
| A03 | [03-CONSUMERS.md](03-CONSUMERS.md) | `docs/agent-results/par-01/consumer-bridges.md` |
| A04 | [04-GNOSUX.md](04-GNOSUX.md) | `docs/agent-results/par-01/gnosux-compatibility.md` |
| A05 | [05-PERFORMANCE.md](05-PERFORMANCE.md) | `docs/agent-results/par-01/observation-performance.md` |
| A06 | [06-DOCS-BRIDGE.md](06-DOCS-BRIDGE.md) | `docs/agent-results/par-01/public-docs-candidates.md` |
| B01 | [07-BLOBIN-RESEARCH.md](07-BLOBIN-RESEARCH.md) | `docs/agent-results/par-01/blobin-research.md` |
| B02 | [08-BLOBIN-DOCS.md](08-BLOBIN-DOCS.md) | `docs/product-dossiers/blob.in/**` |

B01 **et** B02 s'exécutent en parallèle: B02 rédige depuis la baseline vérifiée puis réconcilie les propositions de B01 via l'intégrateur, sans prétendre que les nouvelles découvertes sont déjà validées.

## Dispatch, isolation, intégration

Chaque worker lit `AGENTS.md`, `contracts/TRONC_COMMUN.md`, `CONTRACT.md`, `QUESTLOG.md`, `docs/provenance/seed-b07be20.md`, `00-BASE.md` puis son contrat spécialisé; il reconstruit HEAD et status avant tout acte durable.

Créer **un worktree + branche propres par worker**, à partir du **commit intégrant ce lot** (ne pas lancer sur `main`):
```bash
ROOT=/mnt/workbench/workspaces/gnu.in.labs/in.gnu6
BASE="<sha-du-commit-du-lot>"
for ID in A01 A02 A03 A04 A05 A06 B01 B02; do
  git -C "$ROOT" worktree add -b "feat/par-01-$ID" \
    "/mnt/workbench/workspaces/gnu.in.labs/.worktrees/in-gnu6-par-01-$ID" "$BASE"
done
```
Cette commande **prépare** seulement les worktrees; elle ne lance aucun agent. Vérifier avant de créer qu'aucune branche/path ne préexiste. Aucun worker ne modifie une autre branche, `main`, `AGENTS.md`, `CONTRACT.md`, `QUESTLOG.md`, un repo consommateur ou le snapshot historique.

Les agents rendent des **commits locaux** indépendants. L'intégrateur unique ([09-INTEGRATION.md](09-INTEGRATION.md)) inspecte chaque diff puis sélectionne ce qui est intégrable. **Aucun push GitHub, CI automatique, publication npm ou docs.gnu6.live, action VPS ou promotion de statut** n'est accordé par ce lot.

Le lot est terminé quand les huit livrables ont un HEAD, un inventaire de sources, des assertions assorties de preuves, des limites `NOT_RUN/UNKNOWN` et les décisions requises; RED documenté est acceptable, faux PASS non.
