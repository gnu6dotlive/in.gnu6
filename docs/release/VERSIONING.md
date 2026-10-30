# Versions et checkpoints — [in.gnu6]

**Décision initiale : 2026-10-10.** La numérotation raconte une qualification ; elle ne remplace jamais ses preuves.

## Trois identités à ne pas confondre

1. **Version de travail npm** — calendrier `YYYY.MM.REVISION-CHANNEL`, syntaxe compatible SemVer pour l'outillage, mais **sémantique CalVer** explicitement documentée. Première version de travail : `2026.10.0-research`. Le paquet reste `private: true` (le code du dépôt est sous licence MIT depuis le 2026-10-30) ; une valeur de `version` ne crée aucun contrat public.
2. **Checkpoint d'ingénierie** — une portée + un résultat + un SHA complet + les empreintes de preuve. Exemple : `GPU05.PACER07@4f4ae70…`, `GPU05.ENDURANCE01@0fa5d12…`. Un checkpoint peut être `LOCAL_PASS`, `RED` ou `OPEN`; il n'est pas nécessairement une release.
3. **Release publique** — événement distinct, seulement après une licence choisie, un contrat d'API/distribution, un inventaire de dépendances, l'audit de confidentialité et les gates de portabilité pertinents. Les tags publics sont créés **uniquement au moment de cet événement**, jamais rétroactivement pour maquiller un essai local.

## Règle calendaire

- `YYYY` : année du train de travail.
- `MM` : mois du train.
- `REVISION` : nouvelle itération **justifiée par un périmètre de qualification ou une rupture de contrat**, pas compteur automatique de commits.
- `CHANNEL` : `research`, `preview` ou `rc.N` jusqu'à ratification. Une version stable exige le contrat de sortie ; `research` ne doit pas être supprimé pour embellir le README.
- Une nouvelle observation négative est un progrès scientifique, **pas nécessairement une nouvelle version npm**.

Pas de série `0.0.x` fourre-tout, pas de tag de release pour chaque micro-patch, pas de revendication de stabilité d'API fondée sur les nombres.

## Registre initial de référence

| Jalon | Source durable | Résultat |
|---|---|---|
| Noyau sémantique MQ-00→15 | Main Questline, tests et dossiers MQ | RATIFIED pour mandat explicitement borné |
| GPU05.PACER07 | `4f4ae70ad7563620c4a5946e3a3f8ff71835539f` | Reversed startup A/B local : première frame présentée avec barrière |
| GPU05.ENDURANCE01 | `0fa5d12704a5f8fd89d2396697eab35977b7f45c` | 125 s de présentation réelle, zéro discarded sur une configuration de référence |
| CP03.HYBRID60 | Contrat CP03, vérification indépendante | **OPEN**, notamment entrée-à-photon |

Le merge documentaire d'un résultat expérimental ne change pas automatiquement son niveau de qualification. La première release ne peut pas simplement hériter du statut d'un checkpoint.

## Règle de publication

Toute note publique doit répondre en cinq points : **fait**, **environnement**, **méthode de mesure**, **limites**, **preuve attribuable**. Les chemins absolus des machines, tokens de session et données sensibles restent hors de la surface de présentation publique. Une capture ou un graphique non étayé n'est pas une preuve.
