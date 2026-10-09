# CUDA-05D — Post-acquisition implementation amendment 01

**Date:** 2026-10-09 UTC. **Protocol:** CUDA-05D-P01 at `ee0ea36bae0950c04e13100c7f53cf7e643c82c4`. **Nature:** provenance pin repair, not a change of tested question, inputs, gates or outcomes.

## Identified issue

The initial preregistered `evaluate05d.py` attached the **current HEAD** to an interpretation. This is correct for the first run immediately after preregistration, but subsequent reruns after committing evidence would incorrectly label the new branch HEAD as the frozen protocol commit. The flaw concerns audit lineage, not the observation acquisition.

## Repair and impact

Replace the moving HEAD with the first Git commit that **added** `05D-PREREG.md` (`git log --diff-filter=A --format=%H -- <exact path>`). Require exactly one introduction. The original raw observation JSON remains untouched; no output values or decision rules are revised. Regenerate derived interpretation/public snapshot and their hashes. The independent checker must verify the protocol file bytes exactly against the pinned Git object and reject falsified peer-UID data.

**Scientific qualification:** This is a disclosed post-acquisition code amendment. Neither the original protocol nor its frozen commit is rewritten. A future measurement run is a new observation, not evidence that the first run was repeated.
