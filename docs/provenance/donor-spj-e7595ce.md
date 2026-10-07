# Donor receipt — SP Joaillerie Établi deck

Status: **OBSERVED DONOR — NOT AUTHORITY**

Source repository: spjoaillerie-grimoire
Source revision: e7595cea3962771453db72c0c537bfeb7f05ccc6

The source worktree had unrelated untracked artifacts at intake. The tracked donor blobs below are revision-bound.

Relevant source blobs:
- packages/app-core/src/lib/game-engine.ts → 9c59eb0b02f8a0d2485fecd3c249bc060fff1ab8
- packages/app-core/src/hooks/useGameEngine.ts → ae5b94d5591fe6977a239c697cdaf4f794c02b86
- packages/app-core/src/pages/Etabli/TarotCard.tsx → 0b91ddbc3290e1676423f3e16f2238d6b114dc91

## Technical capital extracted

SPJ contributes:
- viewport-derived card metrics;
- centered card fan;
- drag identity;
- reorder hysteresis;
- velocity-derived wobble tilt;
- playzone attraction;
- spring-rendered tarot cards.

## Important distinction

SPJ catalog/product semantics, React hooks, checkout/business state and visual branding remain SPJ-owned.

The reusable capital is:
- ordered persistent card identities;
- consumer-selected deck metrics;
- drag/reorder observations;
- spatial target regions.

This receipt does not make SPJ's GameEngine the [in.gnu6] ontology.
