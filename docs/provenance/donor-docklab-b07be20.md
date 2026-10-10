# Donor receipt — gnu6 Docklab

Status: **OBSERVED DONOR — NOT AUTHORITY**

Source repository: gnu6-live
Source revision: b07be200ac3dbec8e6dd896c84bb043390ea4260
Source worktree was clean at intake.

Relevant source blobs:
- app/dock-lab/layout.ts → fae8590d7ce701a12dbaf17c09506d4608c30389
- app/dock-lab/geometry.ts → 1de0182fdf7519216533d910e24455d73ec47573
- app/dock-lab/reorder-preview.ts → 667303ac1e59f92b0e6928adb2379f11b7c7a5d1

## Technical capital extracted

Docklab models a deck relative to an environment edge using:
- an origin;
- a tangent axis;
- an inward axis;
- local coordinates (u,v);
- local↔global conversion;
- edge-relative card placement;
- tarot fan composition;
- hover lift / scale;
- neighbor yield;
- reorder preview.

## Important distinction

Docklab's edge names, pinned/runtime zones, tokens, Gnosix semantics and profile taxonomy are consumer concepts.

The reusable capital is the more general statement:

> a scene collection may be laid out in a local spatial frame whose basis is attached to an environment surface.

That requirement belongs to [in.gnu6] only if a second consumer or the ontology itself supports the abstraction.

This receipt does not import Docklab source into the core.
