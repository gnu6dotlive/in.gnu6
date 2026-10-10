# UPSTREAM-R&D-01 — Point d'entrée

Lot de recherche versionné, **pas une fonctionnalité livrée de [in.gnu6]**.

- [Contrat maître](../../contracts/agent-batches/2026-10-10-upstream-rd-01/00-BASE.md) et [dispatch](../../contracts/agent-batches/2026-10-10-upstream-rd-01/dispatch.json)
- [Sources exactes](../../contracts/agent-batches/2026-10-10-upstream-rd-01/sources.json) et [questlog](../../contracts/agent-batches/2026-10-10-upstream-rd-01/QUESTLOG.md)
- [Carte interprojets](./upstream-rd-01-cross-map.md)
- [12 reçus initiaux](../../evidence/upstream-rd-01/intake/source-probe.json) et 18 [reçus approfondis](../../evidence/upstream-rd-01/w01/source-evidence.json) répartis par worker W01–W06 (30 acquisitions; fichiers parfois redondants)
- [Dossiers W01–W06](../agent-results/upstream-rd-01/), [revu de licences](../agent-results/upstream-rd-01/license-review.md), [smoke hôte](../agent-results/upstream-rd-01/environment-qualification.md)
- [W07 — revue interprojets](../agent-results/upstream-rd-01/cross-analysis.md), [W08 — oracles](../agent-results/upstream-rd-01/experiments.md), [W09 — intégration](../agent-results/upstream-rd-01/integration.md), [W10 — surfaces](../agent-results/upstream-rd-01/docs-bridge.md)
- [Preuve du refus Bubblewrap hôte](../../evidence/upstream-rd-01/intake/host-denial-oracle.json) ; **MXC SDK non testé**
- [W01A MXC — source et réseau](../agent-results/upstream-rd-01/mxc-security-01.md), [W03A Deno — broker](../agent-results/upstream-rd-01/deno-permission-01.md), [W14 — composition fichiers initiale](../agent-results/upstream-rd-01/composite-01.md) et [contrat W14](../../contracts/agent-batches/2026-10-10-upstream-rd-01/14-MXC-DENO-BOUNDARY.md)
- [W14B — broker Deno réel + confinement MXC](../agent-results/upstream-rd-01/w14b-broker-net-02.md) : IPC 9/9, MXC refus réseau 3/3, composition broker×MXC 5/5; egress positif et Judgment Plane non qualifiés
- [Brouillon public, non publié](./public-candidates/upstream-rd-01-fr.md)

Source de branche: `in.gnu6 origin/main@4042faa`. Le main local `d160639` (53 commits en avance) a été expressément exclu. Six sources GitHub ont été consultées à des SHA précis (30 acquisitions de fichiers, incluant quelques chemins revus deux fois). Six workers déterministes ont produit des commits réconciliés. Un test Bubblewrap hôte borné a passé; ni SDK MXC, ni compilation GPU, ni intégration produit, ni agent LLM autonome, ni déploiement public n'a été qualifié.
